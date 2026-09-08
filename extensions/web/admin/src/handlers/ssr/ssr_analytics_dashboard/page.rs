//! Assembly of the whole page context from the loaded data and the query.
//!
//! Every tab's view builder is called here, so the template is handed one
//! fully-formed struct and branches on booleans rather than deciding anything
//! itself. Split from `mod.rs` at the 300-line ceiling.

use crate::handlers::ssr::format::format_cost;
use crate::repositories::analytics::site::leaderboards::LeaderboardSort;
use crate::repositories::analytics::site::series::SeriesBucket;
use crate::repositories::scope::Attribution;
use crate::util::time_range::TimeRange;

use super::context::{AnalyticsDashboardContext, Crumb, DashboardTab, FiltersView};
use super::data::AnalyticsDashboardData;
use super::{
    AnalyticsDashboardQuery, BASE_URL, context, tab_cost, tab_models, tab_sessions, tab_skills,
    tab_tools, urls, view, view_code, view_models, view_spend, view_tables,
};

pub(super) fn sort_links(query: &AnalyticsDashboardQuery) -> Vec<context::SortLinkView> {
    [
        ("Requests", "requests"),
        ("Cost", "cost"),
        ("Tokens", "tokens"),
        ("Last active", "last_active"),
    ]
    .into_iter()
    .map(|(label, key)| context::SortLinkView {
        label,
        href: urls::sort_url(query, key),
        is_active: LeaderboardSort::from_sort_param(query.sort.as_deref())
            == LeaderboardSort::from_sort_param(Some(key)),
    })
    .collect()
}

pub(super) struct PageInput<'a> {
    pub query: &'a AnalyticsDashboardQuery,
    pub tab: DashboardTab,
    pub range: TimeRange,
    pub bucket: SeriesBucket,
    pub page: i64,
    pub filters: FiltersView,
    pub fetched: &'a AnalyticsDashboardData,
    pub slo_ms: i32,
}

#[expect(
    clippy::too_many_lines,
    reason = "one page assembly per handler; splitting is tracked in docs/tech-debt.md"
)]
pub(super) fn page_context(input: PageInput<'_>) -> AnalyticsDashboardContext {
    let PageInput {
        query,
        tab,
        range,
        bucket,
        page,
        filters,
        fetched,
        slo_ms,
    } = input;
    let weekly = bucket == SeriesBucket::Week;

    let leaderboard = view_tables::leaderboard_view(fetched, &range, query, page);
    let chips = urls::active_chips(query);
    let has_active_filters = !chips.is_empty();
    let charts = view_models::overview_charts(fetched, query, &range, weekly);

    AnalyticsDashboardContext {
        page: "analytics-dashboard",
        title: "Analytics".to_owned(),
        time_range: view::time_range_view(query, &range),
        tabs: urls::tab_links(tab, query),
        toolbar_count: format!(
            "{} requests · {} · {} active people",
            fetched.kpis.total_requests,
            format_cost(fetched.kpis.total_cost_microdollars),
            fetched.kpis.active_users
        ),
        breadcrumbs: breadcrumbs(tab),
        is_overview: tab == DashboardTab::Overview,
        is_models: tab == DashboardTab::Models,
        is_skills: tab == DashboardTab::Skills,
        is_tools: tab == DashboardTab::Tools,
        is_sessions: tab == DashboardTab::Sessions,
        is_cost: tab == DashboardTab::Cost,
        is_member_view: query.attribution() == Attribution::Member,
        attribution_links: urls::attribution_links(
            query,
            query.attribution() == Attribution::Member,
        ),

        filters,
        chips,
        has_active_filters,
        clear_url: urls::clear_url(query),
        base_url: BASE_URL,

        kpis: view::kpi_strip(
            &fetched.kpis,
            &range,
            (tab == DashboardTab::Overview).then_some(fetched.series.as_slice()),
        ),
        volume_chart: charts.volume,
        cost_chart: charts.cost,
        model_pie: charts.model_pie,
        model_cost_chart: charts.model_cost,

        leaderboard,
        permissions: view_tables::permission_stats(&fetched.permissions),

        slo_options: urls::slo_links(query, slo_ms),
        latency_link: "/admin/requests".to_owned(),
        has_anomalies: !fetched.anomalies.is_empty(),
        anomalies: view_spend::anomaly_rows(&fetched.anomalies),
        fast_slow: view_spend::fast_slow(&fetched.latency),
        session_costs: view_spend::session_costs(&fetched.session_costs),
        thinking: view_spend::thinking(&fetched.kpis),

        commit_chart: view_code::commit_chart(&fetched.code_series, &range),
        loc_chart: view_code::loc_chart(&fetched.code_series, &range),
        code_frames: view_code::code_frames(&fetched.code_totals),

        models: tab_models::models_tab(&fetched.tabs.models, &fetched.tabs.redirects, query),
        skills: tab_skills::skills_tab(
            &tab_skills::SkillsInput {
                rows: &fetched.tabs.skills,
                total_rows: fetched.tabs.skills_total,
                by_model: &fetched.tabs.skill_models,
                totals: fetched.tabs.skill_totals,
                page,
            },
            query,
        ),
        tools: tab_tools::tools_tab(
            &fetched.tabs.tool_servers,
            &fetched.tabs.tools,
            fetched.tabs.tools_total,
            (page, query),
        ),
        sessions: tab_sessions::sessions_tab(
            &tab_sessions::SessionsInput {
                rows: &fetched.tabs.sessions,
                total_rows: fetched.tabs.sessions_total,
                ratings: fetched.tabs.session_ratings,
                page,
            },
            query,
        ),
        cost: tab_cost::cost_tab(
            &tab_cost::CostInput {
                days: &fetched.tabs.cost_days,
                providers: &fetched.tabs.cost_providers,
                models: &fetched.tabs.cost_models,
                containers: &fetched.tabs.cost_containers,
                axis: query.container_axis(),
                is_internal: query.is_internal_audience(),
            },
            &range,
            query,
        ),
    }
}

// Why: a tab is a view of one page, not a page of its own, so the trail names
// the tab rather than pretending `/admin/analytics?tab=models` is a child
// route. The section root is always reachable from the second crumb.
fn breadcrumbs(tab: DashboardTab) -> Vec<Crumb> {
    let mut crumbs = vec![
        Crumb {
            label: "Admin".to_owned(),
            href: Some("/admin".to_owned()),
        },
        Crumb {
            label: "Analytics".to_owned(),
            href: (tab != DashboardTab::Overview).then(|| BASE_URL.to_owned()),
        },
    ];
    if tab != DashboardTab::Overview {
        crumbs.push(Crumb {
            label: tab_label(tab).to_owned(),
            href: None,
        });
    }
    crumbs
}

const fn tab_label(tab: DashboardTab) -> &'static str {
    match tab {
        DashboardTab::Overview => "Overview",
        DashboardTab::Models => "Models",
        DashboardTab::Skills => "Skills",
        DashboardTab::Tools => "Tools",
        DashboardTab::Sessions => "Sessions",
        DashboardTab::Cost => "Cost",
    }
}
