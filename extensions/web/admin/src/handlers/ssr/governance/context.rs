//! The template context for `/admin/governance`.
//!
//! Assembling it is a pure function of what the database returned and what the
//! URL asked for, so it is separated from the handler that does the I/O: the
//! shape below is the whole contract with `governance-warnings.hbs`.

use serde::Serialize;

use super::columns::columns;
use super::data::GovernanceData;
use super::kpis::{GovernanceKpiView, StageFilterView, kpis, stage_filters};
use super::urls::{ColumnHeader, build_pagination, filter_url, url_with};
use super::{BASE_URL, GovernanceQuery, GovernanceTab, TabLink, view};
use crate::handlers::ssr::list_view::{
    PageWindow, Pagination, ScopeFilterView, SelectOptionView, TimeRangeContext,
};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::governance::decision_log::DecisionSort;
use crate::util::time_range::TimeRange;

// Why: One entry in a ranking list on the hooks tab.
#[derive(Debug, Serialize)]
pub(super) struct RankView {
    name: String,
    detail: String,
    count: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct GovernancePageContext {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) subtitle: &'static str,
    pub(super) breadcrumbs: Vec<BreadcrumbView>,
    pub(super) tabs: Vec<TabLink>,
    pub(super) tab_label: &'static str,
    pub(super) is_decisions: bool,
    pub(super) is_safety: bool,
    pub(super) is_hooks: bool,
    pub(super) time_range: TimeRangeContext,
    pub(super) scope_filter: ScopeFilterView,
    pub(super) kpis: Vec<GovernanceKpiView>,
    pub(super) stage_filters: Vec<StageFilterView>,
    pub(super) columns: Vec<ColumnHeader>,
    pub(super) decisions: Vec<view::DecisionRow>,
    pub(super) findings: Vec<view::FindingRow>,
    pub(super) hooks: Vec<view::HookRow>,
    pub(super) has_rows: bool,
    pub(super) row_count: String,
    pub(super) pagination: Pagination,
    pub(super) policies: Vec<SelectOptionView>,
    pub(super) categories: Vec<SelectOptionView>,
    pub(super) outcomes: Vec<SelectOptionView>,
    pub(super) search: String,
    pub(super) csv_url: String,
    pub(super) clear_url: String,
    pub(super) has_filters: bool,
    pub(super) base_url: &'static str,
    pub(super) decision_options: Vec<SelectOptionView>,
    pub(super) pretool_24h: i64,
    pub(super) posttool_24h: i64,
    pub(super) top_policies: Vec<RankView>,
    pub(super) top_actors: Vec<RankView>,
}

// Why: What one context build is derived from.
pub(super) struct Build<'a> {
    pub(super) query: &'a GovernanceQuery,
    pub(super) tab: GovernanceTab,
    pub(super) range: TimeRange,
    pub(super) page: i64,
    pub(super) sort: DecisionSort,
    pub(super) scope_filter: ScopeFilterView,
    pub(super) data: &'a GovernanceData,
}

#[expect(
    clippy::too_many_lines,
    reason = "one page assembly per handler; splitting is tracked in docs/tech-debt.md"
)]
pub(super) fn build(input: Build<'_>) -> GovernancePageContext {
    let Build {
        query,
        tab,
        range,
        page,
        sort,
        scope_filter,
        data,
    } = input;

    let (rows_shown, total, noun) = match tab {
        GovernanceTab::Decisions => (data.decisions.len(), data.decision_total, "decisions"),
        GovernanceTab::Safety => (data.findings.len(), data.finding_total, "findings"),
        GovernanceTab::Hooks => (
            data.hooks.len(),
            i64::try_from(data.hooks.len()).unwrap_or(0),
            "events",
        ),
    };
    let window = PageWindow::new(
        page,
        super::PAGE_SIZE,
        total,
        i64::try_from(rows_shown).unwrap_or(0),
        noun,
    );

    GovernancePageContext {
        page: "governance-warnings",
        title: "Governance",
        subtitle: "The policy chain and the safety scanners, over one window.",
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::current("Governance"),
        ],
        tabs: tab_links(query, tab, data),
        tab_label: tab.label(),
        is_decisions: tab == GovernanceTab::Decisions,
        is_safety: tab == GovernanceTab::Safety,
        is_hooks: tab == GovernanceTab::Hooks,
        time_range: time_range_context(query, range),
        scope_filter,
        kpis: kpis(query, data),
        stage_filters: stage_filters(query, data),
        columns: columns(query, tab, sort),
        decisions: view::decision_rows(&data.decisions),
        findings: view::finding_rows(&data.findings),
        hooks: view::hook_rows(&data.hooks),
        has_rows: rows_shown > 0,
        row_count: format!("{total} {noun}"),
        pagination: build_pagination(query, window, noun),
        policies: options("All policies", query.policy.as_deref(), &data.policies),
        categories: options(
            "All categories",
            query.category.as_deref(),
            &data.categories,
        ),
        outcomes: outcome_options(query),
        search: query.q.clone().unwrap_or_default(),
        csv_url: csv_url(query),
        clear_url: format!("{BASE_URL}?tab={}", tab.as_str()),
        has_filters: query.policy.is_some()
            || query.decision.is_some()
            || query.category.is_some()
            || query.blocked.is_some()
            || query.q.as_deref().is_some_and(|q| !q.is_empty()),
        base_url: BASE_URL,
        decision_options: decision_options(query),
        pretool_24h: data.pretool_24h,
        posttool_24h: data.posttool_24h,
        top_policies: data
            .top_policies
            .iter()
            .map(|p| RankView {
                name: p.policy.clone(),
                detail: p.tool_name.clone(),
                count: p.hits,
            })
            .collect(),
        top_actors: data
            .top_actors
            .iter()
            .map(|a| RankView {
                name: a.display_name.clone(),
                detail: a.email.clone().unwrap_or_else(|| a.user_id.to_string()),
                count: a.deny_count,
            })
            .collect(),
    }
}

fn tab_links(
    query: &GovernanceQuery,
    active: GovernanceTab,
    data: &GovernanceData,
) -> Vec<TabLink> {
    [
        (GovernanceTab::Decisions, data.stats.evaluated),
        (GovernanceTab::Safety, data.safety.findings),
        (GovernanceTab::Hooks, data.pretool_24h + data.posttool_24h),
    ]
    .into_iter()
    .map(|(tab, count)| TabLink {
        label: tab.label(),
        href: filter_url(query, &[("tab", tab.as_str())]),
        is_active: tab == active,
        count,
    })
    .collect()
}

fn options(all_label: &str, selected: Option<&str>, values: &[String]) -> Vec<SelectOptionView> {
    let mut out = vec![SelectOptionView {
        value: String::new(),
        label: all_label.to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(values.iter().map(|value| SelectOptionView {
        selected: selected == Some(value.as_str()),
        value: value.clone(),
        label: value.clone(),
    }));
    out
}

fn decision_options(query: &GovernanceQuery) -> Vec<SelectOptionView> {
    ["", "deny", "warn", "allow"]
        .into_iter()
        .map(|value| SelectOptionView {
            value: value.to_owned(),
            label: if value.is_empty() {
                "All outcomes".to_owned()
            } else {
                value.to_owned()
            },
            selected: query.decision.as_deref().unwrap_or("") == value,
        })
        .collect()
}

fn outcome_options(query: &GovernanceQuery) -> Vec<SelectOptionView> {
    ["", "blocked", "audited"]
        .into_iter()
        .map(|value| SelectOptionView {
            value: value.to_owned(),
            label: if value.is_empty() {
                "All outcomes".to_owned()
            } else {
                value.to_owned()
            },
            selected: query.blocked.as_deref().unwrap_or("") == value,
        })
        .collect()
}

fn time_range_context(query: &GovernanceQuery, range: TimeRange) -> TimeRangeContext {
    TimeRangeContext {
        preset: query.preset.clone().unwrap_or_else(|| "7d".to_owned()),
        from: range.from.format("%Y-%m-%dT%H:%M").to_string(),
        to: range.to.format("%Y-%m-%dT%H:%M").to_string(),
        base_url: BASE_URL,
        query: "",
        rejected: range.rejected_bounds,
    }
}

fn csv_url(query: &GovernanceQuery) -> String {
    url_with(query, &[("page", "")]).replacen(BASE_URL, &format!("{BASE_URL}/warnings.csv"), 1)
}
