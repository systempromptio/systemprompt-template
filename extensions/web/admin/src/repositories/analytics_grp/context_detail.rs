//! Context-detail repository — drives `/admin/contexts/{id}`.
//!
//! A context is the persisted state of an AI conversation: metadata in
//! `user_contexts`, plus every prompt + tool call carried by `ai_requests`
//! linked to that `context_id`.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{AiRequestId, ContextId, SessionId, TraceId, UserId};

#[derive(Debug, Clone)]
pub struct ContextHeader {
    pub context_id: ContextId,
    pub user_id: Option<UserId>,
    pub display_name: Option<String>,
    pub session_id: Option<SessionId>,
    pub name: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct ContextKpis {
    pub request_count: i64,
    pub trace_count: i64,
    pub error_count: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_microdollars: i64,
    pub first_request_at: Option<DateTime<Utc>>,
    pub last_request_at: Option<DateTime<Utc>>,
    pub model: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ContextRequestRow {
    pub id: String,
    pub trace_id: Option<TraceId>,
    pub model: String,
    pub status: String,
    pub latency_ms: Option<i32>,
    pub cost_microdollars: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ContextMessageRow {
    pub request_id: AiRequestId,
    pub role: String,
    pub sequence_number: i32,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ContextToolCallRow {
    pub request_id: AiRequestId,
    pub tool_name: String,
    pub sequence_number: i32,
    pub tool_input: serde_json::Value,
    pub tool_result_payload: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

pub async fn fetch_context_header(
    pool: &PgPool,
    context_id: &ContextId,
) -> Result<Option<ContextHeader>, sqlx::Error> {
    sqlx::query_as!(
        ContextHeader,
        r#"
        SELECT
            COALESCE(c.context_id, r.context_id)  AS "context_id!: ContextId",
            COALESCE(c.user_id, r.user_id)        AS "user_id?: UserId",
            u.display_name                        AS "display_name?",
            COALESCE(c.session_id, r.session_id)  AS "session_id?: SessionId",
            c.name                                AS "name?",
            c.created_at                          AS "created_at?",
            c.updated_at                          AS "updated_at?"
        FROM (
            SELECT
                context_id,
                MAX(user_id) AS user_id,
                MAX(session_id) AS session_id
            FROM ai_requests
            WHERE context_id = $1
            GROUP BY context_id
        ) r
        FULL OUTER JOIN user_contexts c
          ON c.context_id = r.context_id
        LEFT JOIN users u ON u.id = COALESCE(c.user_id, r.user_id)
        WHERE COALESCE(c.context_id, r.context_id) = $1
        LIMIT 1
        "#,
        context_id.as_str()
    )
    .fetch_optional(pool)
    .await
}

pub async fn fetch_context_kpis(
    pool: &PgPool,
    context_id: &ContextId,
) -> Result<ContextKpis, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*)::bigint                                   AS "request_count!",
            COUNT(DISTINCT trace_id)::bigint                   AS "trace_count!",
            COUNT(*) FILTER (WHERE status = 'failed')::bigint  AS "error_count!",
            COALESCE(SUM(input_tokens), 0)::bigint             AS "total_input_tokens!",
            COALESCE(SUM(output_tokens), 0)::bigint            AS "total_output_tokens!",
            COALESCE(SUM(cost_microdollars), 0)::bigint        AS "total_cost_microdollars!",
            MIN(created_at)                                    AS "first_request_at?",
            MAX(created_at)                                    AS "last_request_at?",
            (ARRAY_AGG(model ORDER BY created_at DESC))[1]     AS "model?"
        FROM ai_requests
        WHERE context_id = $1
        "#,
        context_id.as_str()
    )
    .fetch_one(pool)
    .await?;
    Ok(ContextKpis {
        request_count: row.request_count,
        trace_count: row.trace_count,
        error_count: row.error_count,
        total_input_tokens: row.total_input_tokens,
        total_output_tokens: row.total_output_tokens,
        total_cost_microdollars: row.total_cost_microdollars,
        first_request_at: row.first_request_at,
        last_request_at: row.last_request_at,
        model: row.model,
    })
}

pub async fn fetch_context_requests(
    pool: &PgPool,
    context_id: &ContextId,
) -> Result<Vec<ContextRequestRow>, sqlx::Error> {
    sqlx::query_as!(
        ContextRequestRow,
        r#"
        SELECT
            id                                  AS "id!",
            trace_id                            AS "trace_id?: TraceId",
            model                               AS "model!",
            status                              AS "status!",
            latency_ms                          AS "latency_ms?",
            cost_microdollars                   AS "cost_microdollars!",
            created_at                          AS "created_at!"
        FROM ai_requests
        WHERE context_id = $1
        ORDER BY created_at ASC
        LIMIT 500
        "#,
        context_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_context_messages(
    pool: &PgPool,
    context_id: &ContextId,
) -> Result<Vec<ContextMessageRow>, sqlx::Error> {
    sqlx::query_as!(
        ContextMessageRow,
        r#"
        SELECT
            m.request_id      AS "request_id!: AiRequestId",
            m.role            AS "role!",
            m.sequence_number AS "sequence_number!",
            m.content         AS "content!",
            r.created_at      AS "created_at!"
        FROM ai_request_messages m
        JOIN ai_requests r ON r.id = m.request_id
        WHERE r.context_id = $1
        ORDER BY r.created_at ASC, m.sequence_number ASC
        LIMIT 2000
        "#,
        context_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_context_tool_calls(
    pool: &PgPool,
    context_id: &ContextId,
) -> Result<Vec<ContextToolCallRow>, sqlx::Error> {
    sqlx::query_as!(
        ContextToolCallRow,
        r#"
        SELECT
            t.request_id          AS "request_id!: AiRequestId",
            t.tool_name           AS "tool_name!",
            t.sequence_number     AS "sequence_number!",
            t.tool_input          AS "tool_input!",
            t.tool_result_payload AS "tool_result_payload?: serde_json::Value",
            r.created_at          AS "created_at!"
        FROM ai_request_tool_calls t
        JOIN ai_requests r ON r.id = t.request_id
        WHERE r.context_id = $1
        ORDER BY r.created_at ASC, t.sequence_number ASC
        LIMIT 1000
        "#,
        context_id.as_str()
    )
    .fetch_all(pool)
    .await
}
