//! `/admin/governance/decisions/{id}` and `/admin/requests/{id}` —
//! single-request audit detail page.
//!
//! `id` may be an `ai_requests.id`, `request_id`, or `governance_decisions.id`.
//! Renders the full chain (identity, policy evaluations, prompt/response
//! preview, cost, latency, linked trace) using the existing
//! `find_decision_chain` envelope, the client and provider tool schemas the
//! request carried, and the session's stored artifacts.

use crate::error::AdminError;
use std::sync::Arc;

use systemprompt::identifiers::{AgentId, SessionId, TraceId, UserId};

use axum::extract::{Path, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;

use crate::dashboard::link_to;
use crate::error::AdminHtmlResult;
use crate::handlers::ssr::entity_urls::{session_detail_url, trace_detail_url};
use crate::handlers::ssr::page::Page;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::governance::chain::{
    ChainEnvelope, DecisionStage, TranscriptEnvelope, find_decision_chain,
};

#[cfg(feature = "agents")]
mod artifacts;
#[cfg(feature = "gateway")]
mod evidence;
#[cfg(feature = "gateway")]
mod request;
#[cfg(feature = "gateway")]
mod rows;
#[cfg(feature = "gateway")]
mod schemas;

#[derive(Debug, Serialize)]
struct AuditDetailContext<'a> {
    page: &'static str,
    title: String,
    breadcrumbs: Vec<BreadcrumbView>,
    #[cfg(feature = "gateway")]
    #[serde(flatten)]
    panels: request::RequestPanels,
    summary: Summary,
    banner: Option<Banner>,
    decisions: &'a [DecisionStage],
    events: &'a [crate::repositories::governance::chain::ChainUsageEvent],
    session_summary: &'a Option<crate::repositories::governance::chain::SessionSummary>,
    transcript: &'a Option<TranscriptEnvelope>,
    back_url: &'static str,
}

#[derive(Debug, Serialize)]
struct Summary {
    session_id: SessionId,
    session_id_short: String,
    trace_id: Option<TraceId>,
    trace_url: Option<String>,
    session_url: Option<String>,
    user_id: UserId,
    user_url: String,
    agent_id: Option<AgentId>,
    agent_scope: Option<String>,
    decision_count: i64,
    deny_count: i64,
    #[cfg(feature = "gateway")]
    #[serde(flatten)]
    usage: request::RequestUsage,
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
    // JSON: rule-evaluation detail is policy-shaped and rendered by the template's
    // JSON: `{{json}}` helper, so no fixed struct exists to decode it into
    evaluated_rules: serde_json::Value,
}

pub(crate) async fn governance_audit_detail_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let Some(envelope) = find_decision_chain(&pool, &id).await? else {
        return Err(AdminError::NotFound("No audit chain found for that id.".to_owned()).into());
    };

    #[cfg(feature = "gateway")]
    let (panels, title, failure) = {
        let primary = request::pick_primary(&envelope, &id);
        let title = primary.map_or_else(
            || format!("Request · {}", short_id(envelope.session_id.as_str())),
            |r| format!("Request · {}", short_id(&r.id)),
        );
        let failure = primary.map(|r| (r.status.clone(), r.error_message.clone()));
        (
            request::load_request_panels(&pool, &envelope, primary).await,
            title,
            failure,
        )
    };
    #[cfg(not(feature = "gateway"))]
    let (title, failure) = (
        format!("Decision · {}", short_id(envelope.session_id.as_str())),
        None,
    );

    let summary = build_summary(&envelope);
    let banner = build_banner(failure, &envelope);

    let ctx = AuditDetailContext {
        page: "request-detail",
        breadcrumbs: vec![
            BreadcrumbView::link(BACK_LABEL, BACK_URL),
            BreadcrumbView::current(title.clone()),
        ],
        title,
        #[cfg(feature = "gateway")]
        panels,
        summary,
        banner,
        decisions: &envelope.decisions,
        events: &envelope.events,
        session_summary: &envelope.summary,
        transcript: &envelope.transcript,
        back_url: BACK_URL,
    };

    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        "governance-audit-detail",
        &ctx,
        &shell.user,
        &shell.marketplace,
    ))
}

// Why: the log this page is reached from. A reader who clicked a request row
// expects the request list back; without the gateway member the page is only
// reached from the governance decision log.
#[cfg(feature = "gateway")]
const BACK_URL: &str = "/admin/requests";
#[cfg(feature = "gateway")]
const BACK_LABEL: &str = "Requests";
#[cfg(not(feature = "gateway"))]
const BACK_URL: &str = "/admin/governance";
#[cfg(not(feature = "gateway"))]
const BACK_LABEL: &str = "Governance";

fn build_summary(env: &ChainEnvelope) -> Summary {
    Summary {
        session_id: env.session_id.clone(),
        session_id_short: short_id(env.session_id.as_str()),
        trace_id: env.trace_id.clone(),
        trace_url: env
            .trace_id
            .as_ref()
            .and_then(|t| link_to("traces", || trace_detail_url(t))),
        session_url: link_to("sessions", || session_detail_url(&env.session_id)),
        user_id: env.identity.user_id.clone(),
        user_url: format!(
            "/admin/users/{}",
            urlencoding::encode(env.identity.user_id.as_str())
        ),
        agent_id: env.identity.agent_id.clone(),
        agent_scope: env.identity.agent_scope.clone(),
        decision_count: env.totals.decisions,
        deny_count: env.totals.denials,
        #[cfg(feature = "gateway")]
        usage: request::build_usage(env),
    }
}

pub(super) fn is_failed_status(status: &str) -> bool {
    matches!(status, "failed" | "error" | "denied")
}

fn build_banner(failure: Option<(String, Option<String>)>, env: &ChainEnvelope) -> Option<Banner> {
    let (status, error_message) = failure.map_or((None, None), |(s, e)| (Some(s), e));
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
