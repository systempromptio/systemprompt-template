use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Router};
use std::sync::Arc;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::api::ApiError;
use systemprompt_core_system::{AppContext, CollectionResponse, RequestContext};
use systemprompt_models::repository::ServiceRepository;

use crate::services::external_integrations::McpToolLoader;
use crate::services::registry::AgentRegistry;

pub async fn handle_agent_registry(
    Extension(req_ctx): Extension<RequestContext>,
    State(ctx): State<AppContext>,
) -> impl IntoResponse {
    let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());
    let registry = match AgentRegistry::new().await {
        Ok(r) => Arc::new(r),
        Err(e) => {
            logger
                .error(
                    "agent_api",
                    &format!("Failed to load agent registry: {e}"),
                )
                .await
                .ok();
            return ApiError::internal_error(format!("Failed to load agent registry: {e}"))
                .into_response();
        },
    };
    let service_repo = ServiceRepository::new(ctx.db_pool().clone());
    let tool_loader = McpToolLoader::new(ctx.db_pool().clone());
    let api_external_url = &ctx.config().api_external_url;

    match registry.list_agents().await {
        Ok(agents) => {
            let mut agent_cards = Vec::new();

            for agent_config in agents {
                let runtime_status =
                    match service_repo.get_service_by_name(&agent_config.name).await {
                        Ok(Some(service)) => Some((
                            service.status,
                            Some(agent_config.port),
                            service.pid.map(|p| p as u32),
                        )),
                        Ok(None) => Some(("NotStarted".to_string(), Some(agent_config.port), None)),
                        Err(_) => Some(("Unknown".to_string(), Some(agent_config.port), None)),
                    };

                // Create context with the correct agent name
                let agent_ctx =
                    req_ctx
                        .clone()
                        .with_agent_name(systemprompt_identifiers::AgentName::new(
                            agent_config.name.clone(),
                        ));

                let mcp_extensions = tool_loader
                    .create_mcp_extensions(
                        &agent_config.metadata.mcp_servers,
                        api_external_url,
                        &agent_ctx,
                    )
                    .await
                    .unwrap_or_default();

                match registry
                    .to_agent_card(
                        &agent_config.name,
                        api_external_url,
                        mcp_extensions,
                        runtime_status,
                    )
                    .await
                {
                    Ok(card) => {
                        agent_cards.push(card);
                    },
                    Err(e) => {
                        logger
                            .error(
                                "agent_api",
                                &format!("Failed to convert agent to card: {e}"),
                            )
                            .await
                            .ok();
                    },
                }
            }

            agent_cards.sort_by(|a, b| {
                let a_is_default = a
                    .capabilities
                    .extensions
                    .as_ref()
                    .and_then(|exts| exts.iter().find(|e| e.uri == "systemprompt:service-status"))
                    .and_then(|ext| ext.params.as_ref())
                    .and_then(|p| p.get("default"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let b_is_default = b
                    .capabilities
                    .extensions
                    .as_ref()
                    .and_then(|exts| exts.iter().find(|e| e.uri == "systemprompt:service-status"))
                    .and_then(|ext| ext.params.as_ref())
                    .and_then(|p| p.get("default"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                b_is_default.cmp(&a_is_default)
            });

            CollectionResponse::new(agent_cards).into_response()
        },
        Err(e) => {
            logger
                .error("agent_api", &format!("Failed to list agents: {e}"))
                .await
                .ok();
            ApiError::internal_error(format!("Failed to retrieve agent registry: {e}"))
                .into_response()
        },
    }
}

pub fn router(ctx: &AppContext) -> Router {
    Router::new()
        .route("/", get(handle_agent_registry))
        .with_state(ctx.clone())
}
