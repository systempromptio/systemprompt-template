use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use systemprompt_core_ai::AiService;
use systemprompt_core_database::DbPool;
use systemprompt_core_system::RequestContext;
use systemprompt_identifiers::AgentName;
use systemprompt_models::McpTool;

use super::traits::{ExecutionIdLookup, ToolProvider};

#[derive(Debug)]
pub struct AiServiceToolProvider {
    ai_service: Arc<AiService>,
}

impl AiServiceToolProvider {
    pub const fn new(ai_service: Arc<AiService>) -> Self {
        Self { ai_service }
    }
}

#[async_trait]
impl ToolProvider for AiServiceToolProvider {
    async fn list_available_tools_for_agent(
        &self,
        agent_name: &AgentName,
        context: &RequestContext,
    ) -> Result<Vec<McpTool>> {
        self.ai_service
            .list_available_tools_for_agent(agent_name, context)
            .await
    }
}

#[derive(Debug)]
pub struct DatabaseExecutionIdLookup {
    db_pool: DbPool,
}

impl DatabaseExecutionIdLookup {
    pub const fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl ExecutionIdLookup for DatabaseExecutionIdLookup {
    async fn get_mcp_execution_id(&self, ai_tool_call_id: &str) -> Result<Option<String>> {
        use systemprompt_core_mcp::repository::ToolUsageRepository;

        let tool_usage_repo = ToolUsageRepository::new(self.db_pool.clone());
        match tool_usage_repo.get_by_ai_call_id(ai_tool_call_id).await {
            Ok(Some(exec_id)) => Ok(Some(exec_id)),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
