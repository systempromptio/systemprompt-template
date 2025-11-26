use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_system::RequestContext;
use systemprompt_identifiers::AgentName;

use super::client::McpClientManager;
use crate::models::tools::McpTool;

#[derive(Debug)]
pub struct ToolDiscovery {
    client_manager: Arc<McpClientManager>,
}

impl ToolDiscovery {
    pub const fn new(client_manager: Arc<McpClientManager>) -> Self {
        Self { client_manager }
    }

    pub async fn discover_tools(
        &self,
        agent_name: &AgentName,
        context: &RequestContext,
    ) -> Result<Vec<McpTool>> {
        self.client_manager
            .refresh_connections_for_agent(agent_name)
            .await?;
        self.client_manager
            .list_tools_for_agent(agent_name, context)
            .await
    }

    pub async fn find_tool_for_agent(
        &self,
        agent_name: &AgentName,
        tool_name: &str,
        context: &RequestContext,
    ) -> Result<Option<McpTool>> {
        let tools = self
            .client_manager
            .list_tools_for_agent(agent_name, context)
            .await?;
        Ok(tools.into_iter().find(|tool| tool.name == tool_name))
    }
}
