use anyhow::{anyhow, Result};
use std::time::Duration;
use systemprompt_core_logging::CliService;
use tokio::time::sleep;

#[derive(Debug)]
pub struct HealthChecker {
    url: String,
    max_retries: u32,
    retry_delay: Duration,
}

impl HealthChecker {
    pub const fn new(url: String) -> Self {
        Self {
            url,
            max_retries: 20,
            retry_delay: Duration::from_secs(3),
        }
    }

    pub const fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub const fn with_retry_delay(mut self, retry_delay: Duration) -> Self {
        self.retry_delay = retry_delay;
        self
    }

    pub async fn check(&self) -> Result<()> {
        CliService::info("🏥 Performing health checks...");

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        for attempt in 0..self.max_retries {
            sleep(self.retry_delay).await;

            match client.get(&self.url).send().await {
                Ok(response) if response.status().is_success() => {
                    CliService::success("API health check passed");
                    return Ok(());
                },
                Ok(response) => {
                    let remaining = self.max_retries - attempt - 1;
                    CliService::warning(&format!(
                        "Health check attempt {}/{} (HTTP {}, retrying in {}s, {} retries left)...",
                        attempt + 1,
                        self.max_retries,
                        response.status(),
                        self.retry_delay.as_secs(),
                        remaining
                    ));
                },
                Err(e) => {
                    let remaining = self.max_retries - attempt - 1;
                    CliService::warning(&format!(
                        "Health check attempt {}/{} (error: {}, retrying in {}s, {} retries \
                         left)...",
                        attempt + 1,
                        self.max_retries,
                        e,
                        self.retry_delay.as_secs(),
                        remaining
                    ));
                },
            }
        }

        Err(anyhow!(
            "Health check failed after {} attempts ({} seconds)",
            self.max_retries,
            self.max_retries * self.retry_delay.as_secs() as u32
        ))
    }
}
