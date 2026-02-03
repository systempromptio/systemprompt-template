mod constructor;

pub use constructor::ContentManagerServer;

use crate::tools;
use anyhow::Result;
use chrono::Utc;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Implementation, InitializeRequestParams,
    InitializeResult, ListToolsResult, PaginatedRequestParams, ProgressNotificationParam,
    ProgressToken, ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::service::{Peer, RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use std::future::Future;
use std::pin::Pin;
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::mcp::models::{ExecutionStatus, ToolExecutionRequest, ToolExecutionResult};

pub type ProgressCallback = Box<
    dyn Fn(f64, Option<f64>, Option<String>) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send
        + Sync,
>;

fn create_progress_callback(token: ProgressToken, peer: Peer<RoleServer>) -> ProgressCallback {
    Box::new(
        move |progress: f64, total: Option<f64>, message: Option<String>| {
            let token = token.clone();
            let peer = peer.clone();
            Box::pin(async move {
                let _ = peer
                    .notify_progress(ProgressNotificationParam {
                        progress_token: token,
                        progress,
                        total,
                        message,
                    })
                    .await;
            }) as Pin<Box<dyn Future<Output = ()> + Send>>
        },
    )
}

impl ServerHandler for ContentManagerServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: format!("Content Manager ({})", self.service_id),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                title: Some("Content Manager".to_string()),
                website_url: None,
            },
            instructions: Some(
                "Content management MCP server for creating and managing blog content with AI. \
                 Use research_blog to research a topic, then create_blog_post to generate content. \
                 Use generate_featured_image to create striking featured images for blog posts."
                    .to_string(),
            ),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        tracing::info!("Content Manager MCP server initialized");
        Ok(self.get_info())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult {
            tools: tools::list_tools(),
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name = request.name.to_string();
        let started_at = Utc::now();

        let auth_result = enforce_rbac_from_registry(&ctx, self.service_id.as_str()).await?;
        let authenticated_ctx = auth_result
            .expect_authenticated("content-manager requires OAuth but auth was not enforced")?;

        let request_context = authenticated_ctx.context.clone();

        let progress_callback = ctx
            .meta
            .get_progress_token()
            .map(|token| create_progress_callback(token.clone(), ctx.peer.clone()));

        let execution_request = ToolExecutionRequest {
            tool_name: tool_name.clone(),
            server_name: self.service_id.to_string(),
            input: serde_json::to_value(&request.arguments).unwrap_or(serde_json::Value::Null),
            started_at,
            context: request_context.clone(),
            request_method: Some("mcp".to_string()),
            request_source: Some("content-manager".to_string()),
            ai_tool_call_id: None,
        };

        let mcp_execution_id = self
            .tool_usage_repo
            .start_execution(&execution_request)
            .await
            .map_err(|e| {
                tracing::error!(tool = %tool_name, error = %e, "Failed to start execution tracking");
                McpError::internal_error(format!("Failed to start execution tracking: {e}"), None)
            })?;

        let result = tools::handle_tool_call(
            &tool_name,
            request,
            request_context,
            &self.db_pool,
            &self.ai_service,
            &self.image_service,
            &self.skill_loader,
            &self.artifact_repo,
            progress_callback,
            &mcp_execution_id,
        )
        .await;

        let completed_at = Utc::now();
        let execution_result = ToolExecutionResult {
            output: result
                .as_ref()
                .ok()
                .and_then(|r| r.structured_content.clone()),
            output_schema: None,
            status: if result.is_ok() {
                ExecutionStatus::Success.as_str().to_string()
            } else {
                ExecutionStatus::Failed.as_str().to_string()
            },
            error_message: result.as_ref().err().map(|e| e.message.to_string()),
            started_at,
            completed_at,
        };

        if let Err(e) = self
            .tool_usage_repo
            .complete_execution(&mcp_execution_id, &execution_result)
            .await
        {
            tracing::error!(
                tool = %tool_name,
                mcp_execution_id = %mcp_execution_id,
                error = %e,
                "Failed to complete execution tracking"
            );
        }

        if let Err(ref e) = result {
            tracing::error!(tool = %tool_name, error = %e, "Tool failed");
        }

        result
    }
}
