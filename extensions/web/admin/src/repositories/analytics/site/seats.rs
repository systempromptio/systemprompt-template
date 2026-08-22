//! Wasted seats: active members with no gateway requests in a configurable
//! window.
//!
//! The window used to be an `INTERVAL '30 days'` literal in both queries below
//! and in the template copy beside them. REQ-005 asks for "a configurable
//! period such as 30 days", so it is a parameter — bound as a day count and
//! multiplied into an interval, because a caller-supplied interval *string*
//! would be an injection seam for no benefit.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::SiteScope;

#[derive(Debug, Clone)]
pub struct InactiveSeatRow {
    pub user_id: UserId,
    pub label: String,
    pub email: String,
    pub department: String,
    pub org_name: String,
    pub last_request_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub const DEFAULT_INACTIVE_DAYS: i32 = 30;

// Why: The user filter is deliberately not applied — a single user's seat
// status is visible on their detail page.
pub async fn list_inactive_seats(
    pool: &PgPool,
    scope: &SiteScope,
    inactive_days: i32,
) -> Result<Vec<InactiveSeatRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            u.id AS "user_id!: UserId",
            COALESCE(u.display_name, u.full_name, u.name, u.email) AS "label!",
            u.email AS "email!",
            COALESCE(NULLIF(upe.department, ''), 'Default') AS "department!",
            o.name AS "org_name!",
            last.last_request_at AS "last_request_at?"
        FROM organization_members m
        JOIN users u ON u.id = m.user_id AND u.status = 'active'
        JOIN organizations o ON o.id = m.org_id
        LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        LEFT JOIN LATERAL (
            SELECT MAX(r.created_at) AS last_request_at
            FROM ai_requests r
            WHERE r.user_id = u.id AND NOT r.synthetic
        ) last ON TRUE
        WHERE ($1::TEXT IS NULL OR o.slug = $1)
          AND ($2::TEXT IS NULL OR COALESCE(NULLIF(upe.department, ''), 'Default') = $2)
          AND (last.last_request_at IS NULL
               OR last.last_request_at < NOW() - make_interval(days => $3))
        ORDER BY last.last_request_at ASC NULLS FIRST, u.email
        "#,
        scope.org_slug.as_slug(),
        scope.department.as_deref(),
        inactive_days,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| InactiveSeatRow {
            user_id: r.user_id,
            label: r.label,
            email: r.email,
            department: r.department,
            org_name: r.org_name,
            last_request_at: r.last_request_at,
        })
        .collect())
}

pub async fn count_inactive_seats(
    pool: &PgPool,
    scope: &SiteScope,
    inactive_days: i32,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::BIGINT AS "count!"
        FROM organization_members m
        JOIN users u ON u.id = m.user_id AND u.status = 'active'
        JOIN organizations o ON o.id = m.org_id
        LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        WHERE ($1::TEXT IS NULL OR o.slug = $1)
          AND ($2::TEXT IS NULL OR COALESCE(NULLIF(upe.department, ''), 'Default') = $2)
          AND NOT EXISTS (
            SELECT 1 FROM ai_requests r
            WHERE r.user_id = u.id AND NOT r.synthetic
              AND r.created_at >= NOW() - make_interval(days => $3)
          )
        "#,
        scope.org_slug.as_slug(),
        scope.department.as_deref(),
        inactive_days,
    )
    .fetch_one(pool)
    .await
}
