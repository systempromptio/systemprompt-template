use anyhow::Result;
use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer};
use systemprompt_core_database::DbPool;

#[derive(Debug, Clone)]
pub struct TemplatePrompts {
    _db_pool: DbPool,
    _server_name: String,
}

impl TemplatePrompts {
    pub fn new(db_pool: DbPool, server_name: String) -> Self {
        Self {
            _db_pool: db_pool,
            _server_name: server_name,
        }
    }

    pub async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            prompts: vec![],
            next_cursor: None,
        })
    }

    pub async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        match request.name.as_ref() {
            _ => Err(McpError::invalid_params(
                format!("Unknown prompt: {}", request.name),
                None,
            )),
        }
    }
}
