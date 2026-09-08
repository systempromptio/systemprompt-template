//! The roster's headline numbers — the KPI row, and every filter chip's count.
//!
//! One statement rather than one per chip: the four populations are `FILTER`
//! clauses over the same scan, so the tiles and the table they filter can never
//! disagree about how many people there are.

use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::scope::SubjectScope;

use super::WINDOW_DAYS;

/// The five numbers the KPI row states, each one also a filter chip's count.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct RosterStats {
    pub total: i64,
    pub active: i64,
    pub unassigned: i64,
    pub no_role: i64,
    pub inactive_30d: i64,
    pub requests: i64,
    pub cost_microdollars: i64,
    // Why: the previous window, so the two spend KPIs can carry a delta rather
    // than a bare number the reader has nothing to compare against.
    pub prior_cost_microdollars: i64,
    pub prior_requests: i64,
}

pub async fn get_roster_stats(
    pool: &PgPool,
    scope: &SubjectScope,
) -> Result<RosterStats, sqlx::Error> {
    let row = sqlx::query!(
        r#"WITH base AS (
               SELECT u.id, u.roles, (u.status = 'active') AS is_active
                 FROM users u
                WHERE NOT ('anonymous' = ANY(u.roles))
                  AND u.email NOT LIKE '%@anonymous.local'
                  AND ($1::TEXT[] IS NULL OR u.id = ANY($1))
           ), marked AS (
               SELECT b.id, b.is_active,
                      cardinality(array_remove(array_remove(b.roles, 'user'), 'anonymous')) = 0 AS no_role,
                      NOT EXISTS (SELECT 1 FROM group_members gm WHERE gm.user_id = b.id) AS unassigned,
                      GREATEST(
                        (SELECT MAX(ar.created_at) FROM ai_requests ar WHERE ar.user_id = b.id),
                        (SELECT MAX(us.last_activity_at) FROM user_sessions us WHERE us.user_id = b.id),
                        (SELECT MAX(ua.created_at) FROM user_activity ua WHERE ua.user_id = b.id)
                      ) AS last_active
                 FROM base b
           ), spend AS (
               SELECT
                 COALESCE(SUM(ar.cost_microdollars) FILTER (
                     WHERE ar.created_at >= NOW() - make_interval(days => $2::INT)), 0)::BIGINT AS cost,
                 COUNT(*) FILTER (
                     WHERE ar.created_at >= NOW() - make_interval(days => $2::INT))::BIGINT AS requests,
                 COALESCE(SUM(ar.cost_microdollars) FILTER (
                     WHERE ar.created_at >= NOW() - make_interval(days => $2::INT * 2)
                       AND ar.created_at <  NOW() - make_interval(days => $2::INT)), 0)::BIGINT AS prior_cost,
                 COUNT(*) FILTER (
                     WHERE ar.created_at >= NOW() - make_interval(days => $2::INT * 2)
                       AND ar.created_at <  NOW() - make_interval(days => $2::INT))::BIGINT AS prior_requests
               FROM ai_requests ar
               JOIN base b ON b.id = ar.user_id
               WHERE ar.created_at >= NOW() - make_interval(days => $2::INT * 2)
           )
           SELECT COUNT(*)::BIGINT AS "total!",
                  COUNT(*) FILTER (WHERE m.is_active)::BIGINT AS "active!",
                  COUNT(*) FILTER (WHERE m.unassigned)::BIGINT AS "unassigned!",
                  COUNT(*) FILTER (WHERE m.no_role)::BIGINT AS "no_role!",
                  COUNT(*) FILTER (
                      WHERE m.last_active IS NULL
                         OR m.last_active < NOW() - INTERVAL '30 days'
                  )::BIGINT AS "inactive_30d!",
                  (SELECT cost FROM spend) AS "cost!",
                  (SELECT requests FROM spend) AS "requests!",
                  (SELECT prior_cost FROM spend) AS "prior_cost!",
                  (SELECT prior_requests FROM spend) AS "prior_requests!"
             FROM marked m"#,
        scope.as_sql(),
        WINDOW_DAYS,
    )
    .fetch_one(pool)
    .await?;

    Ok(RosterStats {
        total: row.total,
        active: row.active,
        unassigned: row.unassigned,
        no_role: row.no_role,
        inactive_30d: row.inactive_30d,
        requests: row.requests,
        cost_microdollars: row.cost,
        prior_cost_microdollars: row.prior_cost,
        prior_requests: row.prior_requests,
    })
}

// Why: empty deep pages still need the filtered total, without spending/row
// enrichment.
pub(super) async fn count_filtered_users(
    pool: &PgPool,
    scope: &SubjectScope,
    query: &super::RosterQuery,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(r#"
        SELECT COUNT(*)::bigint AS "count!" FROM users u
        WHERE NOT ('anonymous' = ANY(u.roles)) AND u.email NOT LIKE '%@anonymous.local'
          AND ($1::text[] IS NULL OR u.id = ANY($1))
          AND ($2::text IS NULL OR $2 = ANY(u.roles))
          AND ($3::text IS NULL OR position(lower($3) in lower(
              COALESCE(u.display_name, u.full_name, u.name, '') || ' ' || COALESCE(u.email, '') || ' ' || u.id)) > 0)
          AND CASE $4::text
            WHEN 'unassigned' THEN NOT EXISTS (SELECT 1 FROM group_members gm WHERE gm.user_id = u.id)
                OR EXISTS (SELECT 1 FROM group_members gm WHERE gm.user_id = u.id AND gm.group_id = 'unassigned')
            WHEN 'no-role' THEN cardinality(array_remove(array_remove(u.roles, 'user'), 'anonymous')) = 0
            WHEN 'inactive-30d' THEN
                NOT EXISTS (SELECT 1 FROM ai_requests ar WHERE ar.user_id = u.id AND ar.created_at >= NOW() - INTERVAL '30 days')
                AND NOT EXISTS (SELECT 1 FROM user_sessions us WHERE us.user_id = u.id AND us.last_activity_at >= NOW() - INTERVAL '30 days')
                AND NOT EXISTS (SELECT 1 FROM user_activity ua WHERE ua.user_id = u.id AND ua.created_at >= NOW() - INTERVAL '30 days')
            ELSE TRUE END
    "#, scope.as_sql(), query.role.as_deref(), query.search.as_deref(), query.filter.as_str())
    .fetch_one(pool).await
}
