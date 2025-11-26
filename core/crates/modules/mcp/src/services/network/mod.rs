pub mod port_manager;
pub mod proxy;
pub mod routing;

use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub struct NetworkManager;

impl Default for NetworkManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkManager {
    pub const fn new() -> Self {
        Self
    }

    pub async fn prepare_port(&self, port: u16) -> Result<()> {
        port_manager::prepare_port(port).await
    }

    pub async fn is_port_responsive(&self, port: u16) -> Result<bool> {
        port_manager::is_port_responsive(port).await
    }

    pub async fn wait_for_port_release(&self, port: u16) -> Result<()> {
        port_manager::wait_for_port_release(port).await
    }

    pub async fn cleanup_port_resources(&self, port: u16) -> Result<()> {
        port_manager::cleanup_port_resources(port).await
    }

    pub async fn create_router(&self) -> Result<axum::Router> {
        routing::create_base_router().await
    }

    pub async fn apply_cors(&self, router: axum::Router) -> Result<axum::Router> {
        routing::apply_cors_layer(router).await
    }

    pub async fn create_proxy(&self, target_host: &str, target_port: u16) -> Result<axum::Router> {
        proxy::create_proxy_router(target_host, target_port).await
    }
}
