//! Which of the two conversation listings is rendering.
//!
//! `/admin/history` and `/admin/conversations` are one page at two scopes, and
//! everything that differs between them is a method on this enum — the base
//! url every query link is built against, the template, the page id the
//! sidebar highlights, and which `HistoryScope` the rows are fetched under.

use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::analytics::conversations::{
    HistoryScope, history_scope_for, own_history_scope,
};
use crate::types::UserContext;

// Why: which of the two listings is rendering, and everything that differs
// between them — `Own` is `/admin/history`, `Org` the admin-gated
// `/admin/conversations`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HistoryView {
    Own,
    Org,
}

impl HistoryView {
    pub(super) const fn base_url(self) -> &'static str {
        match self {
            Self::Own => "/admin/history",
            Self::Org => "/admin/conversations",
        }
    }

    pub(super) const fn page_id(self) -> &'static str {
        match self {
            Self::Own => "history",
            Self::Org => "conversations",
        }
    }

    pub(super) const fn title(self) -> &'static str {
        match self {
            Self::Own => "My Conversations",
            Self::Org => "Conversations",
        }
    }

    pub(super) const fn template(self) -> &'static str {
        match self {
            Self::Own => "history",
            Self::Org => "conversations",
        }
    }

    pub(super) fn scope(self, ctx: &UserContext) -> HistoryScope {
        match self {
            Self::Own => own_history_scope(ctx),
            Self::Org => history_scope_for(ctx),
        }
    }

    pub(super) fn breadcrumbs(self) -> Vec<BreadcrumbView> {
        match self {
            Self::Own => vec![
                BreadcrumbView::link("Account", "/admin/profile"),
                BreadcrumbView::current("My conversations"),
            ],
            Self::Org => vec![BreadcrumbView::current("Conversations")],
        }
    }
}
