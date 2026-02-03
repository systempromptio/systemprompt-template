use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;

use crate::discord::DiscordConfigValidated;

const DISCORD_API_BASE: &str = "https://discord.com/api/v10";
const HTTP_TIMEOUT_SECONDS: u64 = 30;
const DEFAULT_RATE_LIMIT_SECONDS: u64 = 60;

pub struct DiscordService {
    client: Client,
    config: DiscordConfigValidated,
}

#[derive(Debug, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    pub channel_id: String,
}

#[derive(Debug, Deserialize)]
struct DmChannel {
    id: String,
}

#[derive(Debug, Deserialize)]
struct BotUser {
    pub username: String,
    pub discriminator: String,
}

impl DiscordService {
    pub fn new(config: DiscordConfigValidated) -> anyhow::Result<Self> {
        if !config.is_enabled() {
            anyhow::bail!("Discord extension is not enabled in config");
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(HTTP_TIMEOUT_SECONDS))
            .build()?;

        Ok(Self { client, config })
    }

    pub async fn test_connection(&self) -> anyhow::Result<String> {
        let url = format!("{DISCORD_API_BASE}/users/@me");

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bot {}", self.config.bot_token()))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let bot: BotUser = response.json().await?;
            Ok(format!("{}#{}", bot.username, bot.discriminator))
        } else {
            let error_body = response.text().await.unwrap_or_else(|_| String::new());
            anyhow::bail!("Discord API error ({status}): {error_body}")
        }
    }

    pub async fn send_channel_message(
        &self,
        channel_id: &str,
        content: &str,
    ) -> anyhow::Result<MessageResponse> {
        let url = format!("{DISCORD_API_BASE}/channels/{channel_id}/messages");
        self.send_message_internal(&url, content).await
    }

    pub async fn send_dm(&self, user_id: &str, content: &str) -> anyhow::Result<MessageResponse> {
        let dm_channel = self.create_dm_channel(user_id).await?;
        let url = format!("{DISCORD_API_BASE}/channels/{}/messages", dm_channel.id);
        self.send_message_internal(&url, content).await
    }

    pub async fn send_to_default(&self, content: &str) -> anyhow::Result<MessageResponse> {
        if let Some(channel_id) = self.config.default_channel_id() {
            self.send_channel_message(channel_id, content).await
        } else if let Some(user_id) = self.config.default_user_id() {
            self.send_dm(user_id, content).await
        } else {
            anyhow::bail!(
                "No default target configured (set default_channel_id or default_user_id)"
            )
        }
    }

    async fn create_dm_channel(&self, user_id: &str) -> anyhow::Result<DmChannel> {
        let url = format!("{DISCORD_API_BASE}/users/@me/channels");

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.config.bot_token()))
            .json(&serde_json::json!({ "recipient_id": user_id }))
            .send()
            .await?;

        self.handle_response(response).await
    }

    async fn send_message_internal(
        &self,
        url: &str,
        content: &str,
    ) -> anyhow::Result<MessageResponse> {
        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bot {}", self.config.bot_token()))
            .json(&serde_json::json!({ "content": content }))
            .send()
            .await?;

        self.handle_response(response).await
    }

    async fn handle_response<T: for<'de> Deserialize<'de>>(
        &self,
        response: reqwest::Response,
    ) -> anyhow::Result<T> {
        let status = response.status();

        if status.is_success() {
            Ok(response.json().await?)
        } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(DEFAULT_RATE_LIMIT_SECONDS);
            anyhow::bail!("Rate limited by Discord. Retry after {retry_after} seconds")
        } else if status == reqwest::StatusCode::NOT_FOUND {
            let error_body = response.text().await.unwrap_or_else(|_| String::new());
            if error_body.contains("Unknown Channel") {
                anyhow::bail!("Channel not found: {error_body}")
            }
            if error_body.contains("Unknown User") {
                anyhow::bail!("User not found: {error_body}")
            }
            anyhow::bail!("Discord API error ({status}): {error_body}")
        } else if status == reqwest::StatusCode::FORBIDDEN {
            let error_body = response.text().await.unwrap_or_else(|_| String::new());
            if error_body.contains("Cannot send messages to this user") {
                anyhow::bail!(
                    "Cannot send DM to this user. Make sure they share a server with the bot."
                )
            }
            anyhow::bail!("Discord API forbidden ({status}): {error_body}")
        } else {
            let error_body = response.text().await.unwrap_or_else(|_| String::new());
            anyhow::bail!("Discord API error ({status}): {error_body}")
        }
    }
}
