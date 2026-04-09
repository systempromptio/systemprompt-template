use std::sync::Arc;

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;
use sqlx::{FromRow, PgPool};

use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

#[derive(Debug, Deserialize)]
pub struct TracesQuery {
    pub session_id: Option<String>,
}

#[derive(Debug, FromRow)]
struct TraceEvent {
    id: String,
    event_type: String,
    tool_name: Option<String>,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct TraceGovernanceRow {
    tool_name: String,
    agent_id: Option<String>,
    agent_scope: Option<String>,
    decision: String,
    policy: String,
    reason: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct TraceEntity {
    entity_type: String,
    entity_name: String,
    usage_count: i32,
}

#[derive(Debug, FromRow)]
struct SessionSummaryRow {
    total_events: i64,
    tool_uses: i64,
    prompts: i64,
    errors: i64,
}

pub(crate) async fn traces_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<TracesQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let session_id = query.session_id.as_deref().unwrap_or("");

    if session_id.is_empty() {
        let data = json!({
            "page": "traces",
            "title": "Trace Detail",
            "has_session": false,
            "session_id": "",
        });
        return super::render_page(&engine, "traces", &data, &user_ctx, &mkt_ctx);
    }

    let (events_result, governance_result, entities_result, summary_result) = tokio::join!(
        fetch_trace_events(&pool, session_id),
        fetch_trace_governance(&pool, session_id),
        fetch_trace_entities(&pool, session_id),
        fetch_session_summary(&pool, session_id),
    );

    let events = unwrap_or_warn(events_result, "trace events");
    let governance = unwrap_or_warn(governance_result, "trace governance");
    let entities = unwrap_or_warn(entities_result, "trace entities");
    let summary = summary_result.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch session summary");
        SessionSummaryRow {
            total_events: 0,
            tool_uses: 0,
            prompts: 0,
            errors: 0,
        }
    });

    let data = build_trace_data(session_id, &events, &governance, &entities, &summary);
    super::render_page(&engine, "traces", &data, &user_ctx, &mkt_ctx)
}

fn unwrap_or_warn<T>(result: Result<Vec<T>, sqlx::Error>, label: &str) -> Vec<T> {
    result.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch {}", label);
        vec![]
    })
}

fn build_trace_data(
    session_id: &str,
    events: &[TraceEvent],
    governance: &[TraceGovernanceRow],
    entities: &[TraceEntity],
    summary: &SessionSummaryRow,
) -> serde_json::Value {
    let events_json = build_events_json(events);
    let governance_json = build_governance_json(governance);
    let entities_json = build_entities_json(entities);

    let duration_ms = events
        .first()
        .zip(events.last())
        .map_or(0, |(first, last)| {
            (last.created_at - first.created_at).num_milliseconds()
        });

    json!({
        "page": "traces",
        "title": format!("Trace — {}", &session_id[..session_id.len().min(12)]),
        "has_session": true,
        "session_id": session_id,
        "summary": {
            "total_events": summary.total_events,
            "tool_uses": summary.tool_uses,
            "prompts": summary.prompts,
            "errors": summary.errors,
            "duration_ms": duration_ms,
            "governance_decisions": governance.len(),
        },
        "events": events_json,
        "has_events": !events_json.is_empty(),
        "governance": governance_json,
        "has_governance": !governance_json.is_empty(),
        "entities": entities_json,
        "has_entities": !entities_json.is_empty(),
    })
}

fn event_badge_class(event_type: &str) -> &'static str {
    match event_type {
        t if t.contains("ToolUse") => "mcp-badge-info",
        t if t.contains("Failure") || t.contains("Error") => "mcp-badge-danger",
        t if t.contains("Prompt") || t.contains("Submit") => "mcp-badge-success",
        t if t.contains("Session") => "mcp-badge-warning",
        t if t.contains("Subagent") => "badge-purple",
        _ => "mcp-badge-neutral",
    }
}

fn build_events_json(events: &[TraceEvent]) -> Vec<serde_json::Value> {
    let first_ts = events.first().map(|e| e.created_at);
    events
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let delta_ms = if i > 0 {
                (e.created_at - events[i - 1].created_at).num_milliseconds()
            } else {
                0
            };
            let elapsed_ms = first_ts.map_or(0, |f| (e.created_at - f).num_milliseconds());
            json!({
                "id": e.id,
                "event_type": e.event_type,
                "event_type_short": e.event_type.replace("claude_code_", ""),
                "tool_name": e.tool_name,
                "has_tool": e.tool_name.is_some(),
                "metadata": e.metadata,
                "has_metadata": e.metadata != serde_json::Value::Null && e.metadata != json!({}),
                "created_at": e.created_at.with_timezone(&chrono::Local).format("%H:%M:%S%.3f").to_string(),
                "delta_ms": delta_ms,
                "elapsed_ms": elapsed_ms,
                "badge_class": event_badge_class(&e.event_type),
            })
        })
        .collect()
}

fn build_governance_json(governance: &[TraceGovernanceRow]) -> Vec<serde_json::Value> {
    governance
        .iter()
        .map(|g| {
            json!({
                "tool_name": g.tool_name,
                "agent_id": g.agent_id,
                "agent_scope": g.agent_scope,
                "decision": g.decision,
                "is_denied": g.decision == "deny",
                "is_secret_breach": g.policy == "secret_injection",
                "policy": g.policy,
                "reason": g.reason,
                "created_at": g.created_at.with_timezone(&chrono::Local).format("%H:%M:%S%.3f").to_string(),
            })
        })
        .collect()
}

fn build_entities_json(entities: &[TraceEntity]) -> Vec<serde_json::Value> {
    entities
        .iter()
        .map(|e| {
            let badge_class = match e.entity_type.as_str() {
                "skill" => "badge-blue",
                "agent" => "badge-purple",
                "mcp_tool" => "mcp-badge-info",
                _ => "mcp-badge-neutral",
            };
            json!({
                "entity_type": e.entity_type,
                "entity_name": e.entity_name,
                "usage_count": e.usage_count,
                "badge_class": badge_class,
            })
        })
        .collect()
}

async fn fetch_trace_events(
    pool: &PgPool,
    session_id: &str,
) -> Result<Vec<TraceEvent>, sqlx::Error> {
    sqlx::query_as::<_, TraceEvent>(
        "SELECT id, event_type, tool_name, COALESCE(metadata, '{}'::jsonb) AS metadata, created_at \
         FROM plugin_usage_events \
         WHERE session_id = $1 \
         ORDER BY created_at ASC \
         LIMIT 500",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
}

async fn fetch_trace_governance(
    pool: &PgPool,
    session_id: &str,
) -> Result<Vec<TraceGovernanceRow>, sqlx::Error> {
    sqlx::query_as::<_, TraceGovernanceRow>(
        "SELECT tool_name, agent_id, agent_scope, decision, policy, reason, created_at \
         FROM governance_decisions \
         WHERE session_id = $1 \
         ORDER BY created_at ASC \
         LIMIT 100",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
}

async fn fetch_trace_entities(
    pool: &PgPool,
    session_id: &str,
) -> Result<Vec<TraceEntity>, sqlx::Error> {
    sqlx::query_as::<_, TraceEntity>(
        "SELECT entity_type, entity_name, usage_count \
         FROM session_entity_links \
         WHERE session_id = $1 \
         ORDER BY usage_count DESC \
         LIMIT 50",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
}

async fn fetch_session_summary(
    pool: &PgPool,
    session_id: &str,
) -> Result<SessionSummaryRow, sqlx::Error> {
    sqlx::query_as::<_, SessionSummaryRow>(
        r"SELECT
            COUNT(*)::bigint AS total_events,
            COUNT(*) FILTER (WHERE event_type LIKE '%ToolUse%')::bigint AS tool_uses,
            COUNT(*) FILTER (WHERE event_type LIKE '%Prompt%' OR event_type LIKE '%Submit%')::bigint AS prompts,
            COUNT(*) FILTER (WHERE event_type LIKE '%Failure%' OR event_type LIKE '%Error%')::bigint AS errors
        FROM plugin_usage_events
        WHERE session_id = $1",
    )
    .bind(session_id)
    .fetch_one(pool)
    .await
}
