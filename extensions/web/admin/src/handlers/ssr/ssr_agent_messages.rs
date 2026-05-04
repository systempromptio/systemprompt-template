use std::sync::Arc;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{IdQuery, MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

use crate::repositories::analytics_grp::agents::{list_agent_messages, AgentMessageRow};

pub async fn agent_messages_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<IdQuery>,
) -> Response {
    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let agent_id_param = params.id().unwrap_or_default().to_owned();
    let agent = if agent_id_param.is_empty() {
        None
    } else {
        repositories::find_agent(&services_path, &agent_id_param)
            .ok()
            .flatten()
    };

    let messages = if agent_id_param.is_empty() {
        Vec::new()
    } else {
        fetch_agent_messages(&pool, &agent_id_param).await
    };

    let messages_json: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| {
            json!({
                "id": m.id,
                "session_id": m.session_id,
                "event_type": m.event_type,
                "tool_name": m.tool_name,
                "has_tool": m.tool_name.is_some(),
                "metadata": m.metadata,
                "created_at": m.created_at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string(),
            })
        })
        .collect();

    let cli_command = if agent_id_param.is_empty() {
        "systemprompt admin agents message <id> -m \"...\"".to_string()
    } else {
        format!("systemprompt admin agents message {agent_id_param} -m \"...\"")
    };

    let data = json!({
        "page": "agent-messages",
        "title": "Agent Messages",
        "active_tab": "messages",
        "agent_id": agent_id_param,
        "has_agent": agent.is_some(),
        "agent": agent,
        "messages": messages_json,
        "has_messages": !messages_json.is_empty(),
        "cli_command": cli_command,
    });
    super::render_page(&engine, "agent-messages", &data, &user_ctx, &mkt_ctx)
}

async fn fetch_agent_messages(pool: &PgPool, agent_id: &str) -> Vec<AgentMessageRow> {
    list_agent_messages(pool, agent_id).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, agent_id = %agent_id, "Failed to fetch agent messages");
        vec![]
    })
}
