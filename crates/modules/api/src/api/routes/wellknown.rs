use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use systemprompt_core_agent::services::registry::AgentRegistry;
use systemprompt_core_system::AppContext;

pub fn wellknown_router(ctx: &AppContext) -> Router {
    Router::new()
        .route(
            "/.well-known/agent-card.json",
            get(handle_default_agent_card),
        )
        .route("/.well-known/agent-cards", get(handle_list_agent_cards))
        .route(
            "/.well-known/agent-cards/{agent_name}",
            get(handle_agent_card_by_name),
        )
        .with_state(ctx.clone())
}

async fn handle_default_agent_card(
    State(ctx): State<AppContext>,
) -> Result<impl IntoResponse, StatusCode> {
    let registry = AgentRegistry::new()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let default_agent = registry
        .get_default_agent()
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let base_url = &ctx.config().api_external_url;

    let agent_card = registry
        .to_agent_card(&default_agent.name, base_url, vec![], None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!(agent_card)))
}

async fn handle_agent_card_by_name(
    State(ctx): State<AppContext>,
    Path(agent_name): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let registry = AgentRegistry::new()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let agent_name = agent_name.trim_end_matches(".json");
    let _agent = registry
        .get_agent(agent_name)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let base_url = &ctx.config().api_external_url;

    let agent_card = registry
        .to_agent_card(agent_name, base_url, vec![], None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!(agent_card)))
}

async fn handle_list_agent_cards(
    State(ctx): State<AppContext>,
) -> Result<impl IntoResponse, StatusCode> {
    let registry = AgentRegistry::new()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let agents = registry
        .list_agents()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let base_url = &ctx.config().api_external_url;

    let mut cards = Vec::new();
    for agent in agents {
        if let Ok(card) = registry
            .to_agent_card(&agent.name, base_url, vec![], None)
            .await
        {
            cards.push(card);
        }
    }

    Ok(Json(json!(cards)))
}
