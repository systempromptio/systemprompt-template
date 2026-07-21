//! `/admin/governance/decisions/{id}` — single-request audit detail page.
//!
//! `id` may be an `ai_requests.id`, `request_id`, or `governance_decisions.id`.
//! Renders the full chain (identity, policy evaluations, prompt/response
//! preview, cost, latency, linked trace) using the existing
//! `find_decision_chain` envelope.

use crate::error::AdminError;
use std::sync::Arc;

use systemprompt::identifiers::{AgentId, AiRequestId, SessionId, TraceId, UserId};

use axum::extract::{Extension, Path, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;

use crate::error::AdminHtmlResult;
use crate::repositories::governance::chain::{
    AiRequestSummary, ChainEnvelope, DecisionStage, TranscriptEnvelope, find_decision_chain,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};


#[derive(Debug, Serialize)]
struct AuditDetailContext<'a> {
    page: &'static str,
    title: String,
    summary: Summary,
    primary: Option<PrimaryRequest>,
    banner: Option<Banner>,
    decisions: &'a [DecisionStage],
    requests: &'a [AiRequestSummary],
    events: &'a [crate::repositories::governance::chain::ChainUsageEvent],
    transcript: &'a Option<TranscriptEnvelope>,
    session_summary: &'a Option<crate::repositories::governance::chain::SessionSummary>,
    back_url: &'static str,
}

#[derive(Debug, Serialize)]
struct Summary {
    session_id: SessionId,
    session_id_short: String,
    trace_id: Option<TraceId>,
    trace_url: Option<String>,
    session_url: String,
    user_id: UserId,
    agent_id: Option<AgentId>,
    agent_scope: Option<String>,
    decision_count: i64,
    deny_count: i64,
    request_count: i64,
    input_tokens: i64,
    output_tokens: i64,
    cost_usd: String,
    cost_microdollars: i64,
    latency_ms: i64,
}

#[derive(Debug, Serialize)]
struct PrimaryRequest {
    id: String,
    request_id: AiRequestId,
    model: String,
    provider: String,
    status: String,
    is_failed: bool,
    error_message: Option<String>,
    input_tokens: Option<i32>,
    output_tokens: Option<i32>,
    latency_ms: Option<i32>,
    cost_microdollars: i64,
    created_at_local: String,
}

#[derive(Debug, Serialize)]
struct Banner {
    is_denial: bool,
    is_failure: bool,
    status: Option<String>,
    error_message: Option<String>,
    denial: Option<Denial>,
}

#[derive(Debug, Serialize)]
struct Denial {
    policy: String,
    reason: String,
    tool_name: String,
    decision_id: String,
    evaluated_rules: serde_json::Value,
}

pub(crate) async fn governance_audit_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let Some(envelope) = find_decision_chain(&pool, &id).await? else {
        return Err(AdminError::NotFound("No audit chain found for that id.".to_owned()).into());
    };

    let primary = pick_primary(&envelope, &id);
    let title = primary.map_or_else(
        || format!("Request · {}", short_id(envelope.session_id.as_str())),
        |r| format!("Request · {}", short_id(&r.id)),
    );

    let summary = build_summary(&envelope);
    let primary_json = primary.map(build_primary_json);
    let banner = build_banner(primary, &envelope);

    let ctx = AuditDetailContext {
        page: "request-detail",
        title,
        summary,
        primary: primary_json,
        banner,
        decisions: &envelope.decisions,
        requests: &envelope.requests,
        events: &envelope.events,
        transcript: &envelope.transcript,
        session_summary: &envelope.summary,
        back_url: "/admin/governance/decisions",
    };

    Ok(super::render_typed_page(
        &engine,
        "governance-audit-detail",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}

/// Pin the page to the request the user clicked when possible — fall back
/// to the first request in the session.
fn pick_primary<'a>(env: &'a ChainEnvelope, id: &str) -> Option<&'a AiRequestSummary> {
    env.requests
        .iter()
        .find(|r| r.id == id || r.request_id == id)
        .or_else(|| env.requests.first())
}

fn build_summary(env: &ChainEnvelope) -> Summary {
    let total_latency = env
        .requests
        .iter()
        .filter_map(|r| r.latency_ms)
        .map(i64::from)
        .sum::<i64>();
    let cost_usd = env.totals.total_cost_microdollars as f64 / 1_000_000.0;
    Summary {
        session_id: env.session_id.clone(),
        session_id_short: short_id(env.session_id.as_str()),
        trace_id: env.trace_id.clone(),
        trace_url: env
            .trace_id
            .as_ref()
            .map(|tid| format!("/admin/traces/{}", urlencoding::encode(tid.as_str()))),
        session_url: format!(
            "/admin/sessions/{}",
            urlencoding::encode(env.session_id.as_str())
        ),
        user_id: env.identity.user_id.clone(),
        agent_id: env.identity.agent_id.clone(),
        agent_scope: env.identity.agent_scope.clone(),
        decision_count: env.totals.decision_count,
        deny_count: env.totals.deny_count,
        request_count: env.totals.request_count,
        input_tokens: env.totals.total_input_tokens,
        output_tokens: env.totals.total_output_tokens,
        cost_usd: format!("{cost_usd:.4}"),
        cost_microdollars: env.totals.total_cost_microdollars,
        latency_ms: total_latency,
    }
}

fn is_failed_status(status: &str) -> bool {
    matches!(status, "failed" | "error" | "denied")
}

fn build_primary_json(r: &AiRequestSummary) -> PrimaryRequest {
    PrimaryRequest {
        id: r.id.clone(),
        request_id: r.request_id.clone(),
        model: r.model.clone(),
        provider: r.provider.clone(),
        status: r.status.clone(),
        is_failed: is_failed_status(&r.status),
        error_message: r.error_message.clone(),
        input_tokens: r.input_tokens,
        output_tokens: r.output_tokens,
        latency_ms: r.latency_ms,
        cost_microdollars: r.cost_microdollars,
        created_at_local: r
            .created_at
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    }
}

/// Build the prominent failure / denial banner shown above the chain. None if
/// nothing is amiss — the caller suppresses the banner entirely.
fn build_banner(primary: Option<&AiRequestSummary>, env: &ChainEnvelope) -> Option<Banner> {
    let status = primary.map(|r| r.status.clone());
    let error_message = primary.and_then(|r| r.error_message.clone());
    let failed = status.as_deref().is_some_and(is_failed_status);
    let denial = env
        .decisions
        .iter()
        .find(|d| d.decision == "deny")
        .map(|d| Denial {
            policy: d.policy.clone(),
            reason: d.reason.clone(),
            tool_name: d.tool_name.clone(),
            decision_id: d.id.clone(),
            evaluated_rules: d.evaluated_rules.clone(),
        });
    if !failed && denial.is_none() && error_message.is_none() {
        return None;
    }
    let is_denial = denial.is_some();
    Some(Banner {
        is_denial,
        is_failure: failed && !is_denial,
        status,
        error_message,
        denial,
    })
}

fn short_id(id: &str) -> String {
    if id.len() > 12 {
        format!("{}…", &id[..12])
    } else {
        id.to_owned()
    }
}
