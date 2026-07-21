//! Session-detail repository — drives `/admin/sessions/{id}`.
//!
//! A session groups every AI request, context, and trace produced by a single
//! interactive run. This module assembles the header row from
//! `plugin_session_summaries` (when present) plus an `ai_requests` rollup, and
//! returns the contexts/traces/requests that belong to the session.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, PluginId, SessionId, TraceId, UserId};

#[derive(Debug, Clone)]
pub struct SessionHeader {
    pub session_id: SessionId,
    pub user_id: Option<UserId>,
    pub display_name: Option<String>,
    pub department: Option<String>,
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
    pub request_count: i64,
    pub last_request_at: Option<DateTime<Utc>>,
    pub model: Option<String>,
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
    pub id: String,
    pub context_id: Option<ContextId>,
    pub trace_id: Option<TraceId>,
    pub model: String,
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
            COALESCE(s.session_id, r.session_id) AS "session_id!: SessionId",
            COALESCE(s.user_id, r.user_id)       AS "user_id?: UserId",
            u.display_name                       AS "display_name?",
            upe.department                       AS "department?",
            COALESCE(s.started_at, r.first_seen) AS "started_at?",
            COALESCE(s.ended_at, r.last_seen)    AS "last_activity_at?",
            s.status                             AS "status?",
            COALESCE(s.model, r.model)           AS "model?",
            s.plugin_id                          AS "plugin_id?: PluginId",
            s.ai_title                           AS "ai_title?"
        FROM (
            SELECT
                session_id,
                MAX(user_id) AS user_id,
                MIN(created_at) AS first_seen,
                MAX(created_at) AS last_seen,
                MAX(model) AS model
            FROM ai_requests
            WHERE session_id = $1
            GROUP BY session_id
        ) r
        FULL OUTER JOIN plugin_session_summaries s
          ON s.session_id = r.session_id
        LEFT JOIN users u ON u.id = COALESCE(s.user_id, r.user_id)
        LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        WHERE COALESCE(s.session_id, r.session_id) = $1
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
        WHERE session_id = $1
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
    sqlx::query_as!(
        SessionContextRow,
        r#"
        SELECT
            r.context_id                         AS "context_id!: ContextId",
            c.name                               AS "name?",
            COUNT(*)::bigint                     AS "request_count!",
            MAX(r.created_at)                    AS "last_request_at?",
            MAX(r.model)                         AS "model?"
        FROM ai_requests r
        LEFT JOIN user_contexts c ON c.context_id = r.context_id
        WHERE r.session_id = $1 AND r.context_id IS NOT NULL
        GROUP BY r.context_id, c.name
        ORDER BY MAX(r.created_at) DESC
        LIMIT 200
        "#,
        session_id.as_str()
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
        WHERE session_id = $1 AND trace_id IS NOT NULL
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
    sqlx::query_as!(
        SessionRequestRow,
        r#"
        SELECT
            id                                  AS "id!",
            context_id                          AS "context_id?: ContextId",
            trace_id                            AS "trace_id?: TraceId",
            model                               AS "model!",
            status                              AS "status!",
            latency_ms                          AS "latency_ms?",
            cost_microdollars                   AS "cost_microdollars!",
            created_at                          AS "created_at!"
        FROM ai_requests
        WHERE session_id = $1
        ORDER BY created_at DESC
        LIMIT 200
        "#,
        session_id.as_str()
    )
    .fetch_all(pool)
    .await
}
