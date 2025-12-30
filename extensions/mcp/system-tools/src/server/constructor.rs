use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use systemprompt::agent::services::SkillService;
use systemprompt::ai::{AiService, NoopToolProvider};
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpServerId;
use systemprompt::models::ai::ToolModelConfig;
use systemprompt::models::services::AiConfig;
use systemprompt::system::AppContext;

#[derive(Clone)]
#[allow(dead_code)]
pub struct SystemToolsServer {
    pub(super) db_pool: DbPool,
    pub(super) service_id: McpServerId,
    pub(super) tool_schemas: Arc<HashMap<String, serde_json::Value>>,
    pub(super) app_context: Arc<AppContext>,
    pub(super) file_roots: Arc<Vec<PathBuf>>,
    pub(crate) ai_service: Arc<AiService>,
    pub(crate) ai_config: Arc<AiConfig>,
    pub(crate) skill_service: Arc<SkillService>,
}

impl SystemToolsServer {
    pub fn new(
        db_pool: DbPool,
        service_id: McpServerId,
        app_context: Arc<AppContext>,
    ) -> Result<Self> {
        let tool_schemas = Self::build_tool_schema_cache();
        let file_roots = Self::init_file_roots();

        let ai_config = Arc::new(Self::load_ai_config()?);
        let tool_provider = Arc::new(NoopToolProvider::new());
        let ai_service = Arc::new(
            AiService::new(&app_context, &ai_config, tool_provider)
                .context("Failed to initialize AiService")?,
        );

        let skill_service = Arc::new(SkillService::new(db_pool.clone()));

        Ok(Self {
            db_pool,
            service_id,
            tool_schemas: Arc::new(tool_schemas),
            app_context,
            file_roots: Arc::new(file_roots),
            ai_service,
            ai_config,
            skill_service,
        })
    }

    fn load_ai_config() -> Result<AiConfig> {
        let config_path =
            env::var("AI_CONFIG_PATH").context("AI_CONFIG_PATH environment variable must be set")?;
        let path = std::path::Path::new(&config_path);
        let content = std::fs::read_to_string(path)
            .context("Failed to read AI config file")?;
        serde_yaml::from_str(&content)
            .context("Failed to parse AI config YAML")
    }

    /// Get model configuration for tool execution.
    /// Uses the default provider and model from AI config.
    pub fn get_default_model_config(&self) -> Result<ToolModelConfig> {
        let default_provider = &self.ai_config.default_provider;
        let provider_config = self
            .ai_config
            .providers
            .get(default_provider)
            .ok_or_else(|| anyhow::anyhow!("AI config missing provider '{}'", default_provider))?;

        let default_max_tokens = self.ai_config.default_max_output_tokens.ok_or_else(|| {
            anyhow::anyhow!("AI config missing default_max_output_tokens")
        })?;

        Ok(
            ToolModelConfig::new(default_provider, &provider_config.default_model)
                .with_max_output_tokens(default_max_tokens),
        )
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
