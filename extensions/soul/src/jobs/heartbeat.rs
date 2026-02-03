use anyhow::Result;
use systemprompt::traits::{Job, JobContext, JobResult};

#[derive(Debug, Clone, Copy, Default)]
pub struct HeartbeatJob;

impl HeartbeatJob {
    fn get_config() -> HeartbeatConfig {
        HeartbeatConfig {
            agent_name: std::env::var("SOUL_HEARTBEAT_AGENT")
                .unwrap_or_else(|_| "systemprompt_hub".to_string()),
            message: std::env::var("SOUL_HEARTBEAT_MESSAGE")
                .unwrap_or_else(|_| "Heartbeat: Review recent activity, generate a brief status update, and send it to Discord.".to_string()),
            timeout_secs: std::env::var("SOUL_HEARTBEAT_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(120),
            blocking: std::env::var("SOUL_HEARTBEAT_BLOCKING")
                .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
                .unwrap_or(true),
        }
    }

    pub async fn execute_heartbeat() -> Result<JobResult> {
        let start = std::time::Instant::now();
        let config = Self::get_config();

        tracing::info!(
            agent = %config.agent_name,
            message_preview = %config.message.chars().take(50).collect::<String>(),
            "Heartbeat job started"
        );

        let mut cmd = tokio::process::Command::new("systemprompt");
        cmd.args(["admin", "agents", "message", &config.agent_name]);
        cmd.args(["-m", &config.message]);

        if config.blocking {
            cmd.args(["--blocking", "--timeout", &config.timeout_secs.to_string()]);
        }

        let output = cmd.output().await?;

        #[allow(clippy::cast_possible_truncation)]
        let duration_ms = start.elapsed().as_millis() as u64;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            tracing::info!(
                agent = %config.agent_name,
                duration_ms,
                response_preview = %stdout.chars().take(200).collect::<String>(),
                "Heartbeat completed successfully"
            );
            Ok(JobResult::success()
                .with_stats(1, 0)
                .with_duration(duration_ms))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!(
                agent = %config.agent_name,
                duration_ms,
                error = %stderr,
                "Heartbeat failed"
            );
            Ok(JobResult::success()
                .with_stats(0, 1)
                .with_duration(duration_ms))
        }
    }
}

#[derive(Debug, Clone)]
struct HeartbeatConfig {
    agent_name: String,
    message: String,
    timeout_secs: u64,
    blocking: bool,
}

#[async_trait::async_trait]
impl Job for HeartbeatJob {
    fn name(&self) -> &'static str {
        "soul_heartbeat"
    }

    fn description(&self) -> &'static str {
        "Sends a scheduled message to an agent via A2A protocol. Configure with SOUL_HEARTBEAT_AGENT, SOUL_HEARTBEAT_MESSAGE, SOUL_HEARTBEAT_TIMEOUT."
    }

    fn schedule(&self) -> &'static str {
        "0 0 * * * *"
    }

    async fn execute(&self, _ctx: &JobContext) -> Result<JobResult> {
        Self::execute_heartbeat().await
    }
}

systemprompt::traits::submit_job!(&HeartbeatJob);
