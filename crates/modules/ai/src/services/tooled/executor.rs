use serde_json::json;
use std::sync::Arc;

use crate::models::tools::{CallToolResult, McpTool, ToolCall};
use crate::services::mcp::McpClientManager;
use systemprompt_models::ai::ToolModelOverrides;

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
        agent_overrides: Option<&ToolModelOverrides>,
    ) -> (Vec<ToolCall>, Vec<CallToolResult>) {
        let default_overrides = ToolModelOverrides::new();
        let overrides = agent_overrides.unwrap_or(&default_overrides);
        let mut tool_results = Vec::new();

        for tool_call in &tool_calls {
            let tool = tools.iter().find(|t| t.name == tool_call.name);

            if let Some(tool) = tool {
                let resolved_config = resolve_model_config(tool, overrides);
                let enriched_ctx = if let Some(config) = resolved_config {
                    context.clone().with_tool_model_config(config)
                } else {
                    context.clone()
                };

                match self
                    .client_manager
                    .execute_tool(tool_call, &tool.service_id, &enriched_ctx)
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

fn resolve_model_config(
    tool: &McpTool,
    agent_overrides: &ToolModelOverrides,
) -> Option<systemprompt_models::ai::ToolModelConfig> {
    if let Some(server_overrides) = agent_overrides.get(&tool.service_id) {
        if let Some(tool_override) = server_overrides.get(&tool.name) {
            return Some(tool_override.clone());
        }
    }
    tool.model_config.clone()
}
