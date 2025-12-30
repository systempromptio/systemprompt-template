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

use super::map_credential_error;
use crate::sync::{DeployCrateResult, SyncService};

#[derive(Debug, Deserialize)]
struct DeployInput {
    #[serde(default)]
    skip_build: bool,
    tag: Option<String>,
}

#[must_use]
pub fn deploy_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "skip_build": {
                "type": "boolean",
                "default": false,
                "description": "Skip cargo build and web asset compilation (use existing)"
            },
            "tag": {
                "type": "string",
                "description": "Custom Docker image tag (default: deploy-{timestamp})"
            }
        }
    })
}

#[must_use]
pub fn deploy_output_schema() -> JsonValue {
    ToolResponse::<DeployCrateResult>::schema()
}

pub async fn handle_deploy(
    sync_service: &Arc<SyncService>,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();
    let input: DeployInput = serde_json::from_value(serde_json::Value::Object(args))
        .map_err(|e| McpError::invalid_params(format!("Invalid input parameters: {e}"), None))?;

    tracing::info!(skip_build = input.skip_build, tag = ?input.tag, "Deploying");

    let result = sync_service
        .deploy_crate(input.skip_build, input.tag)
        .await
        .map_err(map_credential_error)?;

    let status = if result.success {
        "successful"
    } else {
        "failed"
    };
    let url_text = result
        .deployment_url
        .as_ref()
        .map_or_else(String::new, |u| format!(" at {u}"));

    let summary_text = format!(
        "Deployment {}: image={}, {} steps completed{}",
        status,
        result.image_tag,
        result.steps_completed.len(),
        url_text
    );

    let is_error = !result.success;
    let metadata = ExecutionMetadata::new().tool("deploy");
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
        is_error: Some(is_error),
        meta: metadata.to_meta(),
    })
}
