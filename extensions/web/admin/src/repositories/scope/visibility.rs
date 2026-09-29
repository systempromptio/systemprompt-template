//! Resolving which users a listing may span for a given caller.
//!
//! Two layers. [`Visibility`] is what the caller is allowed to see and is a
//! pure function of their identity; [`ScopeRequest`] adds the group and
//! project filters a query string asked for. Resolving the pair against the
//! database yields a [`SubjectScope`] — either every user, or an explicit id
//! list — which is what scoped queries bind.
//!
//! Narrowing to a user id list rather than to a column keeps the rule in one
//! place: a listing joins whatever it likes and adds one `user_id = ANY($n)`,
//! instead of every query restating what membership means.

use crate::types::UserContext;

/// What a caller may see before any query filter is applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    All,
    Groups(Vec<String>),
}

impl Visibility {
    #[must_use]
    pub fn for_user(user_ctx: &UserContext) -> Self {
        if user_ctx.is_console {
            Self::All
        } else {
            Self::Groups(user_ctx.group_ids.clone())
        }
    }
}

/// A listing's resolved scope: what the caller may see, narrowed by what they
/// asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeRequest {
    pub visibility: Visibility,
    pub group: Option<String>,
    pub project: Option<String>,
}

impl ScopeRequest {
    // Why: Why: a requested group the caller is not in is ignored rather than
    // refused. The query string is not an authorisation surface, so a
    // non-console caller asking for someone else's group gets their own view
    // back, never an error that would tell them the group exists.
    #[must_use]
    pub fn from_query(user_ctx: &UserContext, group: Option<&str>, project: Option<&str>) -> Self {
        let visibility = Visibility::for_user(user_ctx);
        let group = match (&visibility, group) {
            (_, None) => None,
            (Visibility::All, Some(g)) => Some(g.to_owned()),
            (Visibility::Groups(own), Some(g)) => own.iter().find(|o| o.as_str() == g).cloned(),
        };
        Self {
            visibility,
            group,
            project: project.map(ToOwned::to_owned),
        }
    }

    // Why: The group ids a query must narrow to, or `None` for every group.
    #[must_use]
    pub fn group_filter(&self) -> Option<Vec<String>> {
        match (&self.visibility, self.group.as_ref()) {
            (Visibility::All, None) => None,
            (Visibility::All, Some(group)) => Some(vec![group.clone()]),
            (Visibility::Groups(own), None) => Some(own.clone()),
            (Visibility::Groups(own), Some(group)) => {
                Some(own.iter().filter(|o| *o == group).cloned().collect())
            },
        }
    }
}

/// The users a scoped query may return.
///
/// `All` binds NULL, which every scoped predicate reads as "do not narrow".
/// `Users` binds the id list, and an empty list is a legitimate answer that
/// matches nothing — the least-attached caller must not widen to everything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubjectScope {
    All,
    Users(Vec<String>),
}

impl SubjectScope {
    #[must_use]
    pub const fn as_sql(&self) -> Option<&[String]> {
        match self {
            Self::All => None,
            Self::Users(ids) => Some(ids.as_slice()),
        }
    }
}

// Why: whether one owner's rows are inside the caller's view — the question a
// detail page asks before rendering a session, context, trace or request the
// header lookup landed on. A console caller sees every owner; a scoped caller
// resolves their unfiltered view once and checks membership. An unknown owner
// is outside every scoped view.
pub async fn may_view(
    pool: &sqlx::PgPool,
    user_ctx: &UserContext,
    owner: Option<&systemprompt::identifiers::UserId>,
) -> Result<bool, sqlx::Error> {
    if user_ctx.is_console {
        return Ok(true);
    }
    let Some(owner) = owner else {
        return Ok(false);
    };
    let request = ScopeRequest::from_query(user_ctx, None, None);
    let scope = super::membership::get_subject_scope(pool, &request).await?;
    Ok(match scope {
        SubjectScope::All => true,
        SubjectScope::Users(ids) => ids.iter().any(|id| id == owner.as_str()),
    })
}
