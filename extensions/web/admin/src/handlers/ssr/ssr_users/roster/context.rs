//! Typed view model for the `/admin/users` roster.
//!
//! Strict Handlebars turns an absent field into a 500, so every name the
//! template reads is a field here and the two move together.

use serde::Serialize;
use systemprompt::identifiers::UserId;

use crate::handlers::ssr::list_view::{Pagination, ScopeFilterView};
use crate::handlers::ssr::types::{BreadcrumbView, FilterChipView, SortHeaderView};

#[derive(Debug, Serialize)]
pub(crate) struct RosterContext {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,

    pub kpis: RosterKpiView,
    pub chips: Vec<FilterChipView>,
    pub role_options: Vec<RoleOptionView>,
    pub search: String,
    pub scope_filter: ScopeFilterView,

    pub sort_headers: Vec<SortHeaderView>,
    pub rows: Vec<RosterRowView>,
    pub has_rows: bool,
    pub row_count: usize,
    pub pagination: Pagination,

    // Why: the write controls. A project manager reads the roster but may not
    // create an account or change a role, so the buttons are absent rather
    // than present and refused.
    pub can_write: bool,
    pub role_choices: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct RosterKpiView {
    pub total: i64,
    pub active: i64,
    pub unassigned: i64,
    pub no_role: i64,
    pub inactive: i64,
    pub cost_display: String,
    pub cost_delta: String,
    pub cost_delta_dir: &'static str,
    pub requests_display: String,
    pub requests_delta: String,
    pub requests_delta_dir: &'static str,
    pub unassigned_url: String,
    pub no_role_url: String,
    pub inactive_url: String,
    pub all_url: String,
    pub filter_active: bool,
    pub unassigned_active: bool,
    pub no_role_active: bool,
    pub inactive_active: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct RoleOptionView {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct RosterRowView {
    pub user_id: UserId,
    pub name: String,
    pub initials: String,
    pub email: String,
    pub detail_url: String,
    pub roles: Vec<String>,
    pub roles_title: String,
    pub has_roles: bool,
    // Why: groups and projects share one cell. Two columns of chips is what
    // pushed a 32px row to 80px, and the question they answer together —
    // "what does this person belong to" — is one question.
    pub scope: Vec<RosterChipView>,
    pub has_scope: bool,
    pub last_active: Option<String>,
    pub last_active_title: String,
    // Why: the conversation the stamp came from, when it was a gateway turn.
    pub last_active_url: Option<String>,
    pub cost_display: String,
    pub requests_display: String,
    pub tokens_display: String,
    pub is_active: bool,
    pub status_label: &'static str,
    pub status_tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct RosterChipView {
    pub id: String,
    pub label: String,
    pub href: String,
    pub tone: &'static str,
}
