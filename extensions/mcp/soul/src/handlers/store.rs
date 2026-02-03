use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;
use systemprompt::identifiers::{ArtifactId, McpExecutionId};
use systemprompt::models::artifacts::ToolResponse;
use systemprompt::models::execution::context::RequestContext;
use systemprompt::models::ExecutionMetadata;
use systemprompt_soul_extension::{CreateMemoryParams, MemoryCategory, MemoryService, MemoryType};

use crate::server::SoulMcpServer;
use crate::tools::StoreMemoryInput;

impl SoulMcpServer {
    #[allow(clippy::missing_panics_doc)]
    pub async fn handle_store(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        service: &MemoryService,
        ctx: &RequestContext,
        execution_id: &McpExecutionId,
    ) -> Result<CallToolResult, McpError> {
        let input: StoreMemoryInput = serde_json::from_value(serde_json::Value::Object(arguments))
            .map_err(|e| McpError::invalid_params(format!("Invalid input: {e}"), None))?;

        let memory_type: MemoryType = input
            .memory_type
            .parse()
            .map_err(|e: String| McpError::invalid_params(e, None))?;

        let category: MemoryCategory = input
            .category
            .parse()
            .map_err(|e: String| McpError::invalid_params(e, None))?;

        let mut params =
            CreateMemoryParams::new(memory_type, category, &input.subject, &input.content);

        if let Some(ctx_text) = input.context_text {
            params = params.with_context_text(ctx_text);
        }
        if let Some(priority) = input.priority {
            params = params.with_priority(priority);
        }
        if let Some(tags) = input.tags {
            params = params.with_tags(tags);
        }

        let memory = service
            .store(&params)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to store memory: {e}"), None))?;

        let metadata = ExecutionMetadata::with_request(ctx).with_tool("memory_store");

        let memory_data = serde_json::json!({
            "id": memory.id,
            "memory_type": memory.memory_type,
            "category": memory.category,
            "subject": memory.subject,
            "created_at": memory.created_at,
        });

        let artifact = serde_json::json!({
            "artifact_type": "text",
            "title": "Memory Stored",
            "data": memory_data
        });

        let response = ToolResponse::new(
            ArtifactId::generate(),
            execution_id.clone(),
            artifact,
            metadata.clone(),
        );

        Ok(CallToolResult {
            content: vec![Content::text(format!(
                "Memory stored successfully.\n\n{}",
                serde_json::to_string_pretty(&memory_data)
                    .expect("memory_data serialization cannot fail")
            ))],
            is_error: Some(false),
            meta: metadata.to_meta(),
            structured_content: response.to_json().ok(),
        })
    }
}
