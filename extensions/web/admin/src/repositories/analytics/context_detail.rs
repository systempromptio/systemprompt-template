//! Context-detail repository — drives `/admin/contexts/{id}`.
//!
//! A context is the persisted state of an AI conversation: metadata in
//! `user_contexts`, plus every prompt + tool call carried by `ai_requests`
//! linked to that `context_id`.
//!
//! Message and tool-call bodies are fetched by request id, never by context.
//! The gateway stores the whole message array on every request, so a context's
//! message rows grow with the square of its turns; a context-wide query needs
//! a cap, and any cap on it silently truncates the one request the transcript
//! is actually built from. The caller names the handful of requests it needs
//! (`transcript_view::transcript_request_ids`) and the queries stay bounded by
//! the transcript rather than by the conversation's whole history.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{
    AiRequestId, ContextId, GatewayConversationId, SessionId, TraceId, UserId,
};

// Why: a conversation of any sane size fits well inside this; the cap only
// exists so a runaway context cannot pull an unbounded result set into memory.
// It is taken newest-first and reversed, so what a cap drops is always the
// oldest history rather than the transcript the reader is built from.
const REQUEST_CAP: i64 = 5000;

#[derive(Debug, Clone)]
pub struct ContextHeader {
    pub context_id: ContextId,
    pub user_id: Option<UserId>,
    pub display_name: Option<String>,
    pub session_id: Option<SessionId>,
    pub name: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    // Why: the Claude Code session uuid, which is also the key into the hook
    // pipeline's `plugin_session_summaries` row the next two fields come from.
    pub client_session_id: Option<String>,
    pub ai_title: Option<String>,
    pub hook_status: Option<String>,
}

#[derive(Debug, Clone, Default)]
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
    pub turn_count: i64,
    pub side_call_count: i64,
    pub side_call_cost_microdollars: i64,
    pub tool_call_count: i64,
    pub models: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ContextRequestRow {
    pub id: AiRequestId,
    pub trace_id: Option<TraceId>,
    // Why: NULL for a gateway-rejected request, which never reached a provider.
    pub model: Option<String>,
    pub status: String,
    pub latency_ms: Option<i32>,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cost_microdollars: i64,
    pub created_at: DateTime<Utc>,
    // Why: 'turn' | 'probe' | 'utility', as `conversation_requests` decides it.
    pub effective_kind: String,
    pub gateway_conversation_id: Option<GatewayConversationId>,
    pub max_tokens: Option<i32>,
    pub message_count: i64,
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
    // JSON: arbitrary MCP tool arguments, shaped by each tool's own schema.
    pub tool_input: serde_json::Value,
    // JSON: arbitrary MCP tool result payload, shaped by each tool's own schema.
    pub tool_result_payload: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

pub async fn find_context_header(
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
            c.updated_at                          AS "updated_at?",
            r.client_session_id                   AS "client_session_id?",
            s.ai_title                            AS "ai_title?",
            s.status                              AS "hook_status?"
        FROM (
            SELECT
                context_id,
                MAX(user_id) AS user_id,
                MAX(session_id) AS session_id,
                MAX(client_session_id) AS client_session_id
            FROM ai_requests
            WHERE context_id = $1
            GROUP BY context_id
        ) r
        FULL OUTER JOIN user_contexts c
          ON c.context_id = r.context_id
        LEFT JOIN plugin_session_summaries s ON s.session_id = r.client_session_id
        LEFT JOIN users u ON u.id = COALESCE(c.user_id, r.user_id)
        WHERE COALESCE(c.context_id, r.context_id) = $1
        LIMIT 1
        "#,
        context_id.as_str()
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_context_kpis(
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
            (ARRAY_AGG(model ORDER BY created_at DESC)
                FILTER (WHERE effective_kind = 'turn' AND model IS NOT NULL))[1]
                                                               AS "model?",
            COUNT(*) FILTER (WHERE effective_kind = 'turn')::bigint  AS "turn_count!",
            COUNT(*) FILTER (WHERE effective_kind <> 'turn')::bigint AS "side_call_count!",
            COALESCE(SUM(cost_microdollars) FILTER (WHERE effective_kind <> 'turn'), 0)::bigint
                                                               AS "side_call_cost_microdollars!",
            (SELECT COUNT(*)::bigint FROM ai_request_tool_calls t
              JOIN conversation_requests tr ON tr.id = t.request_id
             WHERE tr.context_id = $1 AND tr.effective_kind = 'turn')
                                                               AS "tool_call_count!",
            COALESCE(ARRAY_AGG(DISTINCT model) FILTER (WHERE model IS NOT NULL),
                     ARRAY[]::text[])                          AS "models!: Vec<String>"
        FROM conversation_requests
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
        turn_count: row.turn_count,
        side_call_count: row.side_call_count,
        side_call_cost_microdollars: row.side_call_cost_microdollars,
        tool_call_count: row.tool_call_count,
        models: row.models,
    })
}

pub async fn list_context_requests(
    pool: &PgPool,
    context_id: &ContextId,
) -> Result<Vec<ContextRequestRow>, sqlx::Error> {
    let mut rows = sqlx::query_as!(
        ContextRequestRow,
        r#"
        SELECT
            id                                  AS "id!: AiRequestId",
            trace_id                            AS "trace_id?: TraceId",
            model                               AS "model?",
            status                              AS "status!",
            latency_ms                          AS "latency_ms?",
            input_tokens                        AS "input_tokens?",
            output_tokens                       AS "output_tokens?",
            cost_microdollars                   AS "cost_microdollars!",
            created_at                          AS "created_at!",
            effective_kind                      AS "effective_kind!",
            gateway_conversation_id             AS "gateway_conversation_id?: GatewayConversationId",
            max_tokens                          AS "max_tokens?",
            (SELECT COUNT(*)::bigint FROM ai_request_messages m
              WHERE m.request_id = cr.id)       AS "message_count!"
        FROM conversation_requests cr
        WHERE context_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        "#,
        context_id.as_str(),
        REQUEST_CAP
    )
    .fetch_all(pool)
    .await?;
    // Why: the cap has to fall on the OLDEST requests, never the newest. The
    // transcript is built from the latest request of each thread, so a
    // cap taken ascending drops exactly the rows the reader needs and the page
    // renders empty while the KPI tiles still report the real totals.
    rows.reverse();
    Ok(rows)
}

pub async fn list_messages_for_requests(
    pool: &PgPool,
    request_ids: &[String],
) -> Result<Vec<ContextMessageRow>, sqlx::Error> {
    if request_ids.is_empty() {
        return Ok(Vec::new());
    }
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
        WHERE m.request_id = ANY($1)
        ORDER BY r.created_at ASC, m.sequence_number ASC
        "#,
        request_ids
    )
    .fetch_all(pool)
    .await
}

pub async fn list_tool_calls_for_requests(
    pool: &PgPool,
    request_ids: &[String],
) -> Result<Vec<ContextToolCallRow>, sqlx::Error> {
    if request_ids.is_empty() {
        return Ok(Vec::new());
    }
    // JSON: per-tool payload columns — see ContextToolCallRow above.
    sqlx::query_as!(
        ContextToolCallRow,
        r#"
        SELECT
            t.request_id          AS "request_id!: AiRequestId",
            t.tool_name           AS "tool_name!",
            t.sequence_number     AS "sequence_number!",
            -- `tool_input` is a TEXT column holding a JSON document. Selecting
            -- it raw decoded to JSON null in every row, because sqlx handed
            -- the text bytes to `serde_json::Value`'s Postgres decoder, which
            -- expects the JSON wire format. The cast makes the column what the
            -- Rust type already claimed it was.
            t.tool_input::jsonb   AS "tool_input!: serde_json::Value",
            t.tool_result_payload AS "tool_result_payload?: serde_json::Value",
            r.created_at          AS "created_at!"
        FROM ai_request_tool_calls t
        JOIN ai_requests r ON r.id = t.request_id
        WHERE t.request_id = ANY($1)
        ORDER BY r.created_at ASC, t.sequence_number ASC
        "#,
        request_ids
    )
    .fetch_all(pool)
    .await
}
