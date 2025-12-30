use rmcp::{
    model::{CallToolRequestParam, CallToolResult, Content},
    service::RequestContext,
    ErrorData as McpError, RoleServer,
};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use systemprompt::identifiers::{ArtifactId, McpExecutionId};
use systemprompt::models::artifacts::{ExecutionMetadata, ToolResponse};

use crate::sync::{ExportTarget, SyncDirection, SyncService};

#[derive(Debug, Deserialize)]
struct ExportInput {
    target: String,
    #[serde(default)]
    output_dir: Option<String>,
    #[serde(default)]
    filter: Option<String>,
}

#[must_use]
pub fn export_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "target": {
                "type": "string",
                "enum": ["content", "skills", "all"],
                "description": "What to export: content (blog/legal), skills, or all"
            },
            "output_dir": {
                "type": "string",
                "description": "Optional output directory (defaults to services/)"
            },
            "filter": {
                "type": "string",
                "description": "Optional filter (source for content, skill_id for skills)"
            }
        },
        "required": ["target"]
    })
}

#[must_use]
pub fn export_output_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "artifact_id": { "type": "string" },
            "execution_id": { "type": "string" },
            "data": {
                "type": "object",
                "description": "Export result containing exported items"
            },
            "metadata": { "type": "object" }
        }
    })
}

fn serialize_result<T: serde::Serialize>(result: &T) -> Result<JsonValue, McpError> {
    serde_json::to_value(result)
        .map_err(|e| McpError::internal_error(format!("Failed to serialize result: {e}"), None))
}

pub async fn handle_export(
    sync_service: &Arc<SyncService>,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();
    let input: ExportInput = serde_json::from_value(serde_json::Value::Object(args))
        .map_err(|e| McpError::invalid_params(format!("Invalid input parameters: {e}"), None))?;

    let target: ExportTarget = input.target.parse().map_err(|e: anyhow::Error| {
        McpError::invalid_params(format!("Invalid target: {e}"), None)
    })?;

    tracing::info!(
        target = %input.target,
        output_dir = ?input.output_dir,
        filter = ?input.filter,
        "Running export"
    );

    let direction = SyncDirection::Push;

    let (summary_text, result_json) = match target {
        ExportTarget::Content => {
            let result = sync_service
                .sync_content(direction, false, input.filter)
                .await
                .map_err(|e| McpError::internal_error(format!("Export failed: {e}"), None))?;
            let summary = format!(
                "Content export complete: {} items exported",
                result.summary.total_files
            );
            (summary, serialize_result(&result)?)
        }
        ExportTarget::Skills => {
            let result = sync_service
                .sync_skills(direction, false, input.filter)
                .await
                .map_err(|e| McpError::internal_error(format!("Export failed: {e}"), None))?;
            let summary = format!(
                "Skills export complete: {} items exported",
                result.summary.total_files
            );
            (summary, serialize_result(&result)?)
        }
        ExportTarget::All => {
            let content_result = sync_service.sync_content(direction, false, None).await.ok();
            let skills_result = sync_service.sync_skills(direction, false, None).await.ok();

            let content_count = content_result.as_ref().map_or(0, |r| r.summary.total_files);
            let skills_count = skills_result.as_ref().map_or(0, |r| r.summary.total_files);

            let summary = format!(
                "Full export complete: {} content items, {} skills exported",
                content_count, skills_count
            );

            let result = json!({
                "content": content_result,
                "skills": skills_result,
                "total_items": content_count + skills_count
            });

            (summary, result)
        }
    };

    let metadata = ExecutionMetadata::new().tool("export");
    let artifact_id = ArtifactId::new(uuid::Uuid::new_v4().to_string());
    let tool_response = ToolResponse::new(
        artifact_id.clone(),
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
