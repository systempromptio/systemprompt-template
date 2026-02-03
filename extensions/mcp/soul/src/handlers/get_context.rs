use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;
use systemprompt::identifiers::{ArtifactId, McpExecutionId};
use systemprompt::models::artifacts::ToolResponse;
use systemprompt::models::execution::context::RequestContext;
use systemprompt::models::ExecutionMetadata;
use systemprompt_soul_extension::{MemoryService, MemoryType};

use crate::server::SoulMcpServer;
use crate::tools::GetContextInput;

impl SoulMcpServer {
    pub async fn handle_get_context(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        service: &MemoryService,
        ctx: &RequestContext,
        execution_id: &McpExecutionId,
    ) -> Result<CallToolResult, McpError> {
        let input: GetContextInput =
            serde_json::from_value(serde_json::Value::Object(arguments))
                .map_err(|e| McpError::invalid_params(format!("Invalid input: {e}"), None))?;

        let input_memory_types = input.memory_types.clone();
        let input_subject = input.subject.clone();

        let memory_types: Option<Vec<MemoryType>> = input_memory_types
            .as_ref()
            .map(|types| types.iter().filter_map(|t| t.parse().ok()).collect());

        let context_string = service
            .build_context_string(
                memory_types.as_deref(),
                input_subject.as_deref(),
                Some(input.max_items),
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to get context: {e}"), None))?;

        let metadata = ExecutionMetadata::with_request(ctx).with_tool("memory_get_context");

        let artifact = serde_json::json!({
            "artifact_type": "text",
            "title": "Memory Context",
            "data": {
                "context": context_string,
                "max_items": input.max_items,
                "memory_types": input_memory_types,
                "subject": input_subject
            }
        });

        let response = ToolResponse::new(
            ArtifactId::generate(),
            execution_id.clone(),
            artifact,
            metadata.clone(),
        );

        Ok(CallToolResult {
            content: vec![Content::text(context_string)],
            is_error: Some(false),
            meta: metadata.to_meta(),
            structured_content: response.to_json().ok(),
        })
    }
}
