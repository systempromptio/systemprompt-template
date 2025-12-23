use rmcp::{
    model::{CallToolRequestParam, CallToolResult, Content},
    service::RequestContext,
    ErrorData as McpError, RoleServer,
};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::models::artifacts::{ExecutionMetadata, ToolResponse};

use super::map_credential_error;
use crate::sync::{SyncDirection, SyncService, SyncTarget};

#[derive(Debug, Deserialize)]
struct SyncInput {
    target: String,
    direction: String,
    #[serde(default)]
    dry_run: bool,
    #[serde(default)]
    filter: Option<String>,
}

#[must_use]
pub fn sync_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "target": {
                "type": "string",
                "enum": ["files", "database", "content", "skills", "all"],
                "description": "What to sync: files (service configs), database (agents/skills/contexts), content (blog/legal), skills (skill definitions), or all"
            },
            "direction": {
                "type": "string",
                "enum": ["push", "pull"],
                "description": "Sync direction: 'push' uploads local to cloud, 'pull' downloads cloud to local"
            },
            "dry_run": {
                "type": "boolean",
                "default": false,
                "description": "Preview changes without applying them"
            },
            "filter": {
                "type": "string",
                "description": "Optional filter (table names for database, source for content, skill_id for skills)"
            }
        },
        "required": ["target", "direction"]
    })
}

#[must_use]
pub fn sync_output_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "artifact_id": { "type": "string" },
            "execution_id": { "type": "string" },
            "data": {
                "type": "object",
                "description": "Sync result containing direction, dry_run status, and summary"
            },
            "metadata": { "type": "object" }
        }
    })
}

fn serialize_result<T: serde::Serialize>(result: &T) -> Result<JsonValue, McpError> {
    serde_json::to_value(result)
        .map_err(|e| McpError::internal_error(format!("Failed to serialize result: {e}"), None))
}

const fn dry_run_suffix(dry_run: bool) -> &'static str {
    if dry_run {
        "(dry run)"
    } else {
        ""
    }
}

const fn check_mark(condition: bool) -> &'static str {
    if condition {
        "✓"
    } else {
        "✗"
    }
}

pub async fn handle_sync(
    sync_service: &Arc<SyncService>,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();
    let input: SyncInput = serde_json::from_value(serde_json::Value::Object(args))
        .map_err(|e| McpError::invalid_params(format!("Invalid input parameters: {e}"), None))?;

    let target: SyncTarget = input.target.parse().map_err(|e: anyhow::Error| {
        McpError::invalid_params(format!("Invalid target: {e}"), None)
    })?;

    let direction: SyncDirection = input.direction.parse().map_err(|e: anyhow::Error| {
        McpError::invalid_params(format!("Invalid direction: {e}"), None)
    })?;

    tracing::info!(
        target = %input.target,
        direction = %direction,
        dry_run = input.dry_run,
        filter = ?input.filter,
        "Running sync"
    );

    let (summary_text, result_json) = execute_sync(sync_service, target, direction, input).await?;

    let metadata = ExecutionMetadata::new().tool("sync");
    let artifact_id = uuid::Uuid::new_v4().to_string();
    let tool_response = ToolResponse::new(
        &artifact_id,
        mcp_execution_id.clone(),
        result_json,
        metadata.clone(),
    );

    Ok(CallToolResult {
        content: vec![Content::text(summary_text)],
        structured_content: Some(tool_response.to_json()),
        is_error: Some(false),
        meta: metadata.to_meta(),
    })
}

async fn execute_sync(
    sync_service: &Arc<SyncService>,
    target: SyncTarget,
    direction: SyncDirection,
    input: SyncInput,
) -> Result<(String, JsonValue), McpError> {
    match target {
        SyncTarget::Files => {
            let result = sync_service
                .sync_files(direction, input.dry_run)
                .await
                .map_err(map_credential_error)?;
            let s = result.summary;
            let summary = format!(
                "Files sync {} complete: {} files ({} created, {} updated, {} deleted)",
                dry_run_suffix(input.dry_run),
                s.total_files,
                s.created,
                s.updated,
                s.deleted
            );
            Ok((summary, serialize_result(&result)?))
        }
        SyncTarget::Database => {
            let result = sync_service
                .sync_database(direction, input.dry_run, None)
                .await
                .map_err(map_credential_error)?;
            let summary = format!(
                "Database sync {} complete: {} tables, {} records",
                dry_run_suffix(input.dry_run),
                result.summary.total_tables,
                result.summary.total_records_synced
            );
            Ok((summary, serialize_result(&result)?))
        }
        SyncTarget::Content => {
            let result = sync_service
                .sync_content(direction, input.dry_run, input.filter)
                .await
                .map_err(map_credential_error)?;
            let summary = format!(
                "Content sync {} complete: {} items",
                dry_run_suffix(input.dry_run),
                result.summary.total_files
            );
            Ok((summary, serialize_result(&result)?))
        }
        SyncTarget::Skills => {
            let result = sync_service
                .sync_skills(direction, input.dry_run, input.filter)
                .await
                .map_err(map_credential_error)?;
            let summary = format!(
                "Skills sync {} complete: {} items",
                dry_run_suffix(input.dry_run),
                result.summary.total_files
            );
            Ok((summary, serialize_result(&result)?))
        }
        SyncTarget::All => {
            let result = sync_service
                .sync_all(direction, input.dry_run)
                .await
                .map_err(map_credential_error)?;
            let summary = format!(
                "Full sync {} complete in {}ms (files: {}, database: {}, deploy: {})",
                dry_run_suffix(input.dry_run),
                result.total_duration_ms,
                check_mark(result.files_result.is_some()),
                check_mark(result.database_result.is_some()),
                if result.deploy_result.is_some() {
                    "✓"
                } else {
                    "-"
                }
            );
            Ok((summary, serialize_result(&result)?))
        }
    }
}
