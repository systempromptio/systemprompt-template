use serde_json::json;
use std::sync::Arc;

use crate::models::tools::{CallToolResult, McpTool, ToolCall};
use crate::services::mcp::McpClientManager;

#[derive(Debug)]
pub struct TooledExecutor {
    client_manager: Arc<McpClientManager>,
}

impl TooledExecutor {
    pub const fn new(client_manager: Arc<McpClientManager>) -> Self {
        Self { client_manager }
    }

    pub async fn execute_tool_calls(
        &self,
        tool_calls: Vec<ToolCall>,
        tools: &[McpTool],
        context: &systemprompt_core_system::RequestContext,
    ) -> (Vec<ToolCall>, Vec<CallToolResult>) {
        let mut tool_results = Vec::new();

        for tool_call in &tool_calls {
            if tool_call.is_meta_tool() {
                continue;
            }

            let tool = tools.iter().find(|t| t.name == tool_call.name);

            if let Some(tool) = tool {
                match self
                    .client_manager
                    .execute_tool(tool_call, &tool.service_id, context)
                    .await
                {
                    Ok(result) => tool_results.push(result),
                    Err(e) => {
                        use rmcp::model::Content;
                        tool_results.push(CallToolResult {
                            content: vec![Content::text(format!("Error: {e}"))],
                            structured_content: Some(json!({"error": e.to_string()})),
                            is_error: Some(true),
                            meta: None,
                        });
                    },
                }
            } else {
                use rmcp::model::Content;
                tool_results.push(CallToolResult {
                    content: vec![Content::text(format!(
                        "Error: Tool '{}' not found in provided tools list",
                        tool_call.name
                    ))],
                    structured_content: Some(json!({
                        "error": format!("Tool '{}' not found", tool_call.name)
                    })),
                    is_error: Some(true),
                    meta: None,
                });
            }
        }

        (tool_calls, tool_results)
    }
}
