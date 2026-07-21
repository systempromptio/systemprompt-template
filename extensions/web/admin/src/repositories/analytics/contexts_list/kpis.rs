//! KPI strip + distinct-model lookup for the contexts list page.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::{ContextListFilter, free_text_pattern};

#[derive(Debug, Clone, Copy)]
pub struct ContextListKpis {
    pub total_contexts: i64,
    pub active_users: i64,
    pub total_requests: i64,
    pub total_messages: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_microdollars: i64,
}

pub async fn fetch_context_list_kpis(
    pool: &PgPool,
    filter: &ContextListFilter,
) -> Result<ContextListKpis, sqlx::Error> {
    let pattern = free_text_pattern(filter);

    let row = sqlx::query!(
        r#"
        WITH req AS (
            SELECT
                context_id,
                MAX(user_id)                                AS user_id,
                MAX(model)                                  AS model,
                COUNT(*)::bigint                            AS request_count,
                COALESCE(SUM(input_tokens), 0)::bigint      AS total_input_tokens,
                COALESCE(SUM(output_tokens), 0)::bigint     AS total_output_tokens,
                COALESCE(SUM(cost_microdollars), 0)::bigint AS total_cost_microdollars,
                MAX(created_at)                             AS last_request_at
            FROM ai_requests
            WHERE context_id IS NOT NULL
            GROUP BY context_id
        ),
        msgs AS (
            SELECT r.context_id, COUNT(*)::bigint AS message_count
            FROM ai_request_messages m
            JOIN ai_requests r ON r.id = m.request_id
            WHERE r.context_id IS NOT NULL
            GROUP BY r.context_id
        ),
        joined AS (
            SELECT
                COALESCE(c.context_id, req.context_id) AS context_id,
                COALESCE(c.user_id, req.user_id)       AS user_id,
                req.model                              AS model,
                COALESCE(req.request_count, 0)         AS request_count,
                COALESCE(msgs.message_count, 0)        AS message_count,
                COALESCE(req.total_input_tokens, 0)    AS total_input_tokens,
                COALESCE(req.total_output_tokens, 0)   AS total_output_tokens,
                COALESCE(req.total_cost_microdollars, 0) AS total_cost_microdollars,
                COALESCE(req.last_request_at, c.updated_at) AS last_activity_at
            FROM user_contexts c
            FULL OUTER JOIN req ON req.context_id = c.context_id
            LEFT JOIN msgs ON msgs.context_id = COALESCE(c.context_id, req.context_id)
        )
        SELECT
            COUNT(DISTINCT j.context_id)::bigint              AS "total_contexts!",
            COUNT(DISTINCT j.user_id)::bigint                 AS "active_users!",
            COALESCE(SUM(j.request_count), 0)::bigint         AS "total_requests!",
            COALESCE(SUM(j.message_count), 0)::bigint         AS "total_messages!",
            COALESCE(SUM(j.total_input_tokens), 0)::bigint    AS "total_input_tokens!",
            COALESCE(SUM(j.total_output_tokens), 0)::bigint   AS "total_output_tokens!",
            COALESCE(SUM(j.total_cost_microdollars), 0)::bigint AS "total_cost_microdollars!"
        FROM joined j
        LEFT JOIN users u ON u.id = j.user_id
        WHERE ($1::text IS NULL OR j.user_id = $1)
          AND ($2::text IS NULL OR j.model = $2)
          AND ($3::timestamptz IS NULL OR j.last_activity_at >= $3)
          AND ($4::text IS NULL
               OR u.display_name ILIKE $4
               OR j.context_id ILIKE $4
               OR j.user_id ILIKE $4)
        "#,
        filter.user_id.as_ref().map(UserId::as_str),
        filter.model,
        filter.since,
        pattern,
    )
    .fetch_one(pool)
    .await?;

    Ok(ContextListKpis {
        total_contexts: row.total_contexts,
        active_users: row.active_users,
        total_requests: row.total_requests,
        total_messages: row.total_messages,
        total_input_tokens: row.total_input_tokens,
        total_output_tokens: row.total_output_tokens,
        total_cost_microdollars: row.total_cost_microdollars,
    })
}

pub async fn fetch_distinct_models(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT DISTINCT model AS "model!"
        FROM ai_requests
        WHERE context_id IS NOT NULL
        ORDER BY model
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.model).collect())
}
