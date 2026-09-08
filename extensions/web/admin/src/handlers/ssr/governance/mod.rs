//! `/admin/governance` — what the two enforcement planes decided.
//!
//! Three tabs over one window and one scope. **Decisions** is the governance
//! chain's log: one row per evaluated tool call, with the stage that decided
//! it. **Safety** is the gateway scanners' log, which runs in both directions
//! and records findings whether or not they refused anything. **Hooks** is the
//! plumbing underneath — which hook fired, how often, and who is tripping it
//! most.
//!
//! The two planes are configured separately (`services/governance/config.yaml`
//! and `services/gateway/policies.yaml`) and an operator retuning one has to
//! see what the other is doing, so the KPI strip spans both on every tab: the
//! four chain stages' deny counts beside the scanners' findings and blocks. A
//! findings count with no blocks under it is warn mode absorbing the traffic,
//! and that reading is the point of putting them side by side.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::handlers::ssr::csv::CsvBuilder;
use crate::handlers::ssr::list_view::scope_filter_view;
use crate::repositories::governance::decision_log::{DecisionFilter, DecisionSort};
use crate::repositories::governance::findings::FindingFilter;
use crate::repositories::scope::{ScopeRequest, SubjectScope, membership};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use crate::util::time_range::{TimeRange, TimeRangeQuery, parse_time_range};

mod columns;
mod context;
mod data;
mod kpis;
mod urls;
mod view;


pub(crate) const BASE_URL: &str = "/admin/governance";
const PAGE_SIZE: i64 = 50;
const CSV_LIMIT: i64 = 5_000;

// Why: Which log the URL asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GovernanceTab {
    Decisions,
    Safety,
    Hooks,
}

impl GovernanceTab {
    fn parse(raw: Option<&str>) -> Self {
        match raw {
            Some("safety") => Self::Safety,
            Some("hooks") => Self::Hooks,
            _ => Self::Decisions,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Decisions => "decisions",
            Self::Safety => "safety",
            Self::Hooks => "hooks",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Decisions => "Decisions",
            Self::Safety => "Safety findings",
            Self::Hooks => "Hook events",
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct GovernanceQuery {
    pub tab: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub group: Option<String>,
    pub project: Option<String>,
    pub policy: Option<String>,
    pub decision: Option<String>,
    pub category: Option<String>,
    pub blocked: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: Option<i64>,
}

// Why: One tab's link, as `components/tabs` reads it.
#[derive(Debug, Serialize)]
pub(super) struct TabLink {
    label: &'static str,
    href: String,
    is_active: bool,
    count: i64,
}

// Why: a read surface, so the console predicate applies — a project manager
// reads the governance posture they are accountable for without holding the
// admin role. Their scope narrows the rows; it does not gate the page.
fn require_console(user_ctx: &UserContext) -> Result<(), AdminError> {
    if user_ctx.is_console {
        return Ok(());
    }
    Err(AdminError::Forbidden("Admin access required.".to_owned()))
}

fn sort_from(query: &GovernanceQuery) -> DecisionSort {
    let key = match query.sort.as_deref() {
        Some("policy") => "policy",
        Some("decision") => "decision",
        Some("tool") => "tool",
        Some("user") => "user",
        _ => "when",
    };
    DecisionSort {
        key,
        ascending: query.dir.as_deref() == Some("asc"),
    }
}

fn decision_filter(query: &GovernanceQuery) -> DecisionFilter {
    DecisionFilter {
        policy: non_empty(query.policy.as_deref()),
        decision: non_empty(query.decision.as_deref()),
        search: non_empty(query.q.as_deref()),
    }
}

fn finding_filter(query: &GovernanceQuery) -> FindingFilter {
    FindingFilter {
        category: non_empty(query.category.as_deref()),
        blocked: match query.blocked.as_deref() {
            Some("blocked") => Some(true),
            Some("audited") => Some(false),
            _ => None,
        },
    }
}

fn non_empty(value: Option<&str>) -> Option<String> {
    value.filter(|v| !v.trim().is_empty()).map(str::to_owned)
}

fn range_of(query: &GovernanceQuery) -> TimeRange {
    parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    })
}

async fn resolve_scope(
    pool: &PgPool,
    user_ctx: &UserContext,
    query: &GovernanceQuery,
) -> Result<(ScopeRequest, SubjectScope), AdminError> {
    let request =
        ScopeRequest::from_query(user_ctx, query.group.as_deref(), query.project.as_deref());
    let scope = membership::get_subject_scope(pool, &request).await?;
    Ok((request, scope))
}

pub(crate) async fn governance_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<GovernanceQuery>,
) -> AdminHtmlResult<Response> {
    require_console(&user_ctx)?;

    let tab = GovernanceTab::parse(query.tab.as_deref());
    let range = range_of(&query);
    let (request, scope) = resolve_scope(&pool, &user_ctx, &query).await?;
    let sort = sort_from(&query);
    let page = query.page.unwrap_or(0).max(0);

    let loaded = data::load(
        &pool,
        data::GovernanceRead {
            tab,
            range,
            scope: &scope,
            decision_filter: &decision_filter(&query),
            finding_filter: &finding_filter(&query),
            sort,
            page_size: PAGE_SIZE,
            offset: page * PAGE_SIZE,
        },
    )
    .await;

    let scope_filter = scope_filter_view(
        &pool,
        &user_ctx,
        &request,
        BASE_URL,
        vec![("tab".to_owned(), tab.as_str().to_owned())],
    )
    .await;

    let ctx = context::build(context::Build {
        query: &query,
        tab,
        range,
        page,
        sort,
        scope_filter,
        data: &loaded,
    });

    Ok(super::render_typed_page(
        &engine,
        "governance-warnings",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}

// Why: one file carrying both planes rather than two downloads. The window is
// the whole point of the export — a reviewer comparing what the chain refused
// with what the scanners saw needs the two sets to be provably the same window,
// and two files lose that.
pub(crate) async fn governance_csv(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<GovernanceQuery>,
) -> AdminResult<Response> {
    require_console(&user_ctx)?;

    let range = range_of(&query);
    let (_, scope) = resolve_scope(&pool, &user_ctx, &query).await?;

    let (decisions, _) =
        crate::repositories::governance::decision_log::list_governance_decisions_paged(
            &pool,
            range,
            &scope,
            &decision_filter(&query),
            sort_from(&query),
            CSV_LIMIT,
            0,
        )
        .await?;
    let (findings, _) = crate::repositories::governance::findings::list_safety_findings_paged(
        &pool,
        range,
        &scope,
        &finding_filter(&query),
        CSV_LIMIT,
        0,
    )
    .await?;

    let mut csv = CsvBuilder::new(&[
        "plane",
        "at",
        "outcome",
        "policy_or_category",
        "stage_or_scanner",
        "tool_or_model",
        "user",
        "scope",
        "reason",
    ]);
    for row in &decisions {
        csv.row(&[
            "chain",
            &row.created_at.to_rfc3339(),
            &row.decision,
            &row.policy,
            view::stage_of(&row.policy),
            &row.tool_name,
            row.user_id.as_str(),
            row.agent_scope.as_deref().unwrap_or(""),
            &row.reason,
        ]);
    }
    for row in &findings {
        csv.row(&[
            "safety",
            &row.created_at.to_rfc3339(),
            if row.blocked { "blocked" } else { "audited" },
            &row.category,
            &row.scanner,
            row.model.as_deref().unwrap_or(""),
            row.user_id.as_ref().map_or("", |u| u.as_str()),
            &row.phase,
            row.excerpt.as_deref().unwrap_or(""),
        ]);
    }

    Ok(csv.into_response(&format!("governance-{}.csv", range.from.format("%Y%m%d"))))
}
