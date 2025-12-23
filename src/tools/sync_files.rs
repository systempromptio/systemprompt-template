use rmcp::{
    model::{CallToolRequestParam, CallToolResult, Content},
    service::RequestContext,
    ErrorData as McpError, RoleServer,
};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use systemprompt_identifiers::McpExecutionId;
use systemprompt_models::artifacts::{ExecutionMetadata, ToolResponse};

use crate::config::SyncDirection;
use crate::sync::{SyncFilesResult, SyncService};

#[derive(Debug, Deserialize)]
struct SyncFilesInput {
    direction: String,
    #[serde(default)]
    dry_run: bool,
}

#[must_use]
pub fn sync_files_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "direction": {
                "type": "string",
                "enum": ["push", "pull"],
                "description": "Sync direction: 'push' uploads local to cloud, 'pull' downloads cloud to local"
            },
            "dry_run": {
                "type": "boolean",
                "default": false,
                "description": "Preview changes without applying them"
            }
        },
        "required": ["direction"]
    })
}

#[must_use]
pub fn sync_files_output_schema() -> JsonValue {
    ToolResponse::<SyncFilesResult>::schema()
}

pub async fn handle_sync_files(
    sync_service: &Arc<SyncService>,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();
    let input: SyncFilesInput =
        serde_json::from_value(serde_json::Value::Object(args)).map_err(|e| {
            McpError::invalid_params(format!("Invalid input parameters: {e}"), None)
        })?;

    let direction: SyncDirection = input.direction.parse().map_err(|e: anyhow::Error| {
        McpError::invalid_params(format!("Invalid direction: {e}"), None)
    })?;

    tracing::info!(direction = %direction, dry_run = input.dry_run, "Syncing files");

    let result = sync_service
        .sync_files(direction, input.dry_run)
        .await
        .map_err(|e| McpError::internal_error(format!("Sync failed: {e}"), None))?;

    let summary_text = format!(
        "File sync {} complete: {} files processed ({} created, {} updated, {} deleted, {} unchanged)",
        if input.dry_run { "(dry run)" } else { "" },
        result.summary.total_files,
        result.summary.created,
        result.summary.updated,
        result.summary.deleted,
        result.summary.unchanged
    );

    let metadata = ExecutionMetadata::new().tool("sync_files");
    let artifact_id = uuid::Uuid::new_v4().to_string();
    let tool_response = ToolResponse::new(
        &artifact_id,
        mcp_execution_id.clone(),
        result,
        metadata.clone(),
    );

    Ok(CallToolResult {
        content: vec![Content::text(summary_text)],
        structured_content: Some(tool_response.to_json()),
        is_error: Some(false),
        meta: metadata.to_meta(),
    })
}
