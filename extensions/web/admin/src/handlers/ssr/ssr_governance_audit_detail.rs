//! `/admin/governance/decisions/{id}` — single-request audit detail page.
//!
//! `id` may be an `ai_requests.id`, `request_id`, or `governance_decisions.id`.
//! Renders the full chain (identity, policy evaluations, prompt/response
//! preview, cost, latency, linked trace) using the existing
//! `fetch_decision_chain` envelope.

use std::sync::Arc;

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use crate::repositories::governance_grp::chain::{
    fetch_decision_chain, AiRequestSummary, ChainEnvelope,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

const NOT_FOUND_HTML: &str = "<h1>Request not found</h1>\
<p>No audit chain found for that id.</p>";

pub async fn governance_audit_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let envelope = match fetch_decision_chain(&pool, &id).await {
        Ok(Some(env)) => env,
        Ok(None) => return (StatusCode::NOT_FOUND, Html(NOT_FOUND_HTML)).into_response(),
        Err(e) => {
            tracing::error!(error = %e, id = %id, "fetch_decision_chain failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, Html(NOT_FOUND_HTML))
                .into_response();
        }
    };

    let primary = pick_primary(&envelope, &id);
    let title = primary.map_or_else(
        || format!("Request · {}", short_id(&envelope.session_id)),
        |r| format!("Request · {}", short_id(&r.id)),
    );

    let summary = build_summary(&envelope);
    let primary_json = primary.map(build_primary_json);
    let banner = build_banner(primary, &envelope);

    let data = json!({
        "page": "request-detail",
        "title": title,
        "summary": summary,
        "primary": primary_json,
        "banner": banner,
        "decisions": envelope.decisions,
        "requests": envelope.requests,
        "events": envelope.events,
        "transcript": envelope.transcript,
        "session_summary": envelope.summary,
        "back_url": "/admin/governance/decisions",
    });

    super::render_page(&engine, "governance-audit-detail", &data, &user_ctx, &mkt_ctx)
}

/// Pin the page to the request the user clicked when possible — fall back
/// to the first request in the session.
fn pick_primary<'a>(env: &'a ChainEnvelope, id: &str) -> Option<&'a AiRequestSummary> {
    env.requests
        .iter()
        .find(|r| r.id == id || r.request_id == id)
        .or_else(|| env.requests.first())
}

fn build_summary(env: &ChainEnvelope) -> serde_json::Value {
    let total_latency = env
        .requests
        .iter()
        .filter_map(|r| r.latency_ms)
        .map(i64::from)
        .sum::<i64>();
    let cost_usd = env.totals.total_cost_microdollars as f64 / 1_000_000.0;
    json!({
        "session_id": env.session_id,
        "session_id_short": short_id(&env.session_id),
        "trace_id": env.trace_id,
        "trace_url": env.trace_id.as_ref().map(|tid| {
            format!("/admin/traces/{}", urlencoding::encode(tid))
        }),
        "session_url": format!("/admin/sessions/{}", urlencoding::encode(&env.session_id)),
        "user_id": env.identity.user_id,
        "agent_id": env.identity.agent_id,
        "agent_scope": env.identity.agent_scope,
        "decision_count": env.totals.decision_count,
        "deny_count": env.totals.deny_count,
        "request_count": env.totals.request_count,
        "input_tokens": env.totals.total_input_tokens,
        "output_tokens": env.totals.total_output_tokens,
        "cost_usd": format!("{cost_usd:.4}"),
        "cost_microdollars": env.totals.total_cost_microdollars,
        "latency_ms": total_latency,
    })
}

fn is_failed_status(status: &str) -> bool {
    matches!(status, "failed" | "error" | "denied")
}

fn build_primary_json(r: &AiRequestSummary) -> serde_json::Value {
    json!({
        "id": r.id,
        "request_id": r.request_id,
        "model": r.model,
        "provider": r.provider,
        "status": r.status,
        "is_failed": is_failed_status(&r.status),
        "error_message": r.error_message,
        "input_tokens": r.input_tokens,
        "output_tokens": r.output_tokens,
        "latency_ms": r.latency_ms,
        "cost_microdollars": r.cost_microdollars,
        "created_at_local": r.created_at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}

/// Build the prominent failure / denial banner shown above the chain. None if
/// nothing is amiss — the caller suppresses the banner entirely.
fn build_banner(
    primary: Option<&AiRequestSummary>,
    env: &ChainEnvelope,
) -> Option<serde_json::Value> {
    let status = primary.map(|r| r.status.as_str());
    let error_message = primary.and_then(|r| r.error_message.as_deref());
    let failed = status.is_some_and(is_failed_status);
    let denial = env.decisions.iter().find(|d| d.decision == "deny").map(|d| {
        json!({
            "policy": d.policy,
            "reason": d.reason,
            "tool_name": d.tool_name,
            "decision_id": d.id,
            "evaluated_rules": d.evaluated_rules,
        })
    });
    if !failed && denial.is_none() && error_message.is_none() {
        return None;
    }
    Some(json!({
        "is_denial": denial.is_some(),
        "is_failure": failed && denial.is_none(),
        "status": status,
        "error_message": error_message,
        "denial": denial,
    }))
}

fn short_id(id: &str) -> String {
    if id.len() > 12 {
        format!("{}…", &id[..12])
    } else {
        id.to_string()
    }
}
