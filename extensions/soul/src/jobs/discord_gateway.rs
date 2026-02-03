use anyhow::{Context, Result};
use serenity::Client;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::discord::{DiscordConfigValidated, DiscordHandler};

#[derive(Debug, Clone, Copy, Default)]
pub struct DiscordGatewayJob;

impl DiscordGatewayJob {
    pub async fn execute_gateway() -> Result<JobResult> {
        let start = std::time::Instant::now();

        let config = DiscordConfigValidated::load_from_default_paths()
            .context("Failed to load Discord config")?;

        if !config.is_enabled() {
            tracing::info!("Discord is disabled in config, skipping gateway");
            return Ok(JobResult::success()
                .with_stats(0, 0)
                .with_duration(start.elapsed().as_millis() as u64));
        }

        let gateway_config = config.gateway();
        if !gateway_config.enabled {
            tracing::info!("Discord gateway is disabled in config");
            return Ok(JobResult::success()
                .with_stats(0, 0)
                .with_duration(start.elapsed().as_millis() as u64));
        }

        tracing::info!(
            target_agent = %gateway_config.target_agent,
            message_prefix = %gateway_config.message_prefix,
            "Starting Discord gateway bot"
        );

        let handler = DiscordHandler::try_new(gateway_config.clone())
            .context("Failed to create Discord handler")?;

        tracing::info!(
            "Discord handler created, CLI binary validated"
        );

        let intents = DiscordHandler::required_intents();

        let mut client = Client::builder(config.bot_token(), intents)
            .event_handler(handler)
            .await
            .context("Failed to create Discord client")?;

        tracing::info!("Discord gateway client starting...");

        client
            .start()
            .await
            .context("Discord client connection failed")?;

        #[allow(clippy::cast_possible_truncation)]
        let duration_ms = start.elapsed().as_millis() as u64;
        Ok(JobResult::success()
            .with_stats(1, 0)
            .with_duration(duration_ms))
    }
}

#[async_trait::async_trait]
impl Job for DiscordGatewayJob {
    fn name(&self) -> &'static str {
        "soul_discord_gateway"
    }

    fn description(&self) -> &'static str {
        "Connects to Discord Gateway to receive messages and forward them to agents. \
         Configure with services/config/discord.yaml gateway section."
    }

    fn schedule(&self) -> &'static str {
        "@reboot"
    }

    fn run_on_startup(&self) -> bool {
        true
    }

    async fn execute(&self, _ctx: &JobContext) -> Result<JobResult> {
        Self::execute_gateway().await
    }
}

systemprompt::traits::submit_job!(&DiscordGatewayJob);
