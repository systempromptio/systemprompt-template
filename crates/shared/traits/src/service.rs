use async_trait::async_trait;

#[async_trait]
pub trait Service: Send + Sync {
    fn name(&self) -> &str;

    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
pub trait AsyncService: Service {
    async fn run(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
