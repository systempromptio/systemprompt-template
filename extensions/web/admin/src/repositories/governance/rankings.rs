//! Top-actor / top-policy / grouped-incident rankings over a sliding window,
//! joined to `users` for display names where available.

use sqlx::PgPool;

use crate::types::{TopActor, TopPolicy};

// Live upstream in systemprompt-template via the ssr_governance
// handlers, which this fork does not ship. Kept so the shared
// repository files stay identical across both trees.
// lint-ok: unused-pub
pub async fn list_top_actors(
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

// Live upstream in systemprompt-template via the ssr_governance
// handlers, which this fork does not ship. Kept so the shared
// repository files stay identical across both trees.
// lint-ok: unused-pub
pub async fn list_top_policies(
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
