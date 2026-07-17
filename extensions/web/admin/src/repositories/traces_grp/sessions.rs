use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{AgentId, AiRequestId, McpExecutionId, SessionId, TraceId};

#[derive(Debug)]
pub struct TraceEvent {
    pub id: String,
    pub event_type: String,
    pub tool_name: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct TraceGovernanceRow {
    pub tool_name: String,
    pub agent_id: Option<AgentId>,
    pub agent_scope: Option<String>,
    pub decision: String,
    pub policy: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct TraceEntity {
    pub entity_type: String,
    pub entity_name: String,
    pub usage_count: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct SessionSummaryRow {
    pub total_events: i64,
    pub tool_uses: i64,
    pub prompts: i64,
    pub errors: i64,
}

#[derive(Debug)]
pub struct AiCallRow {
    pub request_id: AiRequestId,
    pub model: String,
    pub provider: String,
    pub status: String,
    pub latency_ms: Option<i32>,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cost_microdollars: i64,
    pub trace_id: Option<TraceId>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct AiMessageRow {
    pub request_id: AiRequestId,
    pub role: String,
    pub sequence_number: i32,
    pub content: String,
}

#[derive(Debug)]
pub struct AiToolCallRow {
    pub request_id: AiRequestId,
    pub tool_name: String,
    pub sequence_number: i32,
    pub tool_input: String,
    pub tool_result_payload: Option<serde_json::Value>,
    pub mcp_execution_id: Option<McpExecutionId>,
}

pub async fn fetch_trace_events(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<TraceEvent>, sqlx::Error> {
    sqlx::query_as!(
        TraceEvent,
        r#"
        SELECT
            id                                                                       AS "id!",
            event_type                                                               AS "event_type!",
            tool_name                                                                AS "tool_name?",
            metadata                                                                 AS "metadata!: serde_json::Value",
            created_at                                                               AS "created_at!"
          FROM (
            SELECT id, event_type, tool_name,
                   COALESCE(metadata, '{}'::jsonb) AS metadata,
                   created_at
              FROM plugin_usage_events
             WHERE session_id = $1
            UNION ALL
            SELECT
                r.id::text                                                           AS id,
                'AIRequest'                                                          AS event_type,
                r.model                                                              AS tool_name,
                jsonb_strip_nulls(jsonb_build_object(
                    'model',                r.model,
                    'provider',             r.provider,
                    'status',               r.status,
                    'latency_ms',           r.latency_ms,
                    'input_tokens',         r.input_tokens,
                    'output_tokens',        r.output_tokens,
                    'cost_microdollars',    r.cost_microdollars,
                    'trace_id',             r.trace_id
                ))                                                                   AS metadata,
                r.created_at                                                         AS created_at
              FROM ai_requests r
             WHERE r.session_id = $1
          ) AS t
         ORDER BY created_at ASC
         LIMIT 500
        "#,
        session_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_trace_governance(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<TraceGovernanceRow>, sqlx::Error> {
    sqlx::query_as!(
        TraceGovernanceRow,
        r#"SELECT tool_name, agent_id AS "agent_id: AgentId", agent_scope, decision, policy, reason, created_at
           FROM governance_decisions
           WHERE session_id = $1
           ORDER BY created_at ASC
           LIMIT 100"#,
        session_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_trace_entities(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<TraceEntity>, sqlx::Error> {
    sqlx::query_as!(
        TraceEntity,
        r#"SELECT entity_type, entity_name, usage_count
           FROM session_entity_links
           WHERE session_id = $1
           ORDER BY usage_count DESC
           LIMIT 50"#,
        session_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_session_summary(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<SessionSummaryRow, sqlx::Error> {
    sqlx::query_as!(
        SessionSummaryRow,
        r#"
        WITH plug AS (
            SELECT
                COUNT(*)                                                                                                AS total,
                COUNT(*) FILTER (WHERE event_type LIKE '%ToolUse%')                                                      AS tool_uses,
                COUNT(*) FILTER (WHERE event_type LIKE '%Prompt%' OR event_type LIKE '%Submit%')                         AS prompts,
                COUNT(*) FILTER (WHERE event_type LIKE '%Failure%' OR event_type LIKE '%Error%')                         AS errors
              FROM plugin_usage_events
             WHERE session_id = $1
        ),
        ai AS (
            SELECT
                COUNT(*)                                                                                                AS total,
                COUNT(*) FILTER (WHERE status NOT IN ('completed', 'pending', 'streaming', 'ok', 'success'))             AS errors
              FROM ai_requests
             WHERE session_id = $1
        )
        SELECT
            (plug.total + ai.total)::bigint     AS "total_events!",
            plug.tool_uses::bigint              AS "tool_uses!",
            plug.prompts::bigint                AS "prompts!",
            (plug.errors + ai.errors)::bigint   AS "errors!"
          FROM plug, ai
        "#,
        session_id.as_str()
    )
    .fetch_one(pool)
    .await
}

pub async fn fetch_trace_ai_calls(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<AiCallRow>, sqlx::Error> {
    sqlx::query_as!(
        AiCallRow,
        r#"
        SELECT
            id              AS "request_id!: AiRequestId",
            model           AS "model!",
            provider        AS "provider!",
            status          AS "status!",
            latency_ms,
            input_tokens,
            output_tokens,
            cost_microdollars AS "cost_microdollars!",
            trace_id        AS "trace_id: TraceId",
            created_at      AS "created_at!"
          FROM ai_requests
         WHERE session_id = $1
         ORDER BY created_at ASC
         LIMIT 200
        "#,
        session_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_trace_ai_messages(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<AiMessageRow>, sqlx::Error> {
    sqlx::query_as!(
        AiMessageRow,
        r#"
        SELECT
            m.request_id      AS "request_id!: AiRequestId",
            m.role            AS "role!",
            m.sequence_number AS "sequence_number!",
            m.content         AS "content!"
          FROM ai_request_messages m
          JOIN ai_requests r ON r.id = m.request_id
         WHERE r.session_id = $1
         ORDER BY r.created_at ASC, m.sequence_number ASC
         LIMIT 1000
        "#,
        session_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_trace_ai_tool_calls(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<AiToolCallRow>, sqlx::Error> {
    sqlx::query_as!(
        AiToolCallRow,
        r#"
        SELECT
            t.request_id          AS "request_id!: AiRequestId",
            t.tool_name           AS "tool_name!",
            t.sequence_number     AS "sequence_number!",
            t.tool_input          AS "tool_input!",
            t.tool_result_payload AS "tool_result_payload?: serde_json::Value",
            t.mcp_execution_id    AS "mcp_execution_id: McpExecutionId"
          FROM ai_request_tool_calls t
          JOIN ai_requests r ON r.id = t.request_id
         WHERE r.session_id = $1
         ORDER BY r.created_at ASC, t.sequence_number ASC
         LIMIT 500
        "#,
        session_id.as_str()
    )
    .fetch_all(pool)
    .await
}
