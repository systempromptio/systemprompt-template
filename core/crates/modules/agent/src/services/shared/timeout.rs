use crate::services::shared::error::{AgentServiceError, Result};
use std::time::Duration;
use tokio::time::timeout;

pub async fn execute_with_timeout<F, T>(duration: Duration, operation: F) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    match timeout(duration, operation).await {
        Ok(result) => result,
        Err(_) => Err(AgentServiceError::Timeout(duration.as_millis() as u64)),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TimeoutConfiguration {
    pub default_timeout: Duration,
    pub connect_timeout: Duration,
    pub read_timeout: Duration,
    pub write_timeout: Duration,
}

impl Default for TimeoutConfiguration {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            read_timeout: Duration::from_secs(30),
            write_timeout: Duration::from_secs(30),
        }
    }
}

pub async fn execute_with_custom_timeout<F, T>(
    config: TimeoutConfiguration,
    timeout_type: TimeoutType,
    operation: F,
) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    let duration = match timeout_type {
        TimeoutType::Connect => config.connect_timeout,
        TimeoutType::Read => config.read_timeout,
        TimeoutType::Write => config.write_timeout,
        TimeoutType::Default => config.default_timeout,
    };

    execute_with_timeout(duration, operation).await
}

#[derive(Debug, Clone, Copy)]
pub enum TimeoutType {
    Connect,
    Read,
    Write,
    Default,
}
