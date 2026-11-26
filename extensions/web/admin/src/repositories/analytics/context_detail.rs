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
//!
//! Tool calls live in the sibling `context_tool_calls` module and are
//! re-exported here so a caller that reads a context reads it whole.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{
    AiRequestId, AiToolCallId, ContextId, GatewayConversationId, SessionId, TraceId, UserId,
};

pub use super::context_tool_calls::{ContextToolCallRow, list_tool_calls_for_requests};

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
    // Why: `input_tokens` is the uncached count — core stores the four
    // billable counts disjointly — so the wire volume a session sent is the
    // three summed, and a session whose cache never hit shows it here.
    pub total_input_tokens: i64,
    pub total_cache_read_tokens: i64,
    pub total_cache_creation_tokens: i64,
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
    pub cache_read_tokens: Option<i32>,
    pub cache_creation_tokens: Option<i32>,
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
    // Why: the tool_use id a result message answers, when the gateway kept it.
    pub tool_call_id: Option<AiToolCallId>,
    pub name: Option<String>,
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
            COUNT(DISTINCT cr.trace_id)::bigint                AS "trace_count!",
            COUNT(*) FILTER (WHERE cr.status = 'failed')::bigint AS "error_count!",
            COALESCE(SUM(cr.input_tokens), 0)::bigint          AS "total_input_tokens!",
            COALESCE(SUM(ar.cache_read_tokens), 0)::bigint     AS "total_cache_read_tokens!",
            COALESCE(SUM(ar.cache_creation_tokens), 0)::bigint AS "total_cache_creation_tokens!",
            COALESCE(SUM(cr.output_tokens), 0)::bigint         AS "total_output_tokens!",
            COALESCE(SUM(cr.cost_microdollars), 0)::bigint     AS "total_cost_microdollars!",
            MIN(cr.created_at)                                 AS "first_request_at?",
            MAX(cr.created_at)                                 AS "last_request_at?",
            (ARRAY_AGG(cr.model ORDER BY cr.created_at DESC)
                FILTER (WHERE cr.effective_kind = 'turn' AND cr.model IS NOT NULL))[1]
                                                               AS "model?",
            COUNT(*) FILTER (WHERE cr.effective_kind = 'turn')::bigint  AS "turn_count!",
            COUNT(*) FILTER (WHERE cr.effective_kind <> 'turn')::bigint AS "side_call_count!",
            COALESCE(SUM(cr.cost_microdollars) FILTER (WHERE cr.effective_kind <> 'turn'), 0)::bigint
                                                               AS "side_call_cost_microdollars!",
            (SELECT COUNT(*)::bigint FROM ai_request_tool_calls t
              JOIN conversation_requests tr ON tr.id = t.request_id
             WHERE tr.context_id = $1 AND tr.effective_kind = 'turn')
                                                               AS "tool_call_count!",
            COALESCE(ARRAY_AGG(DISTINCT cr.model) FILTER (WHERE cr.model IS NOT NULL),
                     ARRAY[]::text[])                          AS "models!: Vec<String>"
        FROM conversation_requests cr
        JOIN ai_requests ar ON ar.id = cr.id
        WHERE cr.context_id = $1
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
        total_cache_read_tokens: row.total_cache_read_tokens,
        total_cache_creation_tokens: row.total_cache_creation_tokens,
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
            cr.id                               AS "id!: AiRequestId",
            cr.trace_id                         AS "trace_id?: TraceId",
            cr.model                            AS "model?",
            cr.status                           AS "status!",
            cr.latency_ms                       AS "latency_ms?",
            cr.input_tokens                     AS "input_tokens?",
            ar.cache_read_tokens                AS "cache_read_tokens?",
            ar.cache_creation_tokens            AS "cache_creation_tokens?",
            cr.output_tokens                    AS "output_tokens?",
            cr.cost_microdollars                AS "cost_microdollars!",
            cr.created_at                       AS "created_at!",
            cr.effective_kind                   AS "effective_kind!",
            cr.gateway_conversation_id          AS "gateway_conversation_id?: GatewayConversationId",
            cr.max_tokens                       AS "max_tokens?",
            (SELECT COUNT(*)::bigint FROM ai_request_messages m
              WHERE m.request_id = cr.id)       AS "message_count!"
        FROM conversation_requests cr
        JOIN ai_requests ar ON ar.id = cr.id
        WHERE cr.context_id = $1
        ORDER BY cr.created_at DESC
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
            m.tool_call_id    AS "tool_call_id?: AiToolCallId",
            m.name            AS "name?",
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
