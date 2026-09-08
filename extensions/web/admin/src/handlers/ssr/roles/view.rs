//! Template context for `/admin/roles`.
//!
//! The page renders two tables over one read: every person who holds a role,
//! with all of their roles as pills, and every entity a role opens. The role
//! card is a KPI tile that filters the people table rather than a container
//! the rows live inside, so a person holding three roles is one row with
//! three pills an operator can sort by cost, not three entries buried in
//! three cards.

use serde::Serialize;
use systemprompt::identifiers::UserId;

use crate::handlers::ssr::list_view::Pagination;
use crate::handlers::ssr::types::{BreadcrumbView, SortHeaderView};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RoleCardView {
    pub id: String,
    pub label: String,
    pub member_count: usize,
    pub manual_count: usize,
    pub directory_count: usize,
    pub entitlement_count: usize,
    pub tone: &'static str,
    pub href: String,
    pub active: bool,
    pub note: String,
}

// Why: the pill's tone is the grant source. A role granted by hand takes the
// role's own colour and offers a revoke; one the directory projected is grey
// and is withdrawn in the directory, so the table needs no source column.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct RolePillView {
    pub role: String,
    pub label: String,
    pub is_manual: bool,
    pub tone: &'static str,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RoleHolderView {
    pub user_id: UserId,
    pub display_name: String,
    pub email: String,
    pub initials: String,
    pub roles: Vec<RolePillView>,
    pub manual_roles: Vec<RolePillView>,
    pub has_manual: bool,
    pub is_active: bool,
    pub status: &'static str,
    pub status_tone: &'static str,
    pub requests_30d: i64,
    pub cost_30d_microdollars: i64,
    pub href: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RoleEntitlementView {
    pub role: String,
    pub role_label: String,
    pub entity_type: String,
    pub entity_type_label: String,
    pub entity_id: String,
    pub access: String,
    pub access_tone: &'static str,
    pub default_included: bool,
    pub default_label: &'static str,
    pub href: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SelectOption {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

// Why: the estate totals ride in the header's meta line rather than in a KPI
// band of their own. The band an operator acts on is the role tiles, and two
// stacked bands pushed the assignment table below the fold.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct HeaderFactView {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct RolesPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub can_write: bool,
    pub can_grant_platform_admin: bool,
    pub facts: Vec<HeaderFactView>,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub cards: Vec<RoleCardView>,
    pub members: Vec<RoleHolderView>,
    pub member_total: i64,
    pub entitlements: Vec<RoleEntitlementView>,
    pub entitlement_total: usize,
    pub pagination: Pagination,
    pub sort_headers: Vec<SortHeaderView>,
    pub role_options: Vec<SelectOption>,
    pub source_options: Vec<SelectOption>,
    pub status_options: Vec<SelectOption>,
    pub search: String,
    pub known_roles: Vec<SelectOption>,
    pub filters_applied: bool,
    pub clear_url: &'static str,
}
