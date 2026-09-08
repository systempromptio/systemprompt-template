//! Who holds a role, and how much they used the platform holding it.
//!
//! One row per person, carrying every known role they hold. The manual
//! subset is the same distinction `users::roles` draws: a role with a
//! `user_manual_roles` row was granted by hand and can be revoked on the roles
//! page, and one without comes from the directory and cannot.

use sqlx::PgPool;
use systemprompt::identifiers::{Email, UserId};

/// One person and the roles they hold.
#[derive(Debug, Clone)]
pub struct RoleHolderRow {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub email: Option<Email>,
    pub is_active: bool,
    pub roles: Vec<String>,
    pub manual_roles: Vec<String>,
    pub requests_30d: i64,
    pub cost_30d_microdollars: i64,
}

// Why: the caller passes the role vocabulary rather than the query hardcoding
// it, because `users.roles` is a free-text array an older release may have
// written into and a stale value must not invent a role card.
pub async fn list_role_holders(
    pool: &PgPool,
    roles: &[String],
    limit: i64,
) -> Result<Vec<RoleHolderRow>, sqlx::Error> {
    sqlx::query_as!(
        RoleHolderRow,
        r#"SELECT
                u.id AS "user_id!: UserId",
                COALESCE(u.display_name, u.full_name, u.name) AS display_name,
                u.email AS "email?: Email",
                (u.status = 'active') AS "is_active!",
                r.roles AS "roles!: Vec<String>",
                r.manual_roles AS "manual_roles!: Vec<String>",
                COALESCE(a.requests, 0)::BIGINT AS "requests_30d!",
                COALESCE(a.cost, 0)::BIGINT AS "cost_30d_microdollars!"
           FROM users u
           CROSS JOIN LATERAL (
                SELECT ARRAY_AGG(x ORDER BY x) AS roles,
                       ARRAY_REMOVE(
                           ARRAY_AGG(CASE WHEN m.user_id IS NOT NULL THEN x END ORDER BY x),
                           NULL
                       ) AS manual_roles
                  FROM UNNEST(u.roles) AS x
                  LEFT JOIN user_manual_roles m ON m.user_id = u.id AND m.role = x
                 WHERE x = ANY($1::TEXT[])
           ) r
           LEFT JOIN LATERAL (
                SELECT COUNT(*) AS requests,
                       SUM(ar.cost_microdollars) AS cost
                FROM ai_requests ar
                WHERE ar.user_id = u.id
                  AND ar.created_at >= NOW() - INTERVAL '30 days'
           ) a ON TRUE
           WHERE r.roles IS NOT NULL
           ORDER BY LOWER(COALESCE(u.email, u.id))
           LIMIT $2"#,
        roles,
        limit,
    )
    .fetch_all(pool)
    .await
}
