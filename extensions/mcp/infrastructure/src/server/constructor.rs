use std::collections::HashMap;
use std::sync::Arc;

use systemprompt::database::DbPool;
use systemprompt::identifiers::McpServerId;
use systemprompt::system::AppContext;

use crate::prompts::InfrastructurePrompts;
use crate::resources::InfrastructureResources;
use crate::sync::SyncService;

#[derive(Clone, Debug)]
pub struct InfrastructureServer {
    pub(super) db_pool: DbPool,
    pub(super) service_id: McpServerId,
    pub(super) prompts: Arc<InfrastructurePrompts>,
    pub(super) resources: Arc<InfrastructureResources>,
    pub(super) sync_service: Arc<SyncService>,
    pub(super) tool_schemas: Arc<HashMap<String, serde_json::Value>>,
    #[allow(dead_code)]
    pub(super) app_context: Arc<AppContext>,
}

impl InfrastructureServer {
    #[must_use]
    pub fn new(db_pool: DbPool, service_id: McpServerId, app_context: Arc<AppContext>) -> Self {
        let prompts = Arc::new(InfrastructurePrompts::new(
            DbPool::clone(&db_pool),
            service_id.to_string(),
        ));
        let resources = Arc::new(InfrastructureResources::new(
            DbPool::clone(&db_pool),
            service_id.to_string(),
        ));

        let sync_service = Arc::new(SyncService::new(DbPool::clone(&db_pool)));
        let tool_schemas = Self::build_tool_schema_cache();

        Self {
            db_pool,
            service_id,
            prompts,
            resources,
            sync_service,
            tool_schemas: Arc::new(tool_schemas),
            app_context,
        }
    }

    fn build_tool_schema_cache() -> HashMap<String, serde_json::Value> {
        let mut schemas = HashMap::new();
        let tools = crate::tools::register_tools();

        for tool in tools {
            if let Some(output_schema) = tool.output_schema {
                if let Ok(schema_value) = serde_json::to_value(&*output_schema) {
                    schemas.insert(tool.name.to_string(), schema_value);
                }
            }
        }

        schemas
    }

    pub(super) fn get_output_schema_for_tool(&self, tool_name: &str) -> Option<serde_json::Value> {
        self.tool_schemas.get(tool_name).cloned()
    }
}
