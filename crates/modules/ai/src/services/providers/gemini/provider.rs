use anyhow::Result;
use reqwest::Client;
use std::sync::{Arc, Mutex};
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;

use crate::services::schema::ToolNameMapper;

use super::client;
use super::constants::defaults;

#[derive(Debug)]
pub struct GeminiProvider {
    pub(crate) client: Client,
    pub(crate) api_key: String,
    pub(crate) endpoint: String,
    pub(crate) tool_mapper: Arc<Mutex<ToolNameMapper>>,
    pub(crate) db_pool: Option<DbPool>,
    pub(crate) google_search_enabled: bool,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Result<Self> {
        let client = client::build_client()?;
        Ok(Self {
            client,
            api_key,
            endpoint: defaults::ENDPOINT.to_string(),
            tool_mapper: Arc::new(Mutex::new(ToolNameMapper::new())),
            db_pool: None,
            google_search_enabled: false,
        })
    }

    pub fn with_endpoint(api_key: String, endpoint: String) -> Result<Self> {
        let client = client::build_client()?;
        Ok(Self {
            client,
            api_key,
            endpoint,
            tool_mapper: Arc::new(Mutex::new(ToolNameMapper::new())),
            db_pool: None,
            google_search_enabled: false,
        })
    }

    pub fn with_db_pool(mut self, db_pool: DbPool) -> Self {
        self.db_pool = Some(db_pool);
        self
    }

    pub const fn with_google_search(mut self) -> Self {
        self.google_search_enabled = true;
        self
    }

    pub const fn has_google_search(&self) -> bool {
        self.google_search_enabled
    }

    pub(crate) fn logger(&self) -> Option<LogService> {
        self.db_pool
            .as_ref()
            .map(|pool| LogService::system(pool.clone()))
    }

    pub async fn generate_with_code_execution(
        &self,
        messages: &[crate::models::ai::AiMessage],
        metadata: &crate::models::ai::SamplingMetadata,
        model: &str,
    ) -> Result<super::code_execution::CodeExecutionResponse> {
        super::code_execution::generate_with_code_execution(self, messages, metadata, model).await
    }

    pub async fn generate_with_tools_forced(
        &self,
        messages: &[crate::models::ai::AiMessage],
        tools: Vec<crate::models::tools::McpTool>,
        metadata: &crate::models::ai::SamplingMetadata,
        model: &str,
        allowed_function_names: Option<Vec<String>>,
    ) -> Result<(crate::models::ai::AiResponse, Vec<crate::models::tools::ToolCall>)> {
        super::tools::generate_with_tools_forced(
            self,
            messages,
            tools,
            metadata,
            model,
            allowed_function_names,
            None,
        )
        .await
    }

    pub async fn generate_with_tools_forced_with_config(
        &self,
        messages: &[crate::models::ai::AiMessage],
        tools: Vec<crate::models::tools::McpTool>,
        metadata: &crate::models::ai::SamplingMetadata,
        model: &str,
        allowed_function_names: Option<Vec<String>>,
        max_output_tokens: Option<u32>,
    ) -> Result<(crate::models::ai::AiResponse, Vec<crate::models::tools::ToolCall>)> {
        super::tools::generate_with_tools_forced(
            self,
            messages,
            tools,
            metadata,
            model,
            allowed_function_names,
            max_output_tokens,
        )
        .await
    }
}
