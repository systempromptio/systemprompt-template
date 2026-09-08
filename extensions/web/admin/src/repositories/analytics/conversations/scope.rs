//! Who a principal may view conversation history for.
//!
//! Self always; the `admin` and `auditor` roles keep the unrestricted view.
//! Resolution is a pure function over the request context, so the rule is
//! pinned by unit tests without a database.

use systemprompt::identifiers::UserId;

use crate::types::UserContext;

/// The set of user ids a viewer's history queries are constrained to.
/// `All` is the admin/auditor view with no constraint; `Users` is everyone
/// else's explicit allowlist, which always contains the viewer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryScope {
    All,
    Users(Vec<UserId>),
}

impl HistoryScope {
    #[must_use]
    pub fn may_view(&self, target: &UserId) -> bool {
        match self {
            Self::All => true,
            Self::Users(ids) => ids.iter().any(|id| id == target),
        }
    }

    // Why: `None`, not an empty Vec, is the unrestricted case — the SQL layer
    // binds it as a NULL text[] and skips the scope predicate entirely.
    #[must_use]
    pub fn user_ids(&self) -> Option<Vec<String>> {
        match self {
            Self::All => None,
            Self::Users(ids) => Some(ids.iter().map(|id| id.as_str().to_owned()).collect()),
        }
    }
}

#[must_use]
pub fn resolve_history_scope(
    viewer: &UserId,
    has_full_view: bool,
    managed_member_ids: Vec<UserId>,
) -> HistoryScope {
    if has_full_view {
        return HistoryScope::All;
    }
    let mut ids = managed_member_ids;
    if !ids.iter().any(|id| id == viewer) {
        ids.push(viewer.clone());
    }
    HistoryScope::Users(ids)
}

#[must_use]
pub fn has_full_history_view(ctx: &UserContext) -> bool {
    ctx.is_admin || ctx.roles.iter().any(|r| r.eq_ignore_ascii_case("auditor"))
}

// Why: no manager edge exists any more — a non-admin sees exactly themselves.
#[must_use]
pub fn history_scope_for(ctx: &UserContext) -> HistoryScope {
    resolve_history_scope(&ctx.user_id, has_full_history_view(ctx), Vec::new())
}
