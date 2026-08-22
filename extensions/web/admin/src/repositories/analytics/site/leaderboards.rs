//! Top-users leaderboard for the dashboard's usage tab.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::util::time_range::TimeRange;

use super::SiteScope;

/// Sort keys the page offers; anything unrecognised falls back to requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LeaderboardSort {
    #[default]
    Requests,
    Cost,
    Tokens,
    LastActive,
}

impl LeaderboardSort {
    #[must_use]
    pub fn from_sort_param(raw: Option<&str>) -> Self {
        match raw {
            Some("cost") => Self::Cost,
            Some("tokens") => Self::Tokens,
            Some("last_active") => Self::LastActive,
            _ => Self::Requests,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Requests => "requests",
            Self::Cost => "cost",
            Self::Tokens => "tokens",
            Self::LastActive => "last_active",
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserUsageRow {
    pub user_id: UserId,
    pub label: String,
    pub department: String,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub last_active: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Copy)]
pub struct LeaderboardPage {
    pub sort: LeaderboardSort,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list_top_users_by_requests(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
    page: LeaderboardPage,
) -> Result<(Vec<UserUsageRow>, i64), sqlx::Error> {
    // Why: the sort key is compiled into one static query per variant rather
    // than interpolated — sqlx macros check each, and a query string never
    // carries user input.
    let sort_key: i32 = match page.sort {
        LeaderboardSort::Requests => 0,
        LeaderboardSort::Cost => 1,
        LeaderboardSort::Tokens => 2,
        LeaderboardSort::LastActive => 3,
    };
    let rows = sqlx::query!(
        r#"
        SELECT
            r.user_id AS "user_id!: UserId",
            COALESCE(u.display_name, u.full_name, u.name, u.email, r.user_id) AS "label!",
            COALESCE(NULLIF(upe.department, ''), 'Default') AS "department!",
            COUNT(*)::BIGINT AS "requests!",
            COALESCE(SUM(r.input_tokens + r.output_tokens), 0)::BIGINT AS "tokens!",
            COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost!",
            MAX(r.created_at) AS "last_active?",
            COUNT(*) OVER ()::BIGINT AS "total_users!"
        FROM ai_requests r
        LEFT JOIN users u ON u.id = r.user_id
        LEFT JOIN organization_members m ON m.user_id = r.user_id
        LEFT JOIN organizations o ON o.id = m.org_id
        LEFT JOIN user_profile_ext upe ON upe.user_id = r.user_id
        WHERE r.created_at >= $1 AND r.created_at < $2
          AND NOT r.synthetic
          AND ($3::TEXT IS NULL OR o.slug = $3)
          AND ($4::TEXT IS NULL OR COALESCE(NULLIF(upe.department, ''), 'Default') = $4)
          AND ($5::TEXT IS NULL OR r.user_id = $5)
        GROUP BY r.user_id, u.display_name, u.full_name, u.name, u.email, upe.department
        ORDER BY
            CASE WHEN $6 = 0 THEN COUNT(*) END DESC,
            CASE WHEN $6 = 1 THEN COALESCE(SUM(r.cost_microdollars), 0) END DESC,
            CASE WHEN $6 = 2 THEN COALESCE(SUM(r.input_tokens + r.output_tokens), 0) END DESC,
            CASE WHEN $6 = 3 THEN MAX(r.created_at) END DESC,
            COUNT(*) DESC
        LIMIT $7 OFFSET $8
        "#,
        range.from,
        range.to,
        scope.org_slug.as_deref(),
        scope.department.as_deref(),
        scope.user_id_str(),
        sort_key,
        page.limit,
        page.offset,
    )
    .fetch_all(pool)
    .await?;

    let total = rows.first().map_or(0, |r| r.total_users);
    let users = rows
        .into_iter()
        .map(|r| UserUsageRow {
            user_id: r.user_id,
            label: r.label,
            department: r.department,
            requests: r.requests,
            tokens: r.tokens,
            cost_microdollars: r.cost,
            last_active: r.last_active,
        })
        .collect();
    Ok((users, total))
}
