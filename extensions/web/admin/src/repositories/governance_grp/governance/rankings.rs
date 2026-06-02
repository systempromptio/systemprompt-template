//! Top-actor / top-policy / grouped-incident rankings over a sliding window,
//! joined to `users` for display names where available.

use sqlx::PgPool;

use crate::types::{IncidentGroup, TopActor, TopPolicy};

pub async fn fetch_top_actors(
    pool: &PgPool,
    window_seconds: i64,
    limit: i64,
) -> Result<Vec<TopActor>, sqlx::Error> {
    sqlx::query_as!(
        TopActor,
        r#"SELECT
            g.user_id::TEXT AS "user_id!",
            COALESCE(u.display_name, u.full_name, u.name, u.email, g.user_id) AS "display_name!",
            u.email::TEXT AS email,
            COUNT(*) FILTER (WHERE g.decision = 'deny')::bigint AS "deny_count!",
            COUNT(*) FILTER (
                WHERE g.decision = 'deny'
                  AND (g.policy = 'secret_scan' OR g.reason ILIKE '%secret%')
            )::bigint AS "secret_count!",
            COUNT(*)::bigint AS "total!"
        FROM governance_decisions g
        LEFT JOIN users u ON u.id = g.user_id
        WHERE g.created_at > now() - make_interval(secs => $1::double precision)
        GROUP BY g.user_id, u.display_name, u.full_name, u.name, u.email
        ORDER BY 4 DESC, 6 DESC
        LIMIT $2"#,
        window_seconds as f64,
        limit,
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_top_policies(
    pool: &PgPool,
    window_seconds: i64,
    limit: i64,
) -> Result<Vec<TopPolicy>, sqlx::Error> {
    sqlx::query_as!(
        TopPolicy,
        r#"SELECT
            policy,
            tool_name,
            COUNT(*)::bigint AS "hits!",
            COUNT(DISTINCT user_id)::bigint AS "distinct_actors!"
        FROM governance_decisions
        WHERE decision = 'deny'
          AND created_at > now() - make_interval(secs => $1::double precision)
        GROUP BY policy, tool_name
        ORDER BY 3 DESC
        LIMIT $2"#,
        window_seconds as f64,
        limit,
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_grouped_incidents(
    pool: &PgPool,
    window_seconds: i64,
    limit: i64,
) -> Result<Vec<IncidentGroup>, sqlx::Error> {
    sqlx::query_as!(
        IncidentGroup,
        r#"SELECT
            g.agent_id,
            g.user_id::TEXT AS "user_id!",
            COALESCE(u.display_name, u.full_name, u.name, u.email) AS display_name,
            g.policy,
            g.tool_name,
            COUNT(*)::bigint AS "attempts!",
            MIN(g.created_at) AS "first_seen!",
            MAX(g.created_at) AS "last_seen!",
            COALESCE((ARRAY_AGG(g.reason ORDER BY g.created_at DESC))[1], '') AS "sample_reason!"
        FROM governance_decisions g
        LEFT JOIN users u ON u.id = g.user_id
        WHERE g.decision = 'deny'
          AND g.created_at > now() - make_interval(secs => $1::double precision)
        GROUP BY g.agent_id, g.user_id, u.display_name, u.full_name, u.name, u.email,
                 g.policy, g.tool_name
        ORDER BY 6 DESC, 8 DESC
        LIMIT $2"#,
        window_seconds as f64,
        limit,
    )
    .fetch_all(pool)
    .await
}
