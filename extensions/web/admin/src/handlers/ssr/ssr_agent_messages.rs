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

struct AgentMessageRow {
    id: String,
    session_id: String,
    event_type: String,
    tool_name: Option<String>,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
}

/// Agent messages detail page — message thread, AI reasoning, MCP tool calls,
/// artifact creation. Mirrors `demo/agents/03-agent-messaging.sh`.
pub async fn agent_messages_page(
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
    let pattern = format!("%{agent_id}%");
    let result = sqlx::query!(
        r#"SELECT id, session_id, event_type, tool_name,
                  COALESCE(metadata, '{}'::jsonb) AS "metadata!", created_at
           FROM plugin_usage_events
           WHERE metadata->>'agent_id' = $1
              OR metadata->>'agent' = $1
              OR tool_name LIKE $2
           ORDER BY created_at DESC
           LIMIT 100"#,
        agent_id,
        &pattern,
    )
    .fetch_all(pool)
    .await;
    match result {
        Ok(rows) => rows
            .into_iter()
            .map(|r| AgentMessageRow {
                id: r.id,
                session_id: r.session_id,
                event_type: r.event_type,
                tool_name: r.tool_name,
                metadata: r.metadata,
                created_at: r.created_at,
            })
            .collect(),
        Err(e) => {
            tracing::warn!(error = %e, agent_id = %agent_id, "Failed to fetch agent messages");
            vec![]
        }
    }
}
