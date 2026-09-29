//! Template context for `/admin/projects` and `/admin/projects/{id}`.
//!
//! Every displayed number is formatted here, so the templates do no arithmetic
//! and a column cannot disagree with the tile above it. Totals are exclusive
//! attribution — each person counts in one project, so the rows partition the
//! instance and the unattributed remainder is a tile of its own rather than a
//! rounding error. The one member-attributed table on the detail page carries
//! its own label saying so.

use serde::Serialize;
use systemprompt::identifiers::{SessionId, UserId};
use systemprompt_web_shared::ProjectId;

use super::super::list_view::Pagination;
use super::table::SortHeaderView;
use super::{
    AccessRowView, BreadcrumbView, MappingRowView, MemberRowView, MemberSetChipView,
    ModelMixRowView, SvgLineChartView, TabLinkView, UserOptionView,
};

// Why: one KPI tile: a number, its unit, and the sentence under it.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectKpiView {
    pub label: String,
    pub value: String,
    // Why: `note`, not `sub` — `sub` is a registered helper, so a bare `sub`
    // mustache in the kpi partial calls it instead of reading this field.
    pub note: String,
    pub tone: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

// Why: one project on the listing.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectListRowView {
    pub id: ProjectId,
    pub name: String,
    pub href: String,
    pub description: Option<String>,
    pub member_count: i64,
    pub active_members: i64,
    pub group_count: i64,
    pub requests: i64,
    pub tokens: i64,
    pub tokens_display: String,
    pub cost_display: String,
    pub models_used: i64,
    // Why: the vendor prefix repeats on every row of a single-vendor estate,
    // so the cell shows the tail and keeps the full id on its title.
    pub top_model: Option<String>,
    pub top_model_short: Option<String>,
    pub clients_used: i64,
    pub top_client: Option<String>,
    pub tool_calls: i64,
    pub tool_success_pct: i64,
    pub tool_tone: &'static str,
    pub skills_used: i64,
    pub artifacts: i64,
    pub report_href: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ProjectsPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub subtitle: &'static str,
    pub window_label: String,
    pub kpis: Vec<ProjectKpiView>,
    pub sort_headers: ProjectSortHeaders,
    pub rows: Vec<ProjectListRowView>,
    pub export: crate::export::ExportView,
    pub pagination: Pagination,
    pub count_label: String,
    pub query: String,
    pub truncated: bool,
    pub can_manage: bool,
}

// Why: named fields rather than a Vec so the template addresses each header by
// name and the column set is checked when the page compiles.
#[derive(Debug, Serialize)]
pub(crate) struct ProjectSortHeaders {
    pub name: SortHeaderView,
    pub members: SortHeaderView,
    pub groups: SortHeaderView,
    pub requests: SortHeaderView,
    pub tokens: SortHeaderView,
    pub cost: SortHeaderView,
    pub models: SortHeaderView,
    pub clients: SortHeaderView,
    pub tools: SortHeaderView,
    pub skills: SortHeaderView,
    pub artifacts: SortHeaderView,
}

// Why: one tool the project ran, with the share that did not succeed.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectToolRowView {
    pub server_name: String,
    pub tool_name: String,
    pub calls: i64,
    pub failures: i64,
    pub users: i64,
    pub error_pct: i64,
    pub tone: &'static str,
    pub p95_display: String,
}

// Why: one skill the project leant on, with what its users rated it.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectSkillRowView {
    pub skill: String,
    pub invocations: i64,
    pub users: i64,
    pub rating_display: String,
    pub rating_count: i64,
}

// Why: one session the project's people ran.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectSessionRowView {
    pub session_id: SessionId,
    pub session_short: String,
    pub href: String,
    pub user_id: UserId,
    // Why: the name a reader knows the person by; the id when no member row
    // carries one, so a printed report never shows an empty cell.
    pub person: String,
    pub requests: i64,
    pub models: i64,
    pub cost_display: String,
    pub last_activity: String,
}

// Why: one commit a project member landed in the window.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectCommitRowView {
    pub commit_short: String,
    pub user_id: UserId,
    pub branch: String,
    pub message: String,
    pub files_changed: i32,
    pub insertions: i32,
    pub deletions: i32,
    pub committed_at: String,
}

// Why: the Usage tab's body.
#[derive(Debug, Serialize)]
pub(crate) struct ProjectUsageTabView {
    pub daily: SvgLineChartView,
    pub daily_cost: SvgLineChartView,
    pub model_count: i64,
    pub skill_count: i64,
    pub tool_count: i64,
    pub session_count: i64,
    pub commit_count: i64,
    pub models: Vec<ModelMixRowView>,
    pub skills: Vec<ProjectSkillRowView>,
    pub tools: Vec<ProjectToolRowView>,
    pub sessions: Vec<ProjectSessionRowView>,
    pub commits: Vec<ProjectCommitRowView>,
    pub commit_files: i64,
    pub commit_insertions: i64,
    pub commit_deletions: i64,
}

// Why: the Members tab's body.
#[derive(Debug, Serialize)]
pub(crate) struct ProjectMembersTabView {
    pub count: i64,
    pub rows: Vec<MemberRowView>,
    pub addable_users: Vec<UserOptionView>,
}

// Why: the Settings tab's body.
#[derive(Debug, Serialize)]
pub(crate) struct ProjectSettingsTabView {
    pub name: String,
    pub description: String,
    pub source: String,
    pub feeding_groups: Vec<MappingRowView>,
    pub mapping_count: i64,
    pub gated_entities: Vec<AccessRowView>,
    pub gated_count: i64,
    pub can_map: bool,
    pub can_delete: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct ProjectDetailPageData {
    pub page: &'static str,
    pub title: String,
    pub project_id: ProjectId,
    pub project_name: String,
    pub description: Option<String>,
    pub window_label: String,
    pub can_manage: bool,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub tabs: Vec<TabLinkView>,
    pub active_tab: String,
    pub kpis: Vec<ProjectKpiView>,
    pub group_count: i64,
    pub groups_represented: Vec<MemberSetChipView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<ProjectMembersTabView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ProjectUsageTabView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<ProjectSettingsTabView>,
    pub export: crate::export::ExportView,
}
