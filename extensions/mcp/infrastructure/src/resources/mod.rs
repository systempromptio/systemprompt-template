use anyhow::Result;
use rmcp::{
    model::{
        ListResourcesResult, PaginatedRequestParam, ReadResourceRequestParam, ReadResourceResult,
    },
    service::RequestContext,
    ErrorData as McpError, RoleServer,
};
use systemprompt::database::DbPool;

#[derive(Clone, Debug)]
pub struct InfrastructureResources {
    _db_pool: DbPool,
    _server_name: String,
}

impl InfrastructureResources {
    #[must_use]
    pub const fn new(db_pool: DbPool, server_name: String) -> Self {
        Self {
            _db_pool: db_pool,
            _server_name: server_name,
        }
    }

    #[allow(clippy::unused_async)]
    pub async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            next_cursor: None,
            resources: Vec::new(),
        })
    }

    #[allow(clippy::unused_async)]
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
