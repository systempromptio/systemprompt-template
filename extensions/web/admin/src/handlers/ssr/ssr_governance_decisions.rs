//! `/admin/governance/decisions` — the decisions ledger: recent governance
//! decisions inside a time window, filterable by policy, outcome and user.
//! The drill-through target for every policy row and KPI tile.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::format::local_time;
use crate::handlers::ssr::list_view::{
    SelectOptionView, TimeRangeContext, preset_str, time_range_context,
};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::handlers::webhook::governance;
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{DECISION_DENY, GovernanceDecisionRow, MarketplaceContext, UserContext};
use crate::util::time_range::{TimeRange, TimeRangeQuery, parse_time_range};

const BASE_URL: &str = "/admin/governance/decisions";
const DECISIONS_LIMIT: i64 = 200;

#[derive(Debug, Deserialize)]
pub(crate) struct DecisionsQuery {
    policy: Option<String>,
    outcome: Option<String>,
    user_id: Option<UserId>,
    preset: Option<String>,
    from: Option<String>,
    to: Option<String>,
}

#[derive(Debug, Serialize)]
struct DecisionRowView {
    created_at: String,
    policy: String,
    policy_url: String,
    decision: String,
    decision_tone: &'static str,
    is_deny: bool,
    tool_name: String,
    user_id: UserId,
    user_url: String,
    agent_scope: String,
    reason: String,
}

#[derive(Debug, Serialize)]
struct DecisionsKpiView {
    label: &'static str,
    value: String,
    note: String,
    tone: &'static str,
}

#[derive(Debug, Serialize)]
struct GovernanceDecisionsContext {
    page: &'static str,
    title: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    base_url: &'static str,
    time_range: TimeRangeContext,
    kpis: Vec<DecisionsKpiView>,
    total: usize,
    at_limit: bool,
    limit: i64,
    rows: Vec<DecisionRowView>,
    has_rows: bool,
    policy_options: Vec<SelectOptionView>,
    outcome_options: Vec<SelectOptionView>,
    user_filter: String,
    filters_applied: bool,
    clear_url: &'static str,
}

fn normalize(param: Option<&String>) -> Option<&str> {
    param.map(String::as_str).filter(|s| !s.is_empty())
}

fn options(
    entries: &[(String, String)],
    all: &str,
    selected: Option<&str>,
) -> Vec<SelectOptionView> {
    let mut out = vec![SelectOptionView {
        value: String::new(),
        label: all.to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(entries.iter().map(|(value, label)| SelectOptionView {
        selected: selected == Some(value.as_str()),
        value: value.clone(),
        label: label.clone(),
    }));
    out
}

// Why: the policy menu lists what the chain runs plus any id the rows carry,
// so every id with rows on record is selectable, registered or not.
fn policy_options(
    rows: &[GovernanceDecisionRow],
    selected: Option<&str>,
) -> Result<Vec<SelectOptionView>, systemprompt_security::policy::GovernanceEngineError> {
    let mut ids: Vec<String> = governance::engine()?
        .policies()
        .map(|(_, p)| p.id().as_str().to_owned())
        .collect();
    ids.extend(rows.iter().map(|r| r.policy.clone()));
    if let Some(s) = selected {
        ids.push(s.to_owned());
    }
    ids.sort();
    ids.dedup();
    let entries: Vec<(String, String)> = ids.into_iter().map(|id| (id.clone(), id)).collect();
    Ok(options(&entries, "All policies", selected))
}

fn outcome_options(selected: Option<&str>) -> Vec<SelectOptionView> {
    options(
        &[
            ("allow".to_owned(), "Allowed".to_owned()),
            ("deny".to_owned(), "Denied".to_owned()),
        ],
        "Any outcome",
        selected,
    )
}

fn row_view(d: GovernanceDecisionRow) -> DecisionRowView {
    let is_deny = d.decision == DECISION_DENY;
    DecisionRowView {
        created_at: local_time(d.created_at),
        policy_url: format!("/admin/governance/policies/{}", d.policy),
        decision_tone: if is_deny { "err" } else { "ok" },
        is_deny,
        user_url: format!("/admin/user?id={}", urlencoding::encode(d.user_id.as_str())),
        user_id: d.user_id,
        policy: d.policy,
        decision: d.decision,
        tool_name: d.tool_name,
        agent_scope: d.agent_scope.unwrap_or_default(),
        reason: d.reason,
    }
}

fn kpis(rows: &[DecisionRowView], window_label: &str) -> Vec<DecisionsKpiView> {
    let denied = rows.iter().filter(|r| r.is_deny).count();
    let mut policies: Vec<&str> = rows.iter().map(|r| r.policy.as_str()).collect();
    policies.sort_unstable();
    policies.dedup();
    let mut actors: Vec<&str> = rows.iter().map(|r| r.user_id.as_str()).collect();
    actors.sort_unstable();
    actors.dedup();
    vec![
        DecisionsKpiView {
            label: "Decisions",
            value: rows.len().to_string(),
            note: format!("in the last {window_label}"),
            tone: "accent",
        },
        DecisionsKpiView {
            label: "Allowed",
            value: (rows.len() - denied).to_string(),
            note: "passed every enabled stage".to_owned(),
            tone: "ok",
        },
        DecisionsKpiView {
            label: "Denied",
            value: denied.to_string(),
            note: "stopped at the first failing stage".to_owned(),
            tone: if denied > 0 { "err" } else { "ok" },
        },
        DecisionsKpiView {
            label: "Policies firing",
            value: policies.len().to_string(),
            note: policies.join(", "),
            tone: "accent",
        },
        DecisionsKpiView {
            label: "People",
            value: actors.len().to_string(),
            note: "distinct accounts in these rows".to_owned(),
            tone: "accent",
        },
    ]
}

fn window_label(range: TimeRange) -> String {
    let mins = (range.to - range.from).num_minutes();
    if mins < 60 {
        format!("{mins}m")
    } else if mins < 24 * 60 {
        format!("{}h", mins / 60)
    } else {
        format!("{}d", mins / (24 * 60))
    }
}

pub(crate) async fn governance_decisions_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<DecisionsQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let policy = normalize(params.policy.as_ref());
    let outcome = normalize(params.outcome.as_ref());
    let user_id = params.user_id.as_ref();
    let tr_query = TimeRangeQuery {
        from: params.from.clone(),
        to: params.to.clone(),
        preset: params.preset.clone(),
    };
    let range = parse_time_range(&tr_query);
    let preset = preset_str(&tr_query, range);

    // Why: the ledger query has no window parameters and adding one would
    // need a fresh sqlx offline cache, so the newest rows matching the facet
    // filters are fetched and the window is applied here. The cap is the
    // same either way; a window that starts before the oldest fetched row is
    // reported through `at_limit` rather than silently answered short.
    let decisions = repositories::governance::decisions::list_decisions_filtered(
        &pool,
        policy,
        outcome,
        user_id,
        DECISIONS_LIMIT,
    )
    .await
    .map_err(AdminError::from)?;
    let fetched = decisions.len();
    let policy_options = policy_options(&decisions, policy).map_err(AdminError::internal)?;

    let rows: Vec<DecisionRowView> = decisions
        .into_iter()
        .filter(|d| d.created_at >= range.from && d.created_at <= range.to)
        .map(row_view)
        .collect();

    let total = rows.len();
    let filters_applied = policy.is_some() || outcome.is_some() || user_id.is_some();
    let ctx = GovernanceDecisionsContext {
        page: "governance-decisions",
        title: "Decisions",
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Governance", "/admin/governance"),
            BreadcrumbView::current("Decisions"),
        ],
        base_url: BASE_URL,
        time_range: time_range_context(BASE_URL, range, &preset),
        kpis: kpis(&rows, &window_label(range)),
        total,
        at_limit: fetched == usize::try_from(DECISIONS_LIMIT).unwrap_or(usize::MAX),
        limit: DECISIONS_LIMIT,
        has_rows: !rows.is_empty(),
        rows,
        policy_options,
        outcome_options: outcome_options(outcome),
        user_filter: user_id.map(ToString::to_string).unwrap_or_default(),
        filters_applied,
        clear_url: BASE_URL,
    };

    Ok(super::render_typed_page(
        &engine,
        "governance-decisions",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}
