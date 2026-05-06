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

use crate::repositories::governance_grp::chain::fetch_decision_chain;
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

    let primary = envelope.requests.first();
    let title = primary.map_or_else(
        || format!("Request · {}", short_id(&envelope.session_id)),
        |r| format!("Request · {}", short_id(&r.id)),
    );

    let total_latency = envelope
        .requests
        .iter()
        .filter_map(|r| r.latency_ms)
        .map(i64::from)
        .sum::<i64>();
    let cost_usd = envelope.totals.total_cost_microdollars as f64 / 1_000_000.0;

    let summary = json!({
        "session_id": envelope.session_id,
        "session_id_short": short_id(&envelope.session_id),
        "trace_id": envelope.trace_id,
        "trace_url": envelope.trace_id.as_ref().map(|_| {
            format!("/admin/performance/traces/{}", urlencoding::encode(&envelope.session_id))
        }),
        "user_id": envelope.identity.user_id,
        "agent_id": envelope.identity.agent_id,
        "agent_scope": envelope.identity.agent_scope,
        "decision_count": envelope.totals.decision_count,
        "deny_count": envelope.totals.deny_count,
        "request_count": envelope.totals.request_count,
        "input_tokens": envelope.totals.total_input_tokens,
        "output_tokens": envelope.totals.total_output_tokens,
        "cost_usd": format!("{:.4}", cost_usd),
        "cost_microdollars": envelope.totals.total_cost_microdollars,
        "latency_ms": total_latency,
    });

    let data = json!({
        "page": "governance-audit-detail",
        "title": title,
        "summary": summary,
        "decisions": envelope.decisions,
        "requests": envelope.requests,
        "events": envelope.events,
        "transcript": envelope.transcript,
        "session_summary": envelope.summary,
        "back_url": "/admin/governance/decisions",
    });

    super::render_page(&engine, "governance-audit-detail", &data, &user_ctx, &mkt_ctx)
}

fn short_id(id: &str) -> String {
    if id.len() > 12 {
        format!("{}…", &id[..12])
    } else {
        id.to_string()
    }
}
