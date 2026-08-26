//! Who a principal may view conversation history for.
//!
//! Self always; an organization owner/admin additionally sees the members of
//! their organization(s); the `admin` and `auditor` roles keep the existing
//! unrestricted view. Resolution is split so the decision itself is a pure
//! function over already-fetched facts and unit-testable without a database.

use sqlx::PgPool;
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

// Why: org owner/admin IS the v1 "manager" edge — members of the active
// organizations the viewer holds an `owner` or `admin` seat-role in.
pub async fn list_managed_member_ids(
    pool: &PgPool,
    viewer: &UserId,
) -> Result<Vec<UserId>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT m2.user_id AS "user_id!: UserId"
           FROM organization_members m1
           JOIN organizations o ON o.id = m1.org_id AND o.status = 'active'
           JOIN organization_members m2 ON m2.org_id = m1.org_id
           WHERE m1.user_id = $1 AND m1.org_role IN ('owner', 'admin')"#,
        viewer.as_str(),
    )
    .fetch_all(pool)
    .await
}

pub async fn history_scope_for(
    pool: &PgPool,
    ctx: &UserContext,
) -> Result<HistoryScope, sqlx::Error> {
    if has_full_history_view(ctx) {
        return Ok(HistoryScope::All);
    }
    let managed = list_managed_member_ids(pool, &ctx.user_id).await?;
    Ok(resolve_history_scope(&ctx.user_id, false, managed))
}
