use std::sync::Arc;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;

struct AgentTraceRow {
    session_id: String,
    tool_name: String,
    decision: String,
    policy: String,
    reason: String,
    created_at: DateTime<Utc>,
}

/// Agent traces detail page — execution traces, artifact metadata, cost attribution.
/// Mirrors `demo/agents/04-agent-tracing.sh`.
pub async fn agent_traces_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let agent_id_param = params.get("id").cloned().unwrap_or_default();
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
    let result = sqlx::query!(
        r#"SELECT session_id, tool_name, decision, policy, reason, created_at
           FROM governance_decisions
           WHERE agent_id = $1
           ORDER BY created_at DESC
           LIMIT 100"#,
        agent_id,
    )
    .fetch_all(pool)
    .await;
    match result {
        Ok(rows) => rows
            .into_iter()
            .map(|r| AgentTraceRow {
                session_id: r.session_id,
                tool_name: r.tool_name,
                decision: r.decision,
                policy: r.policy,
                reason: r.reason,
                created_at: r.created_at,
            })
            .collect(),
        Err(e) => {
            tracing::warn!(error = %e, agent_id = %agent_id, "Failed to fetch agent traces");
            vec![]
        }
    }
}
