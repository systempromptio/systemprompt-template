//! Typed view-model structs for the site analytics dashboard
//! (`analytics-dashboard`). Mirrors every `{{field}}` / `{{#each}}` /
//! `{{#if}}` referenced by
//! `storage/files/admin/templates/analytics-dashboard.hbs`.

use serde::Serialize;

use crate::handlers::ssr::types::{PieView, SvgLineChartView, SvgStackedChartView};

pub(super) use super::context_overview::{
    AnomalyRowView, BucketLinkView, CodeFrameView, DashboardTabLink, DashboardTimeRange,
    FastSlowView, FiltersView, KpiStripView, LeaderRowView, LeaderboardView, PermissionStatsView,
    ScopeChipView, SessionCostsView, SloOption, SortLinkView, ThinkingView,
};
pub(super) use super::context_tabs::{
    ContainerRowView, CostTabView, ModelUsageRowView, ModelsTabView, RedirectRowView,
    SessionCostRowView, SessionsTabView, SkillModelRowView, SkillRowView, SkillsTabView,
    SupplierRowView, ToolRowView, ToolServerRowView, ToolsTabView,
};

// Why: each tab is its own GET so it can be bookmarked, and so only the
// queries that tab renders ever run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DashboardTab {
    Overview,
    Models,
    Skills,
    Tools,
    Sessions,
    Cost,
}

impl DashboardTab {
    // Why: anything unrecognised lands on Overview — a mistyped tab in a
    // shared link should still show the page rather than a 400.
    pub(super) fn from_query(raw: Option<&str>) -> Self {
        match raw {
            Some("models") => Self::Models,
            Some("skills") => Self::Skills,
            Some("tools") => Self::Tools,
            Some("sessions") => Self::Sessions,
            Some("cost") => Self::Cost,
            _ => Self::Overview,
        }
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Models => "models",
            Self::Skills => "skills",
            Self::Tools => "tools",
            Self::Sessions => "sessions",
            Self::Cost => "cost",
        }
    }
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "one is_* flag per tab plus the template's has_* guards; handlebars branches on \
              flat booleans, so folding them into an enum would just move the bools into the \
              serializer"
)]
#[derive(Debug, Serialize)]
pub(super) struct AnalyticsDashboardContext {
    pub page: &'static str,
    pub title: String,
    pub time_range: DashboardTimeRange,
    pub tabs: Vec<DashboardTabLink>,
    // Why: the window's headline figures, on the toolbar rather than under the
    // title. The page header used to carry a meta line saying the same thing,
    // which cost a row of vertical space the tables needed.
    pub toolbar_count: String,
    pub breadcrumbs: Vec<Crumb>,
    pub is_overview: bool,
    pub is_models: bool,
    pub is_skills: bool,
    pub is_tools: bool,
    pub is_sessions: bool,
    pub is_cost: bool,
    // Why: the one thing a reader must be told before summing a column. Member
    // attribution counts a person in every container they belong to, so the
    // rows deliberately overlap and the page says so rather than letting the
    // reader discover it by adding up to more than the instance.
    pub is_member_view: bool,
    pub attribution_links: Vec<AttributionLink>,

    pub filters: FiltersView,
    pub chips: Vec<ScopeChipView>,
    pub has_active_filters: bool,
    pub clear_url: String,
    pub base_url: &'static str,

    pub kpis: KpiStripView,
    pub volume_chart: SvgLineChartView,
    pub cost_chart: SvgLineChartView,
    pub model_pie: PieView,
    pub model_cost_chart: SvgStackedChartView,

    pub leaderboard: LeaderboardView,
    pub permissions: PermissionStatsView,

    pub slo_options: Vec<SloOption>,
    pub latency_link: String,
    pub anomalies: Vec<AnomalyRowView>,
    pub has_anomalies: bool,
    pub fast_slow: FastSlowView,

    pub session_costs: SessionCostsView,
    pub thinking: ThinkingView,

    pub commit_chart: SvgLineChartView,
    pub loc_chart: SvgLineChartView,
    pub code_frames: Vec<CodeFrameView>,

    pub models: ModelsTabView,
    pub skills: SkillsTabView,
    pub tools: ToolsTabView,
    pub sessions: SessionsTabView,
    pub cost: CostTabView,
}

#[derive(Debug, Serialize)]
pub(super) struct Crumb {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct AttributionLink {
    pub label: &'static str,
    pub href: String,
    pub is_active: bool,
    pub hint: &'static str,
}

#[derive(Debug, Default, Serialize)]
pub(super) struct KpiTile {
    pub label: String,
    pub value: String,
    pub sub: String,
    pub tone: &'static str,
}
