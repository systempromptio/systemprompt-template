use anyhow::Result;
use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer};

use crate::server::AdminServer;

impl AdminServer {
    pub(in crate::server) fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_tools()
                .enable_resources()
                .build(),
            server_info: Implementation {
                name: format!("SystemPrompt Admin MCP Server ({})", self.service_id),
                version: "2.0.0".to_string(),
                icons: None,
                title: Some("Admin Server".into()),
                website_url: Some("https://systemprompt.io".to_string()),
            },
            instructions: Some(
                "SystemPrompt Admin MCP Server - Comprehensive administrative tooling for \
                user management, analytics, system monitoring, OAuth management, MCP monitoring, \
                task tracking, and agent management. All tools return rich artifacts for visualization."
                    .to_string()
            ),
        }
    }

    pub(in crate::server) async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        self.system_log
            .info("mcp_initialize", "=== ADMIN SERVER INITIALIZE ===")
            .await
            .ok();

        if let Some(parts) = context.extensions.get::<axum::http::request::Parts>() {
            self.system_log
                .info(
                    "mcp_admin",
                    &format!(
                        "Admin MCP initialized - URI: {}, server: {}",
                        parts.uri, self.service_id
                    ),
                )
                .await
                .ok();
        }

        Ok(self.get_info())
    }
}
