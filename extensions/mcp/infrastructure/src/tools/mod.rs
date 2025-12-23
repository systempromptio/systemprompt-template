mod config;
mod deploy;
mod export;
mod status;
mod sync;

use rmcp::{
    model::{CallToolRequestMethod, CallToolRequestParam, CallToolResult, ListToolsResult, Tool},
    service::RequestContext,
    ErrorData as McpError, RoleServer,
};
use std::sync::Arc;
use systemprompt::identifiers::McpExecutionId;

use crate::sync::SyncService;

pub use config::{config_input_schema, config_output_schema, handle_config};
pub use deploy::{deploy_input_schema, deploy_output_schema, handle_deploy};
pub use export::{export_input_schema, export_output_schema, handle_export};
pub use status::{handle_status, status_input_schema, status_output_schema};
pub use sync::{handle_sync, sync_input_schema, sync_output_schema};

#[must_use]
pub fn register_tools() -> Vec<Tool> {
    vec![
        create_tool(
            "sync",
            "Sync",
            "Sync between local environment and cloud. Targets: files (service configs), database (agents/skills/contexts), content (blog/legal), skills, or all.",
            &sync_input_schema(),
            &sync_output_schema(),
        ),
        create_tool(
            "export",
            "Export",
            "Export data to disk for backup. Targets: content (blog/legal), skills, or all.",
            &export_input_schema(),
            &export_output_schema(),
        ),
        create_tool(
            "deploy",
            "Deploy",
            "Build and deploy the application to cloud. Builds Rust binary, web assets, creates Docker image, and deploys to Fly.io.",
            &deploy_input_schema(),
            &deploy_output_schema(),
        ),
        create_tool(
            "status",
            "Status",
            "Get current cloud status including connection state, deployment status, and configuration.",
            &status_input_schema(),
            &status_output_schema(),
        ),
        create_tool(
            "config",
            "Config",
            "Display current configuration. Filter by section (agents, mcp, skills, ai, web, content, env, settings) and format (json, yaml, summary).",
            &config_input_schema(),
            &config_output_schema(),
        ),
    ]
}

pub async fn handle_tool_call(
    name: &str,
    request: CallToolRequestParam,
    ctx: RequestContext<RoleServer>,
    sync_service: &Arc<SyncService>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    match name {
        "sync" => handle_sync(sync_service, request, ctx, mcp_execution_id).await,
        "export" => handle_export(sync_service, request, ctx, mcp_execution_id).await,
        "deploy" => handle_deploy(sync_service, request, ctx, mcp_execution_id).await,
        "status" => handle_status(sync_service, request, ctx, mcp_execution_id).await,
        "config" => handle_config(sync_service, request, ctx, mcp_execution_id).await,
        _ => {
            tracing::warn!(tool = %name, "Unknown tool");
            Err(McpError::method_not_found::<CallToolRequestMethod>())
        }
    }
}

pub fn list_tools() -> Result<ListToolsResult, McpError> {
    Ok(ListToolsResult {
        tools: register_tools(),
        next_cursor: None,
    })
}

#[allow(clippy::needless_pass_by_value)]
pub fn map_credential_error(err: anyhow::Error) -> McpError {
    let msg = err.to_string();
    if msg.contains("credentials")
        || msg.contains("NotAuthenticated")
        || msg.contains("NotAvailable")
    {
        McpError::internal_error(
            "Cloud credentials not found. Run 'systemprompt cloud login'",
            None,
        )
    } else if msg.contains("expired") || msg.contains("TokenExpired") {
        McpError::internal_error("Token expired. Run 'systemprompt cloud login'", None)
    } else if msg.contains("tenant") || msg.contains("TenantNotConfigured") {
        McpError::internal_error("No tenant configured. Run 'systemprompt cloud setup'", None)
    } else {
        McpError::internal_error(format!("Cloud error: {msg}"), None)
    }
}

fn create_tool(
    name: &str,
    title: &str,
    description: &str,
    input_schema: &serde_json::Value,
    output_schema: &serde_json::Value,
) -> Tool {
    let input_obj = match input_schema.as_object() {
        Some(obj) => obj.clone(),
        None => serde_json::Map::new(),
    };
    let output_obj = match output_schema.as_object() {
        Some(obj) => obj.clone(),
        None => serde_json::Map::new(),
    };

    Tool {
        name: name.to_string().into(),
        title: Some(title.to_string()),
        description: Some(description.to_string().into()),
        input_schema: Arc::new(input_obj),
        output_schema: Some(Arc::new(output_obj)),
        annotations: None,
        icons: None,
    }
}
