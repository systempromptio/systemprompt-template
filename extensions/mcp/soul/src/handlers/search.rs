use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;
use systemprompt::identifiers::{ArtifactId, McpExecutionId};
use systemprompt::models::artifacts::ToolResponse;
use systemprompt::models::execution::context::RequestContext;
use systemprompt::models::ExecutionMetadata;
use systemprompt_soul_extension::MemoryService;

use crate::server::SoulMcpServer;
use crate::tools::SearchInput;

impl SoulMcpServer {
    pub async fn handle_search(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        service: &MemoryService,
        ctx: &RequestContext,
        execution_id: &McpExecutionId,
    ) -> Result<CallToolResult, McpError> {
        let input: SearchInput = serde_json::from_value(serde_json::Value::Object(arguments))
            .map_err(|e| McpError::invalid_params(format!("Invalid input: {e}"), None))?;

        let memories = service
            .search(
                &input.query,
                input.memory_type.as_deref(),
                input.category.as_deref(),
                Some(input.limit),
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to search: {e}"), None))?;

        let metadata = ExecutionMetadata::with_request(ctx).with_tool("memory_search");

        if memories.is_empty() {
            let artifact = serde_json::json!({
                "artifact_type": "list",
                "title": "Memory Search Results",
                "data": {
                    "query": input.query,
                    "count": 0,
                    "memories": []
                }
            });

            let response = ToolResponse::new(
                ArtifactId::generate(),
                execution_id.clone(),
                artifact,
                metadata.clone(),
            );

            return Ok(CallToolResult {
                content: vec![Content::text("No memories found matching your query.")],
                is_error: Some(false),
                meta: metadata.to_meta(),
                structured_content: response.to_json().ok(),
            });
        }

        let text = memories
            .iter()
            .map(|m| {
                format!(
                    "- [{}] {} ({}): {}",
                    m.memory_type, m.subject, m.category, m.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let memory_data: Vec<serde_json::Value> = memories
            .iter()
            .map(|m| {
                serde_json::json!({
                    "id": m.id,
                    "memory_type": m.memory_type,
                    "category": m.category,
                    "subject": m.subject,
                    "content": m.content
                })
            })
            .collect();

        let artifact = serde_json::json!({
            "artifact_type": "list",
            "title": "Memory Search Results",
            "data": {
                "query": input.query,
                "count": memories.len(),
                "memories": memory_data
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
                "Found {} memories:\n\n{}",
                memories.len(),
                text
            ))],
            is_error: Some(false),
            meta: metadata.to_meta(),
            structured_content: response.to_json().ok(),
        })
    }
}
