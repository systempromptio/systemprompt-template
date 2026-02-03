use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Mutex;

use serde_json;
use serenity::all::{Context, EventHandler, GatewayIntents, Message, Ready};
use systemprompt::cloud::ProjectContext;
use systemprompt::loader::ExtensionLoader;
use tokio::process::Command;

use crate::discord::GatewayConfig;
use crate::SoulError;

const MAX_TRACKED_MESSAGES: usize = 1000;

pub struct DiscordHandler {
    config: GatewayConfig,
    cli_path: PathBuf,
    processed_ids: Mutex<HashSet<String>>,
}

impl DiscordHandler {
    pub fn try_new(config: GatewayConfig) -> Result<Self, SoulError> {
        let cli_path = Self::resolve_cli_binary()?;
        Ok(Self {
            config,
            cli_path,
            processed_ids: Mutex::new(HashSet::new()),
        })
    }

    fn resolve_cli_binary() -> Result<PathBuf, SoulError> {
        let project = ProjectContext::discover();
        let project_root = project.root();

        ExtensionLoader::get_cli_binary_path(project_root, "systemprompt").ok_or_else(|| {
            SoulError::Configuration(format!(
                "systemprompt binary not found in {}/target/{{release,debug}}/",
                project_root.display()
            ))
        })
    }

    #[must_use]
    pub fn required_intents() -> GatewayIntents {
        GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT
    }

    async fn forward_to_agent(&self, formatted_message: &str) -> Option<String> {
        let result = Command::new(&self.cli_path)
            .args(["admin", "agents", "message", &self.config.target_agent])
            .args(["-m", formatted_message])
            .args(["--blocking", "--timeout", "300"])
            .output()
            .await;

        match result {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    tracing::info!(
                        agent = %self.config.target_agent,
                        response_preview = %stdout.chars().take(200).collect::<String>(),
                        "Discord message forwarded to agent successfully"
                    );
                    let response = stdout.trim().to_string();
                    if response.is_empty() {
                        None
                    } else {
                        Some(response)
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::error!(
                        agent = %self.config.target_agent,
                        error = %stderr,
                        "Failed to forward Discord message to agent"
                    );
                    None
                }
            }
            Err(e) => {
                tracing::error!(
                    agent = %self.config.target_agent,
                    error = %e,
                    "Failed to execute agent message command"
                );
                None
            }
        }
    }
}

#[serenity::async_trait]
impl EventHandler for DiscordHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        let message_id = msg.id.to_string();

        {
            let mut processed = match self.processed_ids.lock() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    tracing::error!("processed_ids mutex poisoned, recovering with poisoned guard");
                    poisoned.into_inner()
                }
            };

            if processed.len() >= MAX_TRACKED_MESSAGES {
                processed.clear();
            }

            if !processed.insert(message_id.clone()) {
                tracing::debug!(message_id = %message_id, "Skipping duplicate message");
                return;
            }
        }

        if self.config.ignore_bots && msg.author.bot {
            tracing::trace!(
                author = %msg.author.name,
                "Ignoring bot message"
            );
            return;
        }

        let channel_id_str = msg.channel_id.to_string();
        if self.config.ignore_channels.contains(&channel_id_str) {
            tracing::trace!(
                channel_id = %channel_id_str,
                "Ignoring message from ignored channel"
            );
            return;
        }

        let channel_name = msg
            .channel_id
            .name(&ctx)
            .await
            .unwrap_or_else(|_| "DM".to_string());

        let formatted = serde_json::json!({
            "type": self.config.message_prefix,
            "message_id": msg.id.to_string(),
            "channel_id": channel_id_str,
            "channel_name": channel_name,
            "author": msg.author.name,
            "content": msg.content
        })
        .to_string();

        tracing::info!(
            channel = %channel_name,
            author = %msg.author.name,
            content_preview = %msg.content.chars().take(50).collect::<String>(),
            "Received Discord message, forwarding to agent"
        );

        if let Some(response) = self.forward_to_agent(&formatted).await {
            const MAX_MESSAGE_LENGTH: usize = 2000;

            if response.len() <= MAX_MESSAGE_LENGTH {
                if let Err(e) = msg.channel_id.say(&ctx, &response).await {
                    tracing::error!(error = %e, "Failed to send reply to Discord");
                }
            } else {
                let mut remaining = response.as_str();
                while !remaining.is_empty() {
                    let split_at = remaining
                        .char_indices()
                        .take_while(|(i, _)| *i < MAX_MESSAGE_LENGTH)
                        .last()
                        .map_or(remaining.len(), |(i, c)| i + c.len_utf8());
                    let (chunk, rest) = remaining.split_at(split_at.min(remaining.len()));
                    if let Err(e) = msg.channel_id.say(&ctx, chunk).await {
                        tracing::error!(error = %e, "Failed to send reply chunk to Discord");
                        break;
                    }
                    remaining = rest;
                }
            }
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        tracing::info!(
            bot_name = %ready.user.name,
            guild_count = ready.guilds.len(),
            "Discord gateway connected"
        );
    }
}
