//! Resolving the organization a listing may span for a given caller.
//!
//! `admin` is the role a customer's *own* administrator holds, so it cannot by
//! itself open a cross-customer listing on a pooled instance. Only a platform
//! admin sees every organization; everyone else is pinned to their own.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories::organizations::crud;
use crate::types::UserContext;

/// Whether a listing spans every organization or is pinned to one.
///
/// `AllOrganizations` is the cross-customer view and belongs to a platform
/// admin alone: `admin` is the role a customer's *own* administrator holds, so
/// a listing that reaches every organization by default would hand one
/// customer the shape of another. Naming the wide case rather than spelling it
/// as an absent filter keeps it out of reach of a caller who never considered
/// tenancy, which is why this type has no `Default`.
///
/// The payload is the key the query matches on — a slug for the organization
/// tables, an id where the row carries one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrgScope {
    AllOrganizations,
    Only(String),
}

impl OrgScope {
    // Why: The null-skip bind every scoped query takes. `AllOrganizations`
    // becomes `NULL`, which the `($n IS NULL OR …)` predicate reads as "do not
    // narrow"; `Only` becomes the key to match.
    #[must_use]
    pub const fn as_slug(&self) -> Option<&str> {
        match self {
            Self::AllOrganizations => None,
            Self::Only(key) => Some(key.as_str()),
        }
    }
}

pub async fn listing_scope(pool: &PgPool, user_ctx: &UserContext) -> OrgScope {
    if user_ctx.is_platform_admin {
        return OrgScope::AllOrganizations;
    }
    // Why: An unattached non-platform admin scopes to the empty slug, which
    // matches no organization, so they see nothing. Widening them to
    // `AllOrganizations` would hand the least-attached caller the widest view.
    OrgScope::Only(own_org_slug(pool, &user_ctx.user_id).await)
}

async fn own_org_slug(pool: &PgPool, user_id: &UserId) -> String {
    match crud::find_organization_for_user(pool, user_id).await {
        Ok(slug) => slug.unwrap_or_default(),
        Err(e) => {
            tracing::warn!(error = %e, "find_organization_for_user failed; scoping to no organization");
            String::new()
        },
    }
}

// Why: Whether `user_ctx` may act on `target`'s account.
//
// A platform admin may act on anyone. Everyone else may act only within their
// own organization, and only when both sides actually have one — two
// unattached users are not colleagues, so `(None, None)` is not a match.
pub async fn may_administer(pool: &PgPool, user_ctx: &UserContext, target: &UserId) -> bool {
    if user_ctx.is_platform_admin {
        return true;
    }
    let own = crud::find_organization_for_user(pool, &user_ctx.user_id)
        .await
        .unwrap_or_default();
    let theirs = crud::find_organization_for_user(pool, target)
        .await
        .unwrap_or_default();
    matches!((own, theirs), (Some(a), Some(b)) if a == b)
}
