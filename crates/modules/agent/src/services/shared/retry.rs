use crate::services::shared::error::{AgentServiceError, Result};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone, Copy)]
pub struct RetryConfiguration {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub exponential_base: u32,
}

impl Default for RetryConfiguration {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            exponential_base: 2,
        }
    }
}

pub async fn retry_operation<F, Fut, T>(operation: F, config: RetryConfiguration) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut current_delay = config.initial_delay;

    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) if attempt == config.max_attempts => return Err(error),
            Err(_) => {
                sleep(current_delay).await;
                current_delay = calculate_next_delay(current_delay, &config);
            },
        }
    }

    Err(AgentServiceError::Configuration(
        "RetryConfiguration".to_string(),
        "Retry configuration resulted in no attempts".to_string(),
    ))
}

fn calculate_next_delay(current: Duration, config: &RetryConfiguration) -> Duration {
    let next = current.saturating_mul(config.exponential_base);
    if next > config.max_delay {
        config.max_delay
    } else {
        next
    }
}

pub async fn retry_operation_with_backoff<F, Fut, T>(
    operation: F,
    max_attempts: u32,
    initial_delay: Duration,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let config = RetryConfiguration {
        max_attempts,
        initial_delay,
        ..Default::default()
    };
    retry_operation(operation, config).await
}
