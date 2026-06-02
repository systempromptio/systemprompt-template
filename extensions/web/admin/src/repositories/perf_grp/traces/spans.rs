//! Per-session waterfall spans, unioned and normalised across the governance,
//! gateway-request, and tool-event tables.

use sqlx::PgPool;

use super::{Span, SpanKind, SpanStatus};

/// Resolve `id` (a `session_id` or `trace_id`) to an absolute `session_id`.
pub async fn resolve_trace_session(pool: &PgPool, id: &str) -> Result<Option<String>, sqlx::Error> {
    if let Some(row) = sqlx::query!(
        r#"SELECT session_id AS "session_id!"
           FROM ai_requests
           WHERE trace_id = $1 AND session_id IS NOT NULL
           LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(row.session_id));
    }

    if let Some(row) = sqlx::query!(
        r#"SELECT session_id AS "session_id!"
           FROM governance_decisions
           WHERE session_id = $1
           LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(row.session_id));
    }

    if let Some(row) = sqlx::query!(
        r#"SELECT session_id AS "session_id!"
           FROM ai_requests
           WHERE session_id = $1
           LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(row.session_id));
    }

    Ok(None)
}

async fn fetch_governance_spans(pool: &PgPool, session_id: &str) -> Result<Vec<Span>, sqlx::Error> {
    let decisions = sqlx::query!(
        r#"SELECT
            id              AS "id!",
            tool_name       AS "tool_name!",
            policy          AS "policy!",
            decision        AS "decision!",
            agent_id,
            agent_scope,
            user_id         AS "user_id!",
            created_at      AS "created_at!"
        FROM governance_decisions
        WHERE session_id = $1
           OR session_id IN (
               SELECT DISTINCT trace_id FROM ai_requests
               WHERE session_id = $1 AND trace_id IS NOT NULL)
        ORDER BY created_at ASC"#,
        session_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(decisions
        .into_iter()
        .map(|d| {
            let started = d.created_at;
            let status = if d.decision == "deny" {
                SpanStatus::Deny
            } else {
                SpanStatus::Ok
            };
            Span {
                id: d.id.clone(),
                kind: SpanKind::Governance,
                name: format!("{} / {}", d.policy, d.tool_name),
                started_at: started,
                ended_at: started,
                duration_ms: 0,
                status,
                identity_label: Some(format_identity(
                    Some(d.user_id.as_str()),
                    d.agent_id.as_deref(),
                    d.agent_scope.as_deref(),
                )),
                raw: serde_json::json!({
                    "policy": d.policy,
                    "decision": d.decision,
                    "tool_name": d.tool_name,
                    "agent_id": d.agent_id,
                }),
            }
        })
        .collect())
}

async fn fetch_request_spans(pool: &PgPool, session_id: &str) -> Result<Vec<Span>, sqlx::Error> {
    let requests = sqlx::query!(
        r#"SELECT
            id              AS "id!",
            request_id      AS "request_id!",
            provider        AS "provider!",
            model           AS "model!",
            status          AS "status!",
            latency_ms,
            created_at      AS "created_at!",
            completed_at,
            user_id         AS "user_id!"
        FROM ai_requests
        WHERE session_id = $1
        ORDER BY created_at ASC"#,
        session_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(requests
        .into_iter()
        .map(|r| {
            let started = r.created_at;
            let ended = r.completed_at.unwrap_or_else(|| {
                started
                    + chrono::Duration::milliseconds(i64::from(r.latency_ms.unwrap_or(0)).max(0))
            });
            let dur = (ended - started).num_milliseconds().max(0);
            let status = match r.status.as_str() {
                "ok" | "success" | "completed" => SpanStatus::Ok,
                "pending" => SpanStatus::Pending,
                _ => SpanStatus::Error,
            };
            Span {
                id: r.id.clone(),
                kind: SpanKind::Model,
                name: format!("{}/{}", r.provider, r.model),
                started_at: started,
                ended_at: ended,
                duration_ms: dur,
                status,
                identity_label: Some(format_identity(Some(r.user_id.as_str()), None, None)),
                raw: serde_json::json!({
                    "request_id": r.request_id,
                    "provider": r.provider,
                    "model": r.model,
                    "status": r.status,
                    "latency_ms": r.latency_ms,
                }),
            }
        })
        .collect())
}

async fn fetch_event_spans(pool: &PgPool, session_id: &str) -> Result<Vec<Span>, sqlx::Error> {
    let events = sqlx::query!(
        r#"SELECT
            id              AS "id!",
            event_type      AS "event_type!",
            tool_name,
            user_id         AS "user_id!",
            created_at      AS "created_at!"
        FROM plugin_usage_events
        WHERE session_id = $1
        ORDER BY created_at ASC"#,
        session_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(events
        .into_iter()
        .map(|e| {
            let started = e.created_at;
            let kind = if e.event_type.contains("Spawn") {
                SpanKind::Spawn
            } else {
                SpanKind::Tool
            };
            let name = e.tool_name.clone().unwrap_or_else(|| e.event_type.clone());
            let status = if e.event_type.contains("Failure") || e.event_type.contains("Error") {
                SpanStatus::Error
            } else {
                SpanStatus::Ok
            };
            Span {
                id: e.id.clone(),
                kind,
                name,
                started_at: started,
                ended_at: started,
                duration_ms: 0,
                status,
                identity_label: Some(format_identity(Some(e.user_id.as_str()), None, None)),
                raw: serde_json::json!({
                    "event_type": e.event_type,
                    "tool_name": e.tool_name,
                }),
            }
        })
        .collect())
}

pub async fn fetch_trace_spans(pool: &PgPool, session_id: &str) -> Result<Vec<Span>, sqlx::Error> {
    let mut spans = fetch_governance_spans(pool, session_id).await?;
    spans.extend(fetch_request_spans(pool, session_id).await?);
    spans.extend(fetch_event_spans(pool, session_id).await?);
    spans.sort_by_key(|s| s.started_at);
    Ok(spans)
}

fn format_identity(user: Option<&str>, agent: Option<&str>, scope: Option<&str>) -> String {
    let user_part = user.unwrap_or("?");
    match (agent, scope) {
        (Some(a), Some(s)) => format!("{user_part} · {a} ({s})"),
        (Some(a), None) => format!("{user_part} · {a}"),
        _ => user_part.to_string(),
    }
}
