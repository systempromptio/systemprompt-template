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

use crate::repositories::analytics_grp::agents::{list_agent_traces, AgentTraceRow};

pub async fn agent_traces_page(
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

    let traces = if agent_id_param.is_empty() {
        Vec::new()
    } else {
        fetch_agent_traces(&pool, &agent_id_param).await
    };

    let traces_json: Vec<serde_json::Value> = traces
        .iter()
        .map(|t| {
            json!({
                "session_id": t.session_id,
                "session_id_short": &t.session_id[..t.session_id.len().min(12)],
                "tool_name": t.tool_name,
                "decision": t.decision,
                "is_denied": t.decision == "deny",
                "policy": t.policy,
                "reason": t.reason,
                "created_at": t.created_at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string(),
            })
        })
        .collect();

    let cli_command = if agent_id_param.is_empty() {
        "systemprompt infra logs trace list".to_string()
    } else {
        format!("systemprompt infra logs trace list --agent {agent_id_param}")
    };

    let data = json!({
        "page": "agent-traces",
        "title": "Agent Traces",
        "active_tab": "traces",
        "agent_id": agent_id_param,
        "has_agent": agent.is_some(),
        "agent": agent,
        "traces": traces_json,
        "has_traces": !traces_json.is_empty(),
        "cli_command": cli_command,
    });
    super::render_page(&engine, "agent-traces", &data, &user_ctx, &mkt_ctx)
}

async fn fetch_agent_traces(pool: &PgPool, agent_id: &str) -> Vec<AgentTraceRow> {
    list_agent_traces(pool, agent_id).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, agent_id = %agent_id, "Failed to fetch agent traces");
        vec![]
    })
}
