use crate::services::proxy::ProxyEngine;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{any, get};
use axum::{Json, Router};
use serde::Serialize;
use systemprompt_core_logging::LogService;
use systemprompt_core_mcp::repository::ToolUsageRepository;
use systemprompt_core_system::api::ApiError;
use systemprompt_core_system::{AppContext, ServiceCategory};

#[derive(Debug, Serialize)]
pub struct ToolExecutionResponse {
    pub id: String,
    pub tool_name: String,
    pub mcp_server_name: String,
    pub server_endpoint: String,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub status: String,
}

pub async fn handle_get_execution(
    Path(execution_id): Path<String>,
    State(ctx): State<AppContext>,
) -> impl IntoResponse {
    let logger = LogService::system(ctx.db_pool().clone());
    let repo = ToolUsageRepository::new(ctx.db_pool().clone());

    logger
        .info("mcp_api", &format!("Fetching execution: {execution_id}"))
        .await
        .ok();

    match repo.get_by_id(&execution_id).await {
        Ok(Some(execution)) => {
            let server_endpoint = format!("/api/v1/mcp/{}/mcp", execution.mcp_server_name);

            let response = ToolExecutionResponse {
                id: execution.mcp_execution_id.clone(),
                tool_name: execution.tool_name,
                mcp_server_name: execution.mcp_server_name.clone(),
                server_endpoint,
                input: serde_json::from_str(&execution.input).unwrap_or_default(),
                output: execution.output.and_then(|s| serde_json::from_str(&s).ok()),
                status: execution.status,
            };

            logger
                .info("mcp_api", &format!("Execution found: {execution_id}"))
                .await
                .ok();
            Json(response).into_response()
        },
        Ok(None) => {
            ApiError::not_found(format!("Execution not found: {execution_id}")).into_response()
        },
        Err(e) => {
            logger
                .error(
                    "mcp_api",
                    &format!("Failed to get execution {execution_id}: {e}"),
                )
                .await
                .ok();
            ApiError::internal_error(format!("Failed to get execution: {e}")).into_response()
        },
    }
}

pub fn router(ctx: &AppContext) -> Router {
    let engine = ProxyEngine::new();
    Router::new()
        .route("/executions/{id}", get(handle_get_execution))
        .route(
            "/{service_name}/{*path}",
            any(
                move |Path((service_name, path)): Path<(String, String)>, state, request| {
                    let engine = engine.clone();
                    async move {
                        engine
                            .handle_mcp_request_with_path(
                                Path((service_name, path)),
                                state,
                                request,
                            )
                            .await
                    }
                },
            ),
        )
        .with_state(ctx.clone())
}

systemprompt_core_system::register_module_api!(
    "mcp",
    ServiceCategory::Mcp,
    router,
    true,
    systemprompt_core_system::models::modules::ModuleType::Proxy
);
