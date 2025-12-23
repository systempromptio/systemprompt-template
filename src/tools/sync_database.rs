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
use crate::sync::{SyncDatabaseResult, SyncService, SyncTable};

#[derive(Debug, Deserialize)]
struct SyncDatabaseInput {
    direction: String,
    #[serde(default)]
    dry_run: bool,
    tables: Option<Vec<String>>,
}

#[must_use]
pub fn sync_database_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "direction": {
                "type": "string",
                "enum": ["push", "pull"],
                "description": "Sync direction"
            },
            "dry_run": {
                "type": "boolean",
                "default": false,
                "description": "Preview changes without applying them"
            },
            "tables": {
                "type": "array",
                "items": {
                    "type": "string",
                    "enum": ["agents", "skills", "contexts"]
                },
                "description": "Specific tables to sync (default: all)"
            }
        },
        "required": ["direction"]
    })
}

#[must_use]
pub fn sync_database_output_schema() -> JsonValue {
    ToolResponse::<SyncDatabaseResult>::schema()
}

pub async fn handle_sync_database(
    sync_service: &Arc<SyncService>,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();
    let input: SyncDatabaseInput =
        serde_json::from_value(serde_json::Value::Object(args)).map_err(|e| {
            McpError::invalid_params(format!("Invalid input parameters: {e}"), None)
        })?;

    let direction: SyncDirection = input.direction.parse().map_err(|e: anyhow::Error| {
        McpError::invalid_params(format!("Invalid direction: {e}"), None)
    })?;

    let tables: Option<Vec<SyncTable>> = input.tables.map(|t| {
        t.iter()
            .filter_map(|s| s.parse().ok())
            .collect()
    });

    let tables_str = tables
        .as_ref()
        .map(|t| {
            t.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_else(|| "all".to_string());

    tracing::info!(direction = %direction, dry_run = input.dry_run, tables = %tables_str, "Syncing database");

    let result = sync_service
        .sync_database(direction, input.dry_run, tables)
        .await
        .map_err(|e| McpError::internal_error(format!("Database sync failed: {e}"), None))?;

    let summary_text = format!(
        "Database sync {} complete: {} tables, {} total records synced in {}ms",
        if input.dry_run { "(dry run)" } else { "" },
        result.summary.total_tables,
        result.summary.total_records_synced,
        result.summary.duration_ms
    );

    let metadata = ExecutionMetadata::new().tool("sync_database");
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
