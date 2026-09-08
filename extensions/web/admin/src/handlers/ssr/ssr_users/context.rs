//! Typed view models for the `/admin/users` roster and `/admin/user` detail.
//!
//! Strict Handlebars turns an absent top-level field into a 500, so every name
//! the two templates read is a field here and the two move together. The
//! detail context carries every tab, but only the active tab's option is
//! populated: the handler loads what the reader asked for and nothing else.

use serde::Serialize;
use systemprompt::identifiers::{SessionId, UserId};

use crate::handlers::ssr::list_view::{Pagination, SelectOptionView};
use crate::handlers::ssr::types::{
    BreadcrumbView, FilterChipView, SortHeaderView, TabLinkView, UserAssignmentSummary,
    UserMarketplaceRef, UserRuntimeView, UserTokenView,
};
use crate::repositories::governance::effective::EffectivePermissions;

#[derive(Debug, Serialize)]
pub(crate) struct RosterContext {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub kpis: RosterKpiView,
    pub chips: Vec<FilterChipView>,
    pub role_options: Vec<SelectOptionView>,
    pub department_options: Vec<SelectOptionView>,
    pub search: String,
    pub sort_headers: Vec<SortHeaderView>,
    pub rows: Vec<RosterRowView>,
    pub has_rows: bool,
    pub row_total: i64,
    pub pagination: Pagination,
    pub filters_applied: bool,
    pub clear_url: &'static str,
    // Why: the write controls. Only an admin may create an account, so the
    // button is absent rather than present and refused.
    pub can_write: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct RosterKpiView {
    pub total: i64,
    pub active: i64,
    pub inactive: i64,
    pub no_role: i64,
    pub departments: i64,
    pub model_tokens_display: String,
    pub all_url: String,
    pub inactive_url: String,
    pub no_role_url: String,
    pub filter_active: bool,
    pub inactive_active: bool,
    pub no_role_active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RosterRowView {
    pub user_id: UserId,
    pub name: String,
    pub initials: String,
    pub email: String,
    pub detail_url: String,
    pub roles: Vec<String>,
    pub has_roles: bool,
    pub department: String,
    pub department_url: String,
    pub marketplaces_count: usize,
    pub tokens_count: i64,
    pub token_tone: &'static str,
    pub model_tokens_display: String,
    // Why: the raw values the sort ladder compares; the display strings beside
    // them are what the table shows and sort as text.
    pub model_tokens_raw: i64,
    pub last_active_epoch: i64,
    pub sessions: i64,
    pub last_active: String,
    pub last_active_title: String,
    pub is_active: bool,
    pub status_label: &'static str,
    pub status_tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct UserDetailContext {
    pub page: &'static str,
    pub title: String,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub found: bool,
    pub missing_message: &'static str,
    pub tabs: Vec<TabLinkView>,
    pub tab: &'static str,
    pub header: Option<UserHeaderView>,
    pub kpis: Option<DetailKpiView>,
    pub can_write: bool,
    pub identity: Option<IdentityTabView>,
    pub access: Option<AccessTabView>,
    pub activity: Option<ActivityTabView>,
}

#[derive(Debug, Serialize)]
pub(crate) struct UserHeaderView {
    pub user_id: UserId,
    pub name: String,
    pub email: String,
    pub created_at: String,
    pub last_active: String,
    pub status_label: &'static str,
    pub status_tone: &'static str,
    pub permissions_url: String,
    pub requests_url: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct DetailKpiView {
    pub requests_display: String,
    pub tokens_in_display: String,
    pub tokens_out_display: String,
    pub last_request: String,
    pub events_display: String,
    pub tokens_count: i64,
}

#[derive(Debug, Serialize)]
pub(crate) struct RoleChoiceView {
    pub id: String,
    pub held: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct IdentityTabView {
    pub display_name: String,
    pub email: String,
    pub is_active: bool,
    pub department: String,
    pub department_options: Vec<SelectOptionView>,
    pub role_choices: Vec<RoleChoiceView>,
    pub assignments: UserAssignmentSummary,
    pub has_marketplaces: bool,
    pub marketplaces: Vec<UserMarketplaceRef>,
    pub matrix_url: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AccessTabView {
    pub effective: EffectivePermissions,
    pub has_gateway_routes: bool,
    pub has_mcp_servers: bool,
    pub tokens: Vec<UserTokenView>,
    pub has_tokens: bool,
    pub tokens_url: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct ActivityTabView {
    pub runtime: UserRuntimeView,
    pub categories: Vec<CategoryRowView>,
    pub has_categories: bool,
    pub tools: Vec<ToolRowView>,
    pub has_tools: bool,
    pub sessions: Vec<SessionRowView>,
    pub has_sessions: bool,
    pub events: Vec<EventRowView>,
    pub has_events: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct CategoryRowView {
    pub category: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub(crate) struct ToolRowView {
    pub tool_name: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub(crate) struct SessionRowView {
    pub session_id: SessionId,
    pub short_id: String,
    pub detail_url: String,
    pub started_at: String,
    pub total_events: i64,
    pub tool_uses: i64,
    pub prompts: i64,
    pub errors: i64,
    pub error_tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct EventRowView {
    pub category: String,
    pub description: String,
    pub created_at: String,
    pub created_at_title: String,
}
