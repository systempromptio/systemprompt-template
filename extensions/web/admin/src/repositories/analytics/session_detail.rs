//! Session-detail repository — drives `/admin/sessions/{id}`.
//!
//! A session groups every AI request, context, and trace produced by a single
//! interactive run. This module assembles the header row from
//! `plugin_session_summaries` (when present) plus an `ai_requests` rollup, and
//! returns the contexts/traces/requests that belong to the session.
//!
//! Three id shapes reach `/admin/sessions/{id}` and all resolve here: the
//! gateway's own `sess_…` id (`ai_requests.session_id`), the Claude Code
//! session uuid (`ai_requests.client_session_id`, which is also
//! `plugin_session_summaries.session_id`), and a hook-only session that never
//! produced gateway traffic. Every read matches `session_id OR
//! client_session_id`, so one page answers whichever id the link carried.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{AiRequestId, ContextId, PluginId, SessionId, TraceId, UserId};

#[derive(Debug, Clone)]
pub struct SessionHeader {
    pub session_id: SessionId,
    pub user_id: Option<UserId>,
    pub display_name: Option<String>,
    pub groups: Vec<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub model: Option<String>,
    pub plugin_id: Option<PluginId>,
    pub ai_title: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct SessionKpis {
    pub request_count: i64,
    pub context_count: i64,
    pub trace_count: i64,
    pub error_count: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_microdollars: i64,
}

#[derive(Debug, Clone)]
pub struct SessionContextRow {
    pub context_id: ContextId,
    pub name: Option<String>,
    // Why: the resolved conversation title from `conversation_rollups`, so the
    // row never has to show a bare id.
    pub title: String,
    pub turn_count: i64,
    pub request_count: i64,
    pub last_request_at: Option<DateTime<Utc>>,
    pub model: Option<String>,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub cost_microdollars: i64,
    pub error_count: i64,
}

#[derive(Debug, Clone)]
pub struct SessionTraceRow {
    pub trace_id: TraceId,
    pub request_count: i64,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub error_count: i64,
}

#[derive(Debug, Clone)]
pub struct SessionRequestRow {
    pub id: AiRequestId,
    pub context_id: Option<ContextId>,
    pub trace_id: Option<TraceId>,
    // Why: NULL for a gateway-rejected request, which never reached a provider.
    pub model: Option<String>,
    pub status: String,
    pub latency_ms: Option<i32>,
    pub cost_microdollars: i64,
    pub created_at: DateTime<Utc>,
}

pub async fn find_session_header(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Option<SessionHeader>, sqlx::Error> {
    sqlx::query_as!(
        SessionHeader,
        r#"
        SELECT
            $1::text                             AS "session_id!: SessionId",
            COALESCE(s.user_id, r.user_id)       AS "user_id?: UserId",
            u.display_name                       AS "display_name?",
            ARRAY(SELECT ug.group_id FROM user_groups ug
                   WHERE ug.user_id = u.id)          AS "groups!: Vec<String>",
            COALESCE(s.started_at, r.first_seen) AS "started_at?",
            -- A hook session that never wrote an `ended_at` still has a last
            -- activity: the moment it started. Falling through to NULL made
            -- the field, and the duration derived from it, read as unknown.
            GREATEST(r.last_seen, COALESCE(s.ended_at, s.started_at))
                                                 AS "last_activity_at?",
            s.status                             AS "status?",
            COALESCE(s.model, r.model)           AS "model?",
            s.plugin_id                          AS "plugin_id?: PluginId",
            s.ai_title                           AS "ai_title?"
        FROM (
            SELECT
                COUNT(*)::bigint AS request_count,
                MAX(user_id) AS user_id,
                MIN(created_at) AS first_seen,
                MAX(created_at) AS last_seen,
                (ARRAY_AGG(model ORDER BY created_at DESC)
                    FILTER (WHERE model IS NOT NULL))[1] AS model
            FROM ai_requests
            WHERE session_id = $1 OR client_session_id = $1
        ) r
        LEFT JOIN plugin_session_summaries s ON s.session_id = $1
        LEFT JOIN users u ON u.id = COALESCE(s.user_id, r.user_id)
        WHERE r.request_count > 0 OR s.session_id IS NOT NULL
        LIMIT 1
        "#,
        session_id.as_str()
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_session_kpis(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<SessionKpis, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*)::bigint                                   AS "request_count!",
            COUNT(DISTINCT context_id)::bigint                 AS "context_count!",
            COUNT(DISTINCT trace_id)::bigint                   AS "trace_count!",
            COUNT(*) FILTER (WHERE status = 'failed')::bigint  AS "error_count!",
            COALESCE(SUM(input_tokens), 0)::bigint             AS "total_input_tokens!",
            COALESCE(SUM(output_tokens), 0)::bigint            AS "total_output_tokens!",
            COALESCE(SUM(cost_microdollars), 0)::bigint        AS "total_cost_microdollars!"
        FROM ai_requests
        WHERE session_id = $1 OR client_session_id = $1
        "#,
        session_id.as_str()
    )
    .fetch_one(pool)
    .await?;
    Ok(SessionKpis {
        request_count: row.request_count,
        context_count: row.context_count,
        trace_count: row.trace_count,
        error_count: row.error_count,
        total_input_tokens: row.total_input_tokens,
        total_output_tokens: row.total_output_tokens,
        total_cost_microdollars: row.total_cost_microdollars,
    })
}

pub async fn list_session_contexts(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<SessionContextRow>, sqlx::Error> {
    let legacy = ContextId::legacy();
    sqlx::query_as!(
        SessionContextRow,
        r#"
        SELECT
            r.context_id                         AS "context_id!: ContextId",
            c.name                               AS "name?",
            conversation_title(r.context_id, cr.client_session_id)
                                                 AS "title!",
            COALESCE(cr.turn_count, 0)::bigint   AS "turn_count!",
            COUNT(*)::bigint                     AS "request_count!",
            MAX(r.created_at)                    AS "last_request_at?",
            MAX(r.model)                         AS "model?",
            COALESCE(SUM(r.input_tokens), 0)::bigint     AS "total_input_tokens!",
            COALESCE(SUM(r.output_tokens), 0)::bigint    AS "total_output_tokens!",
            COALESCE(SUM(r.cost_microdollars), 0)::bigint AS "cost_microdollars!",
            COUNT(*) FILTER (WHERE r.status = 'failed')::bigint AS "error_count!"
        FROM ai_requests r
        LEFT JOIN user_contexts c ON c.context_id = r.context_id
        LEFT JOIN conversation_metrics_for(ARRAY(
            SELECT DISTINCT context_id::text FROM ai_requests
            WHERE session_id = $1 OR client_session_id = $1
        )) cr ON cr.context_id = r.context_id
        WHERE (r.session_id = $1 OR r.client_session_id = $1) AND r.context_id <> $2
        GROUP BY r.context_id, c.name, cr.client_session_id, cr.turn_count
        ORDER BY MAX(r.created_at) DESC
        LIMIT 200
        "#,
        session_id.as_str(),
        legacy.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn list_session_traces(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<SessionTraceRow>, sqlx::Error> {
    sqlx::query_as!(
        SessionTraceRow,
        r#"
        SELECT
            trace_id                                            AS "trace_id!: TraceId",
            COUNT(*)::bigint                                    AS "request_count!",
            MIN(created_at)                                     AS "started_at?",
            MAX(COALESCE(completed_at, created_at))             AS "ended_at?",
            COUNT(*) FILTER (WHERE status = 'failed')::bigint   AS "error_count!"
        FROM ai_requests
        WHERE (session_id = $1 OR client_session_id = $1) AND trace_id IS NOT NULL
        GROUP BY trace_id
        ORDER BY MIN(created_at) DESC
        LIMIT 200
        "#,
        session_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn list_session_requests(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<SessionRequestRow>, sqlx::Error> {
    let legacy = ContextId::legacy();
    sqlx::query_as!(
        SessionRequestRow,
        r#"
        SELECT
            id                                  AS "id!: AiRequestId",
            NULLIF(context_id, $2)              AS "context_id?: ContextId",
            trace_id                            AS "trace_id?: TraceId",
            model                               AS "model?",
            status                              AS "status!",
            latency_ms                          AS "latency_ms?",
            cost_microdollars                   AS "cost_microdollars!",
            created_at                          AS "created_at!"
        FROM ai_requests
        WHERE session_id = $1 OR client_session_id = $1
        ORDER BY created_at DESC
        LIMIT 200
        "#,
        session_id.as_str(),
        legacy.as_str()
    )
    .fetch_all(pool)
    .await
}
