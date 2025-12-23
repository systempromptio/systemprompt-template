use anyhow::Result;
use rmcp::{
    model::{
        Implementation, InitializeRequestParam, InitializeResult, ProtocolVersion,
        ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    ErrorData as McpError, RoleServer,
};

use crate::server::InfrastructureServer;

impl InfrastructureServer {
    pub(in crate::server) fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_tools()
                .enable_resources()
                .build(),
            server_info: Implementation {
                name: format!(
                    "SystemPrompt Infrastructure MCP Server ({})",
                    self.service_id
                ),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                title: Some("Infrastructure Server".into()),
                website_url: Some("https://systemprompt.io".to_string()),
            },
            instructions: Some(
                "SystemPrompt Infrastructure MCP Server - Cloud sync tools for deploying and \
                synchronizing SystemPrompt applications. Provides file sync, database sync, \
                crate deployment, and status monitoring capabilities."
                    .to_string(),
            ),
        }
    }

    #[allow(clippy::unused_async)]
    pub(in crate::server) async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        tracing::info!("=== INFRASTRUCTURE SERVER INITIALIZE ===");

        if let Some(parts) = context.extensions.get::<axum::http::request::Parts>() {
            tracing::info!(
                uri = %parts.uri,
                service_id = %self.service_id,
                "Infrastructure MCP initialized"
            );
        }

        Ok(self.get_info())
    }
}
