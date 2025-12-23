use rmcp::{
    model::{CallToolRequestParam, CallToolResult, Content},
    service::RequestContext,
    ErrorData as McpError, RoleServer,
};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use systemprompt_identifiers::McpExecutionId;
use systemprompt_models::artifacts::{ExecutionMetadata, ToolResponse};

use crate::sync::{SyncService, SyncStatusResult};

#[must_use]
pub fn sync_status_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {}
    })
}

#[must_use]
pub fn sync_status_output_schema() -> JsonValue {
    ToolResponse::<SyncStatusResult>::schema()
}

pub async fn handle_sync_status(
    sync_service: &Arc<SyncService>,
    _request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    tracing::info!("Getting sync status");

    let result = sync_service
        .get_status()
        .await
        .map_err(|e| McpError::internal_error(format!("Failed to get status: {e}"), None))?;

    let connection_status = if result.cloud_status.connected {
        "connected"
    } else {
        "disconnected"
    };

    let db_status = if result.database_configured {
        "configured"
    } else {
        "not configured"
    };

    let summary_text = format!(
        "Sync status: tenant={}, cloud={}, database={}",
        if result.tenant_id.is_empty() {
            "not set"
        } else {
            &result.tenant_id
        },
        connection_status,
        db_status
    );

    let metadata = ExecutionMetadata::new().tool("sync_status");
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
