use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;
use systemprompt::identifiers::{ArtifactId, McpExecutionId};
use systemprompt::models::artifacts::ToolResponse;
use systemprompt::models::execution::context::RequestContext;
use systemprompt::models::ExecutionMetadata;
use systemprompt_soul_extension::MemoryService;

use crate::server::SoulMcpServer;
use crate::tools::ForgetInput;

impl SoulMcpServer {
    pub async fn handle_forget(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        service: &MemoryService,
        ctx: &RequestContext,
        execution_id: &McpExecutionId,
    ) -> Result<CallToolResult, McpError> {
        let input: ForgetInput = serde_json::from_value(serde_json::Value::Object(arguments))
            .map_err(|e| McpError::invalid_params(format!("Invalid input: {e}"), None))?;

        let forgotten = service
            .forget(&input.id)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to forget: {e}"), None))?;

        let metadata = ExecutionMetadata::with_request(ctx).with_tool("memory_forget");

        if forgotten {
            let artifact = serde_json::json!({
                "artifact_type": "text",
                "title": "Memory Forgotten",
                "data": {
                    "id": input.id,
                    "status": "forgotten"
                }
            });

            let response = ToolResponse::new(
                ArtifactId::generate(),
                execution_id.clone(),
                artifact,
                metadata.clone(),
            );

            Ok(CallToolResult {
                content: vec![Content::text(format!(
                    "Memory '{}' has been forgotten.",
                    input.id
                ))],
                is_error: Some(false),
                meta: metadata.to_meta(),
                structured_content: response.to_json().ok(),
            })
        } else {
            let artifact = serde_json::json!({
                "artifact_type": "text",
                "title": "Memory Not Found",
                "data": {
                    "id": input.id,
                    "status": "not_found"
                }
            });

            let response = ToolResponse::new(
                ArtifactId::generate(),
                execution_id.clone(),
                artifact,
                metadata.clone(),
            );

            Ok(CallToolResult {
                content: vec![Content::text(format!(
                    "Memory '{}' was not found or already forgotten.",
                    input.id
                ))],
                is_error: Some(true),
                meta: metadata.to_meta(),
                structured_content: response.to_json().ok(),
            })
        }
    }
}
