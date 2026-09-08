//! What a scoped admin query is asked about, and how usage is attributed.
//!
//! Three things pin a scoped query: a [`Scope`] (everyone, one group, one
//! project, one person), an [`Attribution`] (whether a person counts once or
//! in every container they belong to), and a time range — `TimeRange` from
//! `util::time_range`, which every audit and analytics page already parses.
//!
//! [`Visibility`] and [`ScopeRequest`] are the caller-facing half: what an
//! identity may see, narrowed by what a query string asked for. Resolving them
//! yields a [`SubjectScope`], the user id list a listing binds. The membership
//! half lives in [`membership`], which owns the single CTE every container
//! query is built on, and [`defaults`] owns the primary group and project that
//! exclusive attribution reads.

pub mod defaults;
pub mod membership;
pub mod visibility;

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

pub use visibility::{ScopeRequest, SubjectScope, Visibility};

/// Which container a usage query names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Group,
    Project,
}

impl ScopeKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Group => "group",
            Self::Project => "project",
        }
    }

    // Why: the overlap question is symmetric — a group's projects and a
    // project's groups are the same join read from either side.
    #[must_use]
    pub const fn linked(self) -> Self {
        match self {
            Self::Group => Self::Project,
            Self::Project => Self::Group,
        }
    }
}

/// The people a scoped query spans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    All,
    Group(String),
    Project(String),
    User(UserId),
}

impl Scope {
    #[must_use]
    pub const fn kind(&self) -> Option<ScopeKind> {
        match self {
            Self::Group(_) => Some(ScopeKind::Group),
            Self::Project(_) => Some(ScopeKind::Project),
            Self::All | Self::User(_) => None,
        }
    }

    #[must_use]
    pub const fn container_id(&self) -> Option<&str> {
        match self {
            Self::Group(id) | Self::Project(id) => Some(id.as_str()),
            Self::All | Self::User(_) => None,
        }
    }

    pub async fn resolve(
        &self,
        pool: &PgPool,
        attribution: Attribution,
    ) -> Result<SubjectScope, sqlx::Error> {
        match (self.kind(), self.container_id()) {
            (Some(kind), Some(id)) => {
                let ids = membership::list_scope_user_ids(pool, kind, attribution, id).await?;
                Ok(SubjectScope::Users(
                    ids.into_iter()
                        .map(|user_id| user_id.as_str().to_owned())
                        .collect(),
                ))
            },
            _ => match self {
                Self::User(user_id) => Ok(SubjectScope::Users(vec![user_id.as_str().to_owned()])),
                _ => Ok(SubjectScope::All),
            },
        }
    }
}

/// One scoped question: which container, attributed how, over how long.
///
/// The trailing-days window is what the people pages ask for; a page working
/// from absolute bounds parses a
/// [`TimeRange`](crate::util::time_range::TimeRange) and narrows the rows it
/// selects itself.
#[derive(Debug, Clone, Copy)]
pub struct ScopeQuery<'a> {
    pub kind: ScopeKind,
    pub attribution: Attribution,
    pub id: &'a str,
    pub window_days: i32,
}

impl<'a> ScopeQuery<'a> {
    #[must_use]
    pub const fn new(
        kind: ScopeKind,
        attribution: Attribution,
        id: &'a str,
        window_days: i32,
    ) -> Self {
        Self {
            kind,
            attribution,
            id,
            window_days,
        }
    }
}

/// Whether a person's usage lands in one container or in all of them.
///
/// `Exclusive` reads each person's primary container, so container totals
/// partition the instance and sum back to it. `Member` reads full membership,
/// so a person in two containers counts in full in both and the totals
/// deliberately overlap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attribution {
    Exclusive,
    Member,
}

impl Attribution {
    #[must_use]
    pub const fn is_exclusive(self) -> bool {
        matches!(self, Self::Exclusive)
    }
}
