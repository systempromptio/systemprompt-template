#[cfg(test)]
pub mod test_mocks {
    use anyhow::Result;
    use async_trait::async_trait;
    use serde_json::json;
    use std::sync::Arc;
    use std::sync::Mutex;
    use systemprompt_core_system::RequestContext;
    use systemprompt_identifiers::AgentName;
    use systemprompt_models::McpTool;

    use super::super::traits::{ExecutionIdLookup, ToolProvider};

    pub struct MockToolProvider {
        tools: Arc<Mutex<Vec<McpTool>>>,
    }

    impl MockToolProvider {
        pub fn new(tools: Vec<McpTool>) -> Self {
            Self {
                tools: Arc::new(Mutex::new(tools)),
            }
        }

        pub fn with_tool(tool_name: &str, service_id: &str) -> Self {
            Self::new(vec![McpTool {
                name: tool_name.to_string(),
                description: Some(format!("Mock tool: {}", tool_name)),
                input_schema: Some(json!({
                    "type": "object",
                    "properties": {}
                })),
                output_schema: Some(json!({
                    "type": "object",
                    "properties": {
                        "result": {"type": "string"}
                    }
                })),
                service_id: service_id.to_string(),
            }])
        }

        pub fn empty() -> Self {
            Self::new(Vec::new())
        }
    }

    #[async_trait]
    impl ToolProvider for MockToolProvider {
        async fn list_available_tools_for_agent(
            &self,
            _agent_name: &AgentName,
            _context: &RequestContext,
        ) -> Result<Vec<McpTool>> {
            Ok(self.tools.lock().unwrap().clone())
        }
    }

    pub struct MockExecutionIdLookup {
        executions: Arc<Mutex<std::collections::HashMap<String, String>>>,
    }

    impl MockExecutionIdLookup {
        pub fn new() -> Self {
            Self {
                executions: Arc::new(Mutex::new(std::collections::HashMap::new())),
            }
        }

        pub fn with_execution(ai_call_id: &str, execution_id: &str) -> Self {
            let mut map = std::collections::HashMap::new();
            map.insert(ai_call_id.to_string(), execution_id.to_string());
            Self {
                executions: Arc::new(Mutex::new(map)),
            }
        }
    }

    #[async_trait]
    impl ExecutionIdLookup for MockExecutionIdLookup {
        async fn get_mcp_execution_id(&self, ai_tool_call_id: &str) -> Result<Option<String>> {
            Ok(self
                .executions
                .lock()
                .unwrap()
                .get(ai_tool_call_id)
                .cloned())
        }
    }
}
