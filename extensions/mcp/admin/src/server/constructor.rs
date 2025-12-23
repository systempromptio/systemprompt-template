use std::collections::HashMap;
use std::sync::Arc;

use systemprompt_core_agent::services::mcp::ToolResultHandler;
use systemprompt_core_agent::services::ArtifactPublishingService;
use systemprompt_core_database::DbPool;
use systemprompt_core_system::AppContext;
use systemprompt_identifiers::McpServerId;

use crate::prompts::AdminPrompts;
use crate::resources::AdminResources;

#[derive(Clone)]
pub struct AdminServer {
    pub(super) db_pool: DbPool,
    pub(super) service_id: McpServerId,
    pub(super) prompts: Arc<AdminPrompts>,
    pub(super) resources: Arc<AdminResources>,
    pub(super) tool_result_handler: Arc<ToolResultHandler>,
    pub(super) publishing_service: Arc<ArtifactPublishingService>,
    pub(super) tool_schemas: Arc<HashMap<String, serde_json::Value>>,
    pub(super) app_context: Arc<AppContext>,
}

impl AdminServer {
    #[must_use] pub fn new(db_pool: DbPool, service_id: McpServerId, app_context: Arc<AppContext>) -> Self {
        let prompts = Arc::new(AdminPrompts::new(db_pool.clone(), service_id.to_string()));
        let resources = Arc::new(AdminResources::new(db_pool.clone(), service_id.to_string()));
        let tool_result_handler = Arc::new(ToolResultHandler::new(db_pool.clone()));
        let publishing_service = Arc::new(ArtifactPublishingService::new(db_pool.clone()));

        let tool_schemas = Self::build_tool_schema_cache();

        Self {
            db_pool,
            service_id,
            prompts,
            resources,
            tool_result_handler,
            publishing_service,
            tool_schemas: Arc::new(tool_schemas),
            app_context,
        }
    }

    fn build_tool_schema_cache() -> HashMap<String, serde_json::Value> {
        let mut schemas = HashMap::new();
        let tools = crate::tools::register_tools();

        for tool in tools {
            if let Some(output_schema) = tool.output_schema {
                let schema_value =
                    serde_json::to_value(&*output_schema).unwrap_or_else(|_| serde_json::json!({}));
                schemas.insert(tool.name.to_string(), schema_value);
            }
        }

        schemas
    }

    pub(super) fn get_output_schema_for_tool(&self, tool_name: &str) -> Option<serde_json::Value> {
        self.tool_schemas.get(tool_name).cloned()
    }
}
