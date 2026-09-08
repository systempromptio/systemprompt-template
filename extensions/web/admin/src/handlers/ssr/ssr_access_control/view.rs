//! Template context for `/admin/access-control`.
//!
//! The page is the rule ledger: the header counts, the filter state and the
//! paginated rules. `can_write` is the MANAGE tier and gates the two dialogs
//! that write — a project manager reads this page and cannot change it.

use serde::Serialize;

use crate::handlers::ssr::types::BreadcrumbView;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcStatsView {
    pub groups: usize,
    pub projects: usize,
    pub users: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct AccessControlPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub can_write: bool,
    pub stats: AcStatsView,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub ledger: super::rules::AcRulesView,
}
