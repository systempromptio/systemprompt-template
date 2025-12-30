use rmcp::{
    model::{CallToolRequestParam, CallToolResult, Content},
    service::RequestContext,
    ErrorData as McpError, RoleServer,
};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use systemprompt::credentials::CredentialsBootstrap;
use systemprompt::identifiers::{ArtifactId, McpExecutionId};
use systemprompt::models::artifacts::{ExecutionMetadata, ToolResponse};

use crate::sync::{SyncService, SyncStatusResult};

const VERSION_NOT_AVAILABLE: &str = "n/a";

#[must_use]
pub fn status_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {},
        "description": "No input required - returns current cloud status"
    })
}

#[must_use]
pub fn status_output_schema() -> JsonValue {
    ToolResponse::<SyncStatusResult>::schema()
}

pub async fn handle_status(
    sync_service: &Arc<SyncService>,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let _ = request;

    tracing::info!("Getting status");

    let result = sync_service
        .get_status()
        .await
        .map_err(|e| McpError::internal_error(format!("Status check failed: {e}"), None))?;

    let auth_status = match CredentialsBootstrap::get() {
        Ok(Some(creds)) => {
            if creds.is_token_expired() {
                "expired (run 'systemprompt cloud login')"
            } else {
                "authenticated"
            }
        }
        Ok(None) => "not authenticated (run 'systemprompt cloud login')",
        Err(_) => "not initialized",
    };

    let connected = if result.cloud_status.connected {
        "connected"
    } else {
        "disconnected"
    };
    let db_status = if result.database_configured {
        "configured"
    } else {
        "not configured"
    };

    let tenant_display = if result.tenant_id.is_empty() {
        "not configured".to_string()
    } else {
        result.tenant_id.clone()
    };

    let version_display = result
        .cloud_status
        .app_version
        .as_deref()
        .unwrap_or(VERSION_NOT_AVAILABLE);

    let summary_text = format!(
        "Auth: {} | Cloud: {} | Tenant: {} | Database: {} | Version: {}",
        auth_status, connected, tenant_display, db_status, version_display
    );

    let metadata = ExecutionMetadata::new().tool("status");
    let artifact_id = ArtifactId::new(uuid::Uuid::new_v4().to_string());
    let tool_response = ToolResponse::new(
        artifact_id.clone(),
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
