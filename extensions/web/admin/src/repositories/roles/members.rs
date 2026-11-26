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
    #[cfg(feature = "gateway")]
    pub requests_30d: i64,
    #[cfg(feature = "gateway")]
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
    let rows = sqlx::query!(
        r#"SELECT
                u.id AS "user_id!: UserId",
                COALESCE(u.display_name, u.full_name, u.name) AS display_name,
                u.email AS "email?: Email",
                (u.status = 'active') AS "is_active!",
                r.roles AS "roles!: Vec<String>",
                r.manual_roles AS "manual_roles!: Vec<String>"
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
           WHERE r.roles IS NOT NULL
           ORDER BY LOWER(COALESCE(u.email, u.id))
           LIMIT $2"#,
        roles,
        limit,
    )
    .fetch_all(pool)
    .await?;
    #[cfg(feature = "gateway")]
    let spend = list_spend_30d(
        pool,
        &rows
            .iter()
            .map(|r| r.user_id.as_str().to_owned())
            .collect::<Vec<_>>(),
    )
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| {
            #[cfg(feature = "gateway")]
            let (requests_30d, cost_30d_microdollars) =
                spend.get(r.user_id.as_str()).copied().unwrap_or_default();
            RoleHolderRow {
                user_id: r.user_id,
                display_name: r.display_name,
                email: r.email,
                is_active: r.is_active,
                roles: r.roles,
                manual_roles: r.manual_roles,
                #[cfg(feature = "gateway")]
                requests_30d,
                #[cfg(feature = "gateway")]
                cost_30d_microdollars,
            }
        })
        .collect())
}

// Why: the gateway member's thirty-day requests and cost per holder, keyed by
// the ids the holder read returned. A holder with no request has spent nothing.
#[cfg(feature = "gateway")]
async fn list_spend_30d(
    pool: &PgPool,
    ids: &[String],
) -> Result<std::collections::HashMap<String, (i64, i64)>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT user_id AS "user_id!", COUNT(*)::BIGINT AS "requests!",
                  COALESCE(SUM(cost_microdollars), 0)::BIGINT AS "cost!"
           FROM ai_requests
           WHERE user_id = ANY($1) AND created_at >= NOW() - INTERVAL '30 days'
           GROUP BY user_id"#,
        ids,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| (r.user_id, (r.requests, r.cost)))
        .collect())
}
