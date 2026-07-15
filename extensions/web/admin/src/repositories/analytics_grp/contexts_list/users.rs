//! Per-user rollup of context activity — one row per user across all their
//! contexts.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::{ContextListFilter, ContextUserSummary, free_text_pattern, resolved_limit};

#[derive(Debug)]
struct ContextUserSummaryRow {
    user_id: UserId,
    display_name: Option<String>,
    context_count: i64,
    request_count: i64,
    message_count: i64,
    total_input_tokens: i64,
    total_output_tokens: i64,
    total_cost_microdollars: i64,
    error_count: i64,
    last_activity_at: Option<DateTime<Utc>>,
    distinct_models: Vec<String>,
}

impl From<ContextUserSummaryRow> for ContextUserSummary {
    fn from(r: ContextUserSummaryRow) -> Self {
        Self {
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
        }
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "body is one irreducible compile-time-checked query_as! SQL literal"
)]
pub async fn fetch_context_user_summary(
    pool: &PgPool,
    filter: &ContextListFilter,
) -> Result<Vec<ContextUserSummary>, sqlx::Error> {
    let limit = resolved_limit(filter.limit);
    let pattern = free_text_pattern(filter);

    let rows = sqlx::query_as!(
        ContextUserSummaryRow,
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
            ctx.user_id                                       AS "user_id!: UserId",
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
        filter.user_id.as_ref().map(UserId::as_str),
        filter.model,
        filter.since,
        pattern,
        limit,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(ContextUserSummary::from).collect())
}
