//! The paged, sorted, filtered user roster behind `/admin/users`.
//!
//! One statement answers the whole page: the row set, every row's group and
//! project membership, its thirty-day gateway spend, and the total the
//! pagination footer needs, so a fifty-row page is one round trip rather than
//! fifty-one.
//!
//! Sorting and filtering are bound parameters, never interpolated text. The
//! `ORDER BY` is a fixed ladder of `CASE` arms — one pair per sortable column —
//! so the statement stays a static string the `sqlx` macros verify against the
//! live schema, and no caller can reach the query planner with a column name.
//!
//! "Unassigned" is a filter here, not a group: the `user_groups` view already
//! synthesises an `unassigned` row for an account no `group_members` row
//! covers, so the filter matches that derived id as well as a genuinely empty
//! membership.

mod stats;

pub use stats::{RosterStats, get_roster_stats};

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, Email, UserId};

use crate::repositories::scope::SubjectScope;

// Why: how many rows a roster page carries unless the reader asks for another
// size.
pub const DEFAULT_PAGE_SIZE: i64 = 50;

// Why: the window every cost and request figure on the roster is measured over.
pub const WINDOW_DAYS: i32 = 30;

/// One roster row: who they are, what they belong to, what they spent.
#[derive(Debug, Clone, Serialize)]
pub struct RosterRow {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub email: Option<Email>,
    pub roles: Vec<String>,
    pub group_ids: Vec<String>,
    pub project_ids: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    // Why: `None` is "never seen", not "seen when they joined" — the wasted-seat
    // question the roster exists to answer needs those two apart. It is the
    // latest of gateway traffic, session activity, and console activity —
    // the same three terms `stats.rs` uses for the inactive-30d chip.
    pub last_active: Option<DateTime<Utc>>,
    // Why: "gateway" | "session" | "console" — which of the three terms won.
    pub last_active_source: Option<String>,
    // Why: the conversation of the user's latest turn, so the row can link to
    // what they were last doing rather than to a bare timestamp.
    pub last_context_id: Option<ContextId>,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
}

/// The chips above the roster. Each one is a `WHERE` arm, not a saved search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RosterFilter {
    None,
    Unassigned,
    NoRole,
    Inactive30d,
}

impl RosterFilter {
    #[must_use]
    pub fn parse_filter(raw: Option<&str>) -> Self {
        match raw.unwrap_or_default() {
            "unassigned" => Self::Unassigned,
            "no-role" => Self::NoRole,
            "inactive-30d" => Self::Inactive30d,
            _ => Self::None,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Unassigned => "unassigned",
            Self::NoRole => "no-role",
            Self::Inactive30d => "inactive-30d",
        }
    }
}

/// Which column the roster is ordered by. Parsed from `?sort=`, so an unknown
/// value falls back to the default rather than erroring the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RosterSort {
    pub column: &'static str,
    pub descending: bool,
}

impl RosterSort {
    // Why: every column the header row offers, in table order.
    pub const COLUMNS: [&'static str; 9] = [
        "name", "email", "roles", "groups", "projects", "seen", "cost", "requests", "status",
    ];

    #[must_use]
    pub fn parse_sort(column: Option<&str>, direction: Option<&str>) -> Self {
        let column = column
            .and_then(|c| Self::COLUMNS.into_iter().find(|known| *known == c))
            .unwrap_or("cost");
        Self {
            column,
            descending: direction.unwrap_or("desc") != "asc",
        }
    }
}

impl Default for RosterSort {
    fn default() -> Self {
        Self {
            column: "cost",
            descending: true,
        }
    }
}

/// What the roster asks for: which people, narrowed how, ordered how, which
/// page.
#[derive(Debug, Clone)]
pub struct RosterQuery {
    pub filter: RosterFilter,
    pub role: Option<String>,
    pub search: Option<String>,
    pub sort: RosterSort,
    pub limit: i64,
    pub offset: i64,
}

// Why: The window function carries the filtered total on every row, so the
// count and the page come from one scan of one predicate — there is no second
// statement to drift out of step with the first.
pub async fn list_users_paged(
    pool: &PgPool,
    scope: &SubjectScope,
    query: &RosterQuery,
) -> Result<(Vec<RosterRow>, i64), sqlx::Error> {
    let mut transaction = crate::repositories::dashboard_read::begin(pool).await?;
    let rows = sqlx::query_file!(
        "src/repositories/users/roster/page.sql",
        scope.as_sql(),
        WINDOW_DAYS,
        query.filter.as_str(),
        query.role.as_deref(),
        query.search.as_deref(),
        query.sort.column,
        query.sort.descending,
        query.limit,
        query.offset,
    )
    .fetch_all(&mut *transaction)
    .await?;

    transaction.commit().await?;
    let total = if let Some(row) = rows.first() {
        row.total_rows
    } else if query.offset > 0 {
        stats::count_filtered_users(pool, scope, query).await?
    } else {
        0
    };
    Ok((
        rows.into_iter()
            .map(|row| RosterRow {
                user_id: row.user_id,
                display_name: row.display_name,
                email: row.email,
                roles: row.roles,
                group_ids: row.group_ids,
                project_ids: row.project_ids,
                is_active: row.is_active,
                created_at: row.created_at,
                last_active: row.last_active,
                last_active_source: row.last_active_source,
                last_context_id: row.last_context_id,
                requests: row.requests,
                tokens: row.tokens,
                cost_microdollars: row.cost_microdollars,
            })
            .collect(),
        total,
    ))
}
