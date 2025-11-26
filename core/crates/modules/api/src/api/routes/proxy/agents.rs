use crate::services::proxy::ProxyEngine;
use axum::{extract::Path, routing::any, Router};
use systemprompt_core_system::{AppContext, ServiceCategory};

pub fn router(ctx: &AppContext) -> Router {
    let engine = ProxyEngine::new();
    let engine_with_path = engine.clone();

    Router::new()
        // Handle POST requests to base agent URL (JSON-RPC endpoint)
        .route(
            "/{service_name}",
            any(move |Path(service_name): Path<String>, state, request| {
                let engine = engine.clone();
                async move {
                    engine
                        .handle_agent_request(Path((service_name,)), state, request)
                        .await
                }
            }),
        )
        // Handle GET requests with paths (e.g., /.well-known/agent-card.json)
        .route(
            "/{service_name}/{*path}",
            any(
                move |Path((service_name, path)): Path<(String, String)>, state, request| {
                    let engine = engine_with_path.clone();
                    async move {
                        engine
                            .handle_agent_request_with_path(
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
    "agents",
    ServiceCategory::Agent,
    router,
    false, // Agents handle their own authentication based on individual security settings
    systemprompt_core_system::models::modules::ModuleType::Proxy
);
