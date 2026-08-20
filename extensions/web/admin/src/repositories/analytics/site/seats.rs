//! Wasted seats: active members with no gateway requests in 30 days.

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
    /// `None` means the user has never made a gateway request.
    pub last_request_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Active members with no `ai_requests` row in 30 days, never-used first.
///
/// The user filter is deliberately not applied — a single user's seat status
/// is visible on their detail page.
pub async fn list_inactive_seats(
    pool: &PgPool,
    scope: &SiteScope,
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
               OR last.last_request_at < NOW() - INTERVAL '30 days')
        ORDER BY last.last_request_at ASC NULLS FIRST, u.email
        "#,
        scope.org_slug.as_deref(),
        scope.department.as_deref(),
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

pub async fn count_inactive_seats(pool: &PgPool, scope: &SiteScope) -> Result<i64, sqlx::Error> {
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
              AND r.created_at >= NOW() - INTERVAL '30 days'
          )
        "#,
        scope.org_slug.as_deref(),
        scope.department.as_deref(),
    )
    .fetch_one(pool)
    .await
}
