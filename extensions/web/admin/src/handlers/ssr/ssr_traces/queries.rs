use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug)]
pub(super) struct TraceEvent {
    pub id: String,
    pub event_type: String,
    pub tool_name: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub(super) struct TraceGovernanceRow {
    pub tool_name: String,
    pub agent_id: Option<String>,
    pub agent_scope: Option<String>,
    pub decision: String,
    pub policy: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub(super) struct TraceEntity {
    pub entity_type: String,
    pub entity_name: String,
    pub usage_count: i32,
}

#[derive(Debug)]
pub(super) struct SessionSummaryRow {
    pub total_events: i64,
    pub tool_uses: i64,
    pub prompts: i64,
    pub errors: i64,
}

pub(super) async fn fetch_trace_events(
    pool: &PgPool,
    session_id: &str,
) -> Result<Vec<TraceEvent>, sqlx::Error> {
    sqlx::query_as!(
        TraceEvent,
        r#"SELECT id, event_type, tool_name,
                  COALESCE(metadata, '{}'::jsonb) AS "metadata!: serde_json::Value",
                  created_at AS "created_at!"
           FROM plugin_usage_events
           WHERE session_id = $1
           ORDER BY created_at ASC
           LIMIT 500"#,
        session_id
    )
    .fetch_all(pool)
    .await
}

pub(super) async fn fetch_trace_governance(
    pool: &PgPool,
    session_id: &str,
) -> Result<Vec<TraceGovernanceRow>, sqlx::Error> {
    sqlx::query_as!(
        TraceGovernanceRow,
        r#"SELECT tool_name, agent_id, agent_scope, decision, policy, reason, created_at
           FROM governance_decisions
           WHERE session_id = $1
           ORDER BY created_at ASC
           LIMIT 100"#,
        session_id
    )
    .fetch_all(pool)
    .await
}

pub(super) async fn fetch_trace_entities(
    pool: &PgPool,
    session_id: &str,
) -> Result<Vec<TraceEntity>, sqlx::Error> {
    sqlx::query_as!(
        TraceEntity,
        r#"SELECT entity_type, entity_name, usage_count
           FROM session_entity_links
           WHERE session_id = $1
           ORDER BY usage_count DESC
           LIMIT 50"#,
        session_id
    )
    .fetch_all(pool)
    .await
}

pub(super) async fn fetch_session_summary(
    pool: &PgPool,
    session_id: &str,
) -> Result<SessionSummaryRow, sqlx::Error> {
    sqlx::query_as!(
        SessionSummaryRow,
        r#"SELECT
            COUNT(*)::bigint AS "total_events!",
            COUNT(*) FILTER (WHERE event_type LIKE '%ToolUse%')::bigint AS "tool_uses!",
            COUNT(*) FILTER (WHERE event_type LIKE '%Prompt%' OR event_type LIKE '%Submit%')::bigint AS "prompts!",
            COUNT(*) FILTER (WHERE event_type LIKE '%Failure%' OR event_type LIKE '%Error%')::bigint AS "errors!"
           FROM plugin_usage_events
           WHERE session_id = $1"#,
        session_id
    )
    .fetch_one(pool)
    .await
}
