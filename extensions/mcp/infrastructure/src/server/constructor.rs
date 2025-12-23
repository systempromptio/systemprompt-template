use std::collections::HashMap;
use std::sync::Arc;

use systemprompt_core_database::DbPool;
use systemprompt_core_system::AppContext;
use systemprompt_identifiers::McpServerId;

use crate::config::SyncConfig;
use crate::prompts::InfrastructurePrompts;
use crate::resources::InfrastructureResources;
use crate::sync::SyncService;

#[derive(Clone)]
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
            db_pool.clone(),
            service_id.to_string(),
        ));
        let resources = Arc::new(InfrastructureResources::new(
            db_pool.clone(),
            service_id.to_string(),
        ));

        let sync_config = SyncConfig::load().unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to load sync config, using defaults");
            SyncConfig {
                tenant_id: String::new(),
                api_url: "https://api.systemprompt.io".to_string(),
                api_token: String::new(),
                services_path: "services".to_string(),
                database_url: None,
            }
        });
        let sync_service = Arc::new(SyncService::new(sync_config));

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
