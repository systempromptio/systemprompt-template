pub mod cleanup;
pub mod monitor;
pub mod pid_manager;
pub mod spawner;

use crate::McpServerConfig;
use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_system::AppContext;

#[derive(Debug, Clone)]
pub struct ProcessManager {
    app_context: Arc<AppContext>,
}

impl ProcessManager {
    pub const fn new(app_context: Arc<AppContext>) -> Self {
        Self { app_context }
    }

    pub async fn spawn_server(&self, config: &McpServerConfig) -> Result<u32> {
        spawner::spawn_server(self, config).await
    }

    pub async fn is_running(&self, pid: u32) -> Result<bool> {
        monitor::is_process_running(pid).await
    }

    pub async fn find_pid_by_port(&self, port: u16) -> Result<Option<u32>> {
        pid_manager::find_pid_by_port(port).await
    }

    pub async fn find_process_on_port_with_name(
        &self,
        port: u16,
        name: &str,
    ) -> Result<Option<u32>> {
        pid_manager::find_process_on_port_with_name(port, name).await
    }

    pub async fn verify_binary(&self, config: &McpServerConfig) -> Result<()> {
        spawner::verify_binary(config).await
    }

    pub async fn build_server(&self, config: &McpServerConfig) -> Result<()> {
        spawner::build_server(config).await
    }

    pub async fn terminate_gracefully(&self, pid: u32) -> Result<()> {
        cleanup::terminate_gracefully(pid).await
    }

    pub async fn force_kill(&self, pid: u32) -> Result<()> {
        cleanup::force_kill(pid).await
    }

    pub const fn app_context(&self) -> &Arc<AppContext> {
        &self.app_context
    }
}
