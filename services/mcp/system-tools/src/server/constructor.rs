use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::AppContext;
use systemprompt_identifiers::McpServerId;

#[derive(Clone)]
#[allow(dead_code)]
pub struct SystemToolsServer {
    pub(super) db_pool: DbPool,
    pub(super) service_id: McpServerId,
    pub(super) system_log: LogService,
    pub(super) tool_schemas: Arc<HashMap<String, serde_json::Value>>,
    pub(super) app_context: Arc<AppContext>,
    pub(super) file_roots: Arc<Vec<PathBuf>>,
}

impl SystemToolsServer {
    #[must_use]
    pub fn new(db_pool: DbPool, service_id: McpServerId, app_context: Arc<AppContext>) -> Self {
        let system_log = LogService::system(db_pool.clone());
        let tool_schemas = Self::build_tool_schema_cache();
        let file_roots = Self::init_file_roots();

        Self {
            db_pool,
            service_id,
            system_log,
            tool_schemas: Arc::new(tool_schemas),
            app_context,
            file_roots: Arc::new(file_roots),
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

    fn init_file_roots() -> Vec<PathBuf> {
        let roots: Vec<PathBuf> = match env::var("FILE_ROOT") {
            Ok(value) => value.split(',').map(PathBuf::from).collect(),
            Err(_) => env::current_dir()
                .map(|current_working_directory| vec![current_working_directory])
                .unwrap_or_default(),
        };

        roots
            .into_iter()
            .filter_map(|root| match root.canonicalize() {
                Ok(canonical) if canonical.is_dir() => Some(canonical),
                _ => None,
            })
            .collect()
    }

    pub fn validate_path(&self, path: &std::path::Path) -> Result<PathBuf, String> {
        let canonical = path
            .canonicalize()
            .map_err(|error| format!("Path does not exist: '{}' ({})", path.display(), error))?;

        if self.file_roots.is_empty() {
            return Ok(canonical);
        }

        for root in self.file_roots.iter() {
            if canonical.starts_with(root) {
                return Ok(canonical);
            }
        }

        Err(format!(
            "Access denied: '{}' is outside allowed roots",
            canonical.display()
        ))
    }

    pub fn validate_new_path(&self, path: &std::path::Path) -> Result<PathBuf, String> {
        let parent = path
            .parent()
            .ok_or_else(|| format!("Invalid path: '{}'", path.display()))?;

        let canonical_parent = parent.canonicalize().map_err(|error| {
            format!("Parent does not exist: '{}' ({})", parent.display(), error)
        })?;

        if self.file_roots.is_empty() {
            let filename = path.file_name().ok_or("Invalid filename")?;
            return Ok(canonical_parent.join(filename));
        }

        for root in self.file_roots.iter() {
            if canonical_parent.starts_with(root) {
                let filename = path.file_name().ok_or("Invalid filename")?;
                return Ok(canonical_parent.join(filename));
            }
        }

        Err(format!(
            "Access denied: '{}' is outside allowed roots",
            path.display()
        ))
    }

    #[must_use]
    pub fn get_roots(&self) -> &[PathBuf] {
        &self.file_roots
    }

    #[allow(dead_code)]
    pub(super) fn get_output_schema_for_tool(&self, tool_name: &str) -> Option<serde_json::Value> {
        self.tool_schemas.get(tool_name).cloned()
    }
}
