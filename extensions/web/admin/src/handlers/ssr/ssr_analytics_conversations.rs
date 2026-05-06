//! Page 4 — Conversations & Transcripts.
//!
//! Two-pane SSR:
//!   - Left:  filtered list of `plugin_session_summaries` rows.
//!   - Right: turn-by-turn transcript for the session selected via
//!     `?session_id=…`, with PII redacted by default.
//!
//! Raw (un-redacted) bodies are never serialized into the page DOM unless the
//! viewer holds `transcript:view_pii`. When they do, the toggle on the page
//! fetches `/admin/api/conversations/:session_id/raw` on demand.

use std::sync::Arc;

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

use crate::auth::can_view_raw_transcript;
use crate::repositories::analytics_grp::{
    fetch_conversation_detail, fetch_conversation_list, ConversationDetail,
    ConversationListFilter, ConversationListItem, TranscriptTurn,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

#[derive(Debug, Deserialize, Default)]
pub struct ConversationsQuery {
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub plugin_id: Option<String>,
    pub q: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub range: Option<String>,
}

pub async fn analytics_conversations_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<ConversationsQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let can_view_pii = can_view_raw_transcript(&user_ctx);
    let (since, until, range_label) = resolve_time_range(query.since.as_deref(), query.until.as_deref(), query.range.as_deref());

    let filter = ConversationListFilter {
        user_id: empty_to_none(query.user_id.clone()),
        plugin_id: empty_to_none(query.plugin_id.clone()),
        free_text: empty_to_none(query.q.clone()),
        since,
        until,
        limit: 100,
    };

    let sessions = fetch_conversation_list(&pool, &filter)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "fetch_conversation_list failed");
            vec![]
        });

    let sessions_json: Vec<serde_json::Value> = sessions
        .iter()
        .map(|s| build_session_item_json(s, query.session_id.as_deref()))
        .collect();

    let detail = if let Some(sid) = query.session_id.as_deref().filter(|s| !s.is_empty()) {
        fetch_conversation_detail(&pool, sid, can_view_pii)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, session_id = %sid, "fetch_conversation_detail failed");
                None
            })
    } else {
        None
    };

    let detail_json = detail.as_ref().map(build_detail_json);

    let data = json!({
        "page": "analytics-conversations",
        "title": "Conversations & Transcripts",
        "cli_command": "systemprompt analytics conversations list",
        "window_label": range_label,
        "sessions": sessions_json,
        "has_sessions": !sessions_json.is_empty(),
        "selected_session_id": query.session_id,
        "detail": detail_json,
        "filters": {
            "user_id": query.user_id,
            "plugin_id": query.plugin_id,
            "q": query.q,
            "range": query.range,
        },
        "can_view_pii": can_view_pii,
        "redaction_mode": !can_view_pii, // when capability missing, server enforces redaction
    });

    super::render_page(
        &engine,
        "analytics-conversations",
        &data,
        &user_ctx,
        &mkt_ctx,
    )
}

fn empty_to_none(v: Option<String>) -> Option<String> {
    v.filter(|s| !s.trim().is_empty())
}

fn build_session_item_json(
    s: &ConversationListItem,
    selected_session_id: Option<&str>,
) -> serde_json::Value {
    let is_selected = selected_session_id == Some(s.session_id.as_str());
    json!({
        "session_id": s.session_id,
        "user_id": s.user_id,
        "plugin_id": s.plugin_id,
        "model": s.model,
        "status": s.status,
        "ai_title": s.ai_title.clone().unwrap_or_else(|| "(untitled session)".to_string()),
        "started_at": s.started_at.map(|t| t.to_rfc3339()),
        "total_input_tokens": s.total_input_tokens,
        "total_output_tokens": s.total_output_tokens,
        "governance_intervention_count": s.governance_intervention_count,
        "deny_count": s.deny_count,
        "is_selected": is_selected,
    })
}

fn build_turn_json(t: &TranscriptTurn) -> serde_json::Value {
    let chain_target = t
        .governance
        .as_ref()
        .and_then(|g| g.trace_id.clone())
        .unwrap_or_else(|| t.session_id.clone());
    let chain_url = format!("#chain:{chain_target}");
    let tool_calls: Vec<serde_json::Value> = t
        .tool_calls
        .iter()
        .map(|tc| {
            json!({
                "id": tc.id,
                "name": tc.name,
                "args_json": tc.args_json,
                "result_json": tc.result_json,
                "duration_ms": tc.duration_ms,
                "status": tc.status,
            })
        })
        .collect();
    let governance = t.governance.as_ref().map(|g| {
        json!({
            "decision": g.decision,
            "trace_id": g.trace_id,
            "rule_count": g.rule_count,
            "redactions_applied": g.redactions_applied,
        })
    });
    json!({
        "id": t.id,
        "session_id": t.session_id,
        "ordinal": t.ordinal,
        "role": t.role,
        "ts": t.ts.map(|v| v.to_rfc3339()),
        "model": t.model,
        "latency_ms": t.latency_ms,
        "content_redacted": t.content_redacted,
        "content": t.content,
        "redactions_applied": t.redactions_applied,
        "tool_calls": tool_calls,
        "tool_calls_len": t.tool_calls.len(),
        "governance": governance,
        "chain_url": chain_url,
        "anomaly_count": t.anomaly_count,
    })
}

fn build_detail_json(d: &ConversationDetail) -> serde_json::Value {
    let turns_json: Vec<serde_json::Value> = d.turns.iter().map(build_turn_json).collect();
    json!({
        "session_id": d.session_id,
        "user_id": d.user_id,
        "plugin_id": d.plugin_id,
        "ai_title": d.ai_title,
        "ai_summary": d.ai_summary,
        "model": d.model,
        "started_at": d.started_at.map(|t| t.to_rfc3339()),
        "ended_at": d.ended_at.map(|t| t.to_rfc3339()),
        "turns": turns_json,
        "turn_count": d.turns.len(),
    })
}

/// Resolve `(since, until)`. `range` is one of `1h`, `24h`, `7d`, `30d`, `all`,
/// or empty (defaults to 7d). Explicit `since`/`until` win.
fn resolve_time_range(
    since: Option<&str>,
    until: Option<&str>,
    range: Option<&str>,
) -> (Option<DateTime<Utc>>, Option<DateTime<Utc>>, String) {
    if since.is_some() || until.is_some() {
        let parsed_since = since
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&Utc));
        let parsed_until = until
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&Utc));
        return (parsed_since, parsed_until, "custom".to_string());
    }

    let now = Utc::now();
    match range.unwrap_or("7d") {
        "1h"  => (Some(now - Duration::hours(1)),  Some(now), "last hour".to_string()),
        "24h" => (Some(now - Duration::hours(24)), Some(now), "last 24 hours".to_string()),
        "30d" => (Some(now - Duration::days(30)),  Some(now), "last 30 days".to_string()),
        "all" => (None, None, "all time".to_string()),
        _     => (Some(now - Duration::days(7)),   Some(now), "last 7 days".to_string()),
    }
}
