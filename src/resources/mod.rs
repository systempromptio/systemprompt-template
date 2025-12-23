use anyhow::Result;
use rmcp::{
    model::{
        ListResourcesResult, PaginatedRequestParam, ReadResourceRequestParam, ReadResourceResult,
    },
    service::RequestContext,
    ErrorData as McpError, RoleServer,
};
use systemprompt_core_database::DbPool;

#[derive(Clone)]
pub struct InfrastructureResources {
    _db_pool: DbPool,
    _server_name: String,
}

impl InfrastructureResources {
    #[must_use]
    pub fn new(db_pool: DbPool, server_name: String) -> Self {
        Self {
            _db_pool: db_pool,
            _server_name: server_name,
        }
    }

    pub async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        // Infrastructure server doesn't expose resources directly
        // Resources are managed through the sync tools
        Ok(ListResourcesResult {
            next_cursor: None,
            resources: Vec::new(),
        })
    }

    pub async fn read_resource(
        &self,
        _request: ReadResourceRequestParam,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        Err(McpError::invalid_params(
            "Infrastructure server manages resources through sync tools. Use sync_status to check current state.".to_string(),
            None,
        ))
    }
}
