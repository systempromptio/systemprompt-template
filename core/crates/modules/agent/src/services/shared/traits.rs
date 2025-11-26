use crate::services::shared::config::ConfigValidation;
use crate::services::shared::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ServiceLifecycle: Send + Sync {
    type Config: ConfigValidation;

    async fn initialize(config: Self::Config) -> Result<Self>
    where
        Self: Sized;

    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;

    fn name(&self) -> &str;
}

#[async_trait]
pub trait Service: ServiceLifecycle {
    fn capabilities(&self) -> Vec<String>;
    fn version(&self) -> &str;
}
