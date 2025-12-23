mod deploy_crate;
mod sync_all;
mod sync_database;
mod sync_files;
mod sync_status;

use rmcp::{
    model::{CallToolRequestMethod, CallToolRequestParam, CallToolResult, ListToolsResult, Tool},
    service::RequestContext,
    ErrorData as McpError, RoleServer,
};
use std::sync::Arc;
use systemprompt_identifiers::McpExecutionId;

use crate::sync::SyncService;

pub use deploy_crate::{deploy_crate_input_schema, deploy_crate_output_schema, handle_deploy_crate};
pub use sync_all::{handle_sync_all, sync_all_input_schema, sync_all_output_schema};
pub use sync_database::{
    handle_sync_database, sync_database_input_schema, sync_database_output_schema,
};
pub use sync_files::{handle_sync_files, sync_files_input_schema, sync_files_output_schema};
pub use sync_status::{handle_sync_status, sync_status_input_schema, sync_status_output_schema};

#[must_use]
pub fn register_tools() -> Vec<Tool> {
    vec![
        create_tool(
            "sync_files",
            "Sync Files",
            "Sync service configuration files (agents, skills, content, web configs) between local environment and cloud. Use 'push' to upload local changes, 'pull' to download cloud state.",
            sync_files_input_schema(),
            sync_files_output_schema(),
        ),
        create_tool(
            "sync_database",
            "Sync Database",
            "Sync database records (agents, skills, contexts) between local database and cloud. Use 'push' to upload local data, 'pull' to download cloud data.",
            sync_database_input_schema(),
            sync_database_output_schema(),
        ),
        create_tool(
            "deploy_crate",
            "Deploy Crate",
            "Build and deploy the application to cloud. This builds the Rust binary, web assets, creates a Docker image, and deploys to Fly.io.",
            deploy_crate_input_schema(),
            deploy_crate_output_schema(),
        ),
        create_tool(
            "sync_all",
            "Sync All",
            "Sync everything: files, database, and deploy crate (if pushing). This is the complete deployment workflow.",
            sync_all_input_schema(),
            sync_all_output_schema(),
        ),
        create_tool(
            "sync_status",
            "Sync Status",
            "Get current sync status and cloud deployment information.",
            sync_status_input_schema(),
            sync_status_output_schema(),
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
        "sync_files" => {
            handle_sync_files(sync_service, request, ctx, mcp_execution_id).await
        }
        "sync_database" => {
            handle_sync_database(sync_service, request, ctx, mcp_execution_id).await
        }
        "deploy_crate" => {
            handle_deploy_crate(sync_service, request, ctx, mcp_execution_id).await
        }
        "sync_all" => {
            handle_sync_all(sync_service, request, ctx, mcp_execution_id).await
        }
        "sync_status" => {
            handle_sync_status(sync_service, request, ctx, mcp_execution_id).await
        }
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

fn create_tool(
    name: &str,
    title: &str,
    description: &str,
    input_schema: serde_json::Value,
    output_schema: serde_json::Value,
) -> Tool {
    let input_obj = input_schema.as_object().cloned().unwrap_or_default();
    let output_obj = output_schema.as_object().cloned().unwrap_or_default();

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
