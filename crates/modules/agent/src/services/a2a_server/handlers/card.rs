use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use std::sync::Arc;
use systemprompt_core_system::Config;

use super::state::AgentHandlerState;
use crate::services::registry::AgentRegistry;

pub async fn handle_agent_card(State(state): State<Arc<AgentHandlerState>>) -> impl IntoResponse {
    let config = state.config.read().await;
    let agent_name = config.name.clone();
    drop(config);

    let log = state.log.clone();
    log.info(
        "a2a_card",
        &format!("Fetching agent card for: {agent_name}"),
    )
    .await
    .ok();

    let system_config = Config::global();
    let base_url = &system_config.api_external_url;

    match AgentRegistry::new().await {
        Ok(registry) => match registry.get_agent(&agent_name).await {
            Ok(agent_config) => {
                match registry
                    .to_agent_card(&agent_config.name, base_url, vec![], None)
                    .await
                {
                    Ok(agent_card) => (StatusCode::OK, Json(agent_card)).into_response(),
                    Err(e) => {
                        log.error("a2a_card", &format!("Failed to build agent card: {e}"))
                            .await
                            .ok();
                        let error_response = json!({
                            "error": "Internal server error",
                            "message": "Failed to build agent card"
                        });
                        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
                    },
                }
            },
            Err(_e) => {
                log.error("a2a_card", &format!("Agent card not found: {agent_name}"))
                    .await
                    .ok();
                let error_response = json!({
                    "error": "Agent card not found",
                    "message": format!("No agent card available for agent: {agent_name}")
                });
                (StatusCode::NOT_FOUND, Json(error_response)).into_response()
            },
        },
        Err(e) => {
            log.error("a2a_card", &format!("Failed to initialize registry: {e}"))
                .await
                .ok();
            let error_response = json!({
                "error": "Internal server error",
                "message": "Failed to initialize agent registry"
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        },
    }
}
