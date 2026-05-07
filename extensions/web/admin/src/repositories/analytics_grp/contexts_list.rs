//! Contexts-list repository — drives `/admin/entities/contexts`.
//!
//! Aggregates every `ai_requests` row by `context_id` and `FULL OUTER JOIN`s
//! against `user_contexts` so we surface contexts that exist only in one side
//! (a `user_contexts` row with no traffic, or traffic that bypassed
//! `core contexts create`). Mirrors the JOIN shape used by
//! `context_detail::fetch_context_header`.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug, Clone, Default)]
pub struct ContextListFilter {
    pub user_id: Option<String>,
    pub model: Option<String>,
    pub free_text: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub limit: i64,
}

#[derive(Debug, Clone)]
pub struct ContextListItem {
    pub context_id: String,
    pub name: Option<String>,
    pub user_id: Option<String>,
    pub display_name: Option<String>,
    pub session_id: Option<String>,
    pub model: Option<String>,
    pub request_count: i64,
    pub message_count: i64,
    pub error_count: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_microdollars: i64,
    pub first_request_at: Option<DateTime<Utc>>,
    pub last_request_at: Option<DateTime<Utc>>,
    pub last_activity_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct ContextUserSummary {
    pub user_id: String,
    pub display_name: Option<String>,
    pub context_count: i64,
    pub request_count: i64,
    pub message_count: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_microdollars: i64,
    pub error_count: i64,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub distinct_models: Vec<String>,
}

fn resolved_limit(requested: i64) -> i64 {
    if requested > 0 && requested <= 500 {
        requested
    } else {
        100
    }
}

fn free_text_pattern(filter: &ContextListFilter) -> Option<String> {
    filter
        .free_text
        .as_ref()
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{}%", s.replace('\\', "\\\\").replace('%', "\\%")))
}

pub async fn fetch_context_list(
    pool: &PgPool,
    filter: &ContextListFilter,
) -> Result<Vec<ContextListItem>, sqlx::Error> {
    let limit = resolved_limit(filter.limit);
    let pattern = free_text_pattern(filter);

    let rows = sqlx::query!(
        r#"
        WITH req AS (
            SELECT
                context_id,
                MAX(user_id)                                AS user_id,
                MAX(session_id)                             AS session_id,
                COUNT(*)::bigint                            AS request_count,
                COUNT(*) FILTER (WHERE status = 'failed')::bigint AS error_count,
                COALESCE(SUM(input_tokens), 0)::bigint      AS total_input_tokens,
                COALESCE(SUM(output_tokens), 0)::bigint     AS total_output_tokens,
                COALESCE(SUM(cost_microdollars), 0)::bigint AS total_cost_microdollars,
                MIN(created_at)                             AS first_request_at,
                MAX(created_at)                             AS last_request_at,
                (ARRAY_AGG(model ORDER BY created_at DESC))[1] AS model
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
        )
        SELECT
            COALESCE(c.context_id, req.context_id)        AS "context_id!",
            c.name                                        AS "name?",
            COALESCE(c.user_id, req.user_id)              AS "user_id?",
            u.display_name                                AS "display_name?",
            COALESCE(c.session_id, req.session_id)        AS "session_id?",
            req.model                                     AS "model?",
            COALESCE(req.request_count, 0)::bigint        AS "request_count!",
            COALESCE(msgs.message_count, 0)::bigint       AS "message_count!",
            COALESCE(req.error_count, 0)::bigint          AS "error_count!",
            COALESCE(req.total_input_tokens, 0)::bigint   AS "total_input_tokens!",
            COALESCE(req.total_output_tokens, 0)::bigint  AS "total_output_tokens!",
            COALESCE(req.total_cost_microdollars, 0)::bigint AS "total_cost_microdollars!",
            req.first_request_at                          AS "first_request_at?",
            req.last_request_at                           AS "last_request_at?",
            COALESCE(req.last_request_at, c.updated_at)   AS "last_activity_at?"
        FROM user_contexts c
        FULL OUTER JOIN req ON req.context_id = c.context_id
        LEFT JOIN msgs ON msgs.context_id = COALESCE(c.context_id, req.context_id)
        LEFT JOIN users u ON u.id = COALESCE(c.user_id, req.user_id)
        WHERE ($1::text IS NULL OR COALESCE(c.user_id, req.user_id) = $1)
          AND ($2::text IS NULL OR req.model = $2)
          AND ($3::timestamptz IS NULL
               OR COALESCE(req.last_request_at, c.updated_at) >= $3)
          AND ($4::text IS NULL
               OR c.name ILIKE $4
               OR u.display_name ILIKE $4
               OR COALESCE(c.context_id, req.context_id) ILIKE $4
               OR COALESCE(c.user_id, req.user_id) ILIKE $4)
        ORDER BY COALESCE(req.last_request_at, c.updated_at) DESC NULLS LAST
        LIMIT $5
        "#,
        filter.user_id,
        filter.model,
        filter.since,
        pattern,
        limit,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ContextListItem {
            context_id: r.context_id,
            name: r.name,
            user_id: r.user_id,
            display_name: r.display_name,
            session_id: r.session_id,
            model: r.model,
            request_count: r.request_count,
            message_count: r.message_count,
            error_count: r.error_count,
            total_input_tokens: r.total_input_tokens,
            total_output_tokens: r.total_output_tokens,
            total_cost_microdollars: r.total_cost_microdollars,
            first_request_at: r.first_request_at,
            last_request_at: r.last_request_at,
            last_activity_at: r.last_activity_at,
        })
        .collect())
}

pub async fn fetch_context_user_summary(
    pool: &PgPool,
    filter: &ContextListFilter,
) -> Result<Vec<ContextUserSummary>, sqlx::Error> {
    let limit = resolved_limit(filter.limit);
    let pattern = free_text_pattern(filter);

    let rows = sqlx::query!(
        r#"
        WITH ctx AS (
            SELECT
                COALESCE(c.user_id, r.user_id)         AS user_id,
                COALESCE(c.context_id, r.context_id)   AS context_id,
                MAX(c.updated_at)                      AS context_updated_at
            FROM user_contexts c
            FULL OUTER JOIN (
                SELECT DISTINCT context_id, MAX(user_id) AS user_id
                FROM ai_requests
                WHERE context_id IS NOT NULL
                GROUP BY context_id
            ) r ON r.context_id = c.context_id
            GROUP BY COALESCE(c.user_id, r.user_id), COALESCE(c.context_id, r.context_id)
        ),
        req_per_user AS (
            SELECT
                user_id,
                COUNT(*)::bigint                            AS request_count,
                COUNT(*) FILTER (WHERE status = 'failed')::bigint AS error_count,
                COALESCE(SUM(input_tokens), 0)::bigint      AS total_input_tokens,
                COALESCE(SUM(output_tokens), 0)::bigint     AS total_output_tokens,
                COALESCE(SUM(cost_microdollars), 0)::bigint AS total_cost_microdollars,
                MAX(created_at)                             AS last_request_at,
                ARRAY_AGG(DISTINCT model)                   AS models
            FROM ai_requests
            WHERE context_id IS NOT NULL
            GROUP BY user_id
        ),
        msgs_per_user AS (
            SELECT r.user_id, COUNT(*)::bigint AS message_count
            FROM ai_request_messages m
            JOIN ai_requests r ON r.id = m.request_id
            WHERE r.context_id IS NOT NULL
            GROUP BY r.user_id
        )
        SELECT
            ctx.user_id                                       AS "user_id!",
            u.display_name                                    AS "display_name?",
            COUNT(ctx.context_id)::bigint                     AS "context_count!",
            COALESCE(MAX(req_per_user.request_count), 0)::bigint     AS "request_count!",
            COALESCE(MAX(msgs_per_user.message_count), 0)::bigint    AS "message_count!",
            COALESCE(MAX(req_per_user.total_input_tokens), 0)::bigint AS "total_input_tokens!",
            COALESCE(MAX(req_per_user.total_output_tokens), 0)::bigint AS "total_output_tokens!",
            COALESCE(MAX(req_per_user.total_cost_microdollars), 0)::bigint AS "total_cost_microdollars!",
            COALESCE(MAX(req_per_user.error_count), 0)::bigint AS "error_count!",
            GREATEST(MAX(req_per_user.last_request_at), MAX(ctx.context_updated_at))
                                                              AS "last_activity_at?",
            COALESCE(MAX(req_per_user.models), ARRAY[]::text[]) AS "distinct_models!"
        FROM ctx
        LEFT JOIN req_per_user  ON req_per_user.user_id  = ctx.user_id
        LEFT JOIN msgs_per_user ON msgs_per_user.user_id = ctx.user_id
        LEFT JOIN users u       ON u.id = ctx.user_id
        WHERE ctx.user_id IS NOT NULL
          AND ($1::text IS NULL OR ctx.user_id = $1)
          AND ($2::text IS NULL OR $2 = ANY(req_per_user.models))
          AND ($3::timestamptz IS NULL
               OR GREATEST(req_per_user.last_request_at, ctx.context_updated_at) >= $3)
          AND ($4::text IS NULL
               OR ctx.user_id ILIKE $4
               OR u.display_name ILIKE $4)
        GROUP BY ctx.user_id, u.display_name
        ORDER BY GREATEST(MAX(req_per_user.last_request_at), MAX(ctx.context_updated_at))
                 DESC NULLS LAST
        LIMIT $5
        "#,
        filter.user_id,
        filter.model,
        filter.since,
        pattern,
        limit,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ContextUserSummary {
            user_id: r.user_id,
            display_name: r.display_name,
            context_count: r.context_count,
            request_count: r.request_count,
            message_count: r.message_count,
            total_input_tokens: r.total_input_tokens,
            total_output_tokens: r.total_output_tokens,
            total_cost_microdollars: r.total_cost_microdollars,
            error_count: r.error_count,
            last_activity_at: r.last_activity_at,
            distinct_models: r.distinct_models,
        })
        .collect())
}

#[derive(Debug, Clone)]
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
        filter.user_id,
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
