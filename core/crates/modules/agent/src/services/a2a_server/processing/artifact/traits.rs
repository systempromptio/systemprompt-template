use anyhow::Result;
use async_trait::async_trait;
use systemprompt_core_system::RequestContext;
use systemprompt_identifiers::AgentName;
use systemprompt_models::McpTool;

#[async_trait]
pub trait ToolProvider: Send + Sync {
    async fn list_available_tools_for_agent(
        &self,
        agent_name: &AgentName,
        context: &RequestContext,
    ) -> Result<Vec<McpTool>>;
}

#[async_trait]
pub trait ExecutionIdLookup: Send + Sync {
    async fn get_mcp_execution_id(&self, ai_tool_call_id: &str) -> Result<Option<String>>;
}
