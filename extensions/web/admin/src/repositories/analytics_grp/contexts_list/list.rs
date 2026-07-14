//! Per-context list query — one row per context with its request, message,
//! token, and cost rollups.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::{ContextListFilter, ContextListItem, free_text_pattern, resolved_limit};

#[derive(Debug)]
struct ContextListRow {
    context_id: String,
    name: Option<String>,
    user_id: Option<String>,
    display_name: Option<String>,
    session_id: Option<String>,
    model: Option<String>,
    request_count: i64,
    message_count: i64,
    error_count: i64,
    total_input_tokens: i64,
    total_output_tokens: i64,
    total_cost_microdollars: i64,
    first_request_at: Option<DateTime<Utc>>,
    last_request_at: Option<DateTime<Utc>>,
    last_activity_at: Option<DateTime<Utc>>,
}

impl From<ContextListRow> for ContextListItem {
    fn from(r: ContextListRow) -> Self {
        Self {
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
        }
    }
}

pub async fn fetch_context_list(
    pool: &PgPool,
    filter: &ContextListFilter,
) -> Result<Vec<ContextListItem>, sqlx::Error> {
    let limit = resolved_limit(filter.limit);
    let pattern = free_text_pattern(filter);

    let rows = sqlx::query_as!(
        ContextListRow,
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

    Ok(rows.into_iter().map(ContextListItem::from).collect())
}
