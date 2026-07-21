use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{AgentId, SessionId};

#[derive(Debug)]
pub struct AgentMessageRow {
    pub id: String,
    pub session_id: SessionId,
    pub event_type: String,
    pub tool_name: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct AgentTraceRow {
    pub session_id: SessionId,
    pub tool_name: String,
    pub decision: String,
    pub policy: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AgentRow {
    pub agent_id: AgentId,
    pub calls: i64,
    pub errors: i64,
    pub sessions: i64,
}

pub async fn list_agents(pool: &PgPool) -> Result<Vec<AgentRow>, sqlx::Error> {
    sqlx::query_as!(
        AgentRow,
        r#"SELECT
            COALESCE(metadata->>'agent_id', plugin_id, 'unknown') AS "agent_id!: AgentId",
            COUNT(*)::bigint AS "calls!",
            COUNT(*) FILTER (WHERE event_type LIKE '%Failure%' OR event_type LIKE '%Error%')::bigint AS "errors!",
            COUNT(DISTINCT session_id)::bigint AS "sessions!"
          FROM plugin_usage_events
          WHERE created_at >= NOW() - INTERVAL '7 days'
            AND (metadata->>'agent_id' IS NOT NULL OR plugin_id IS NOT NULL)
          GROUP BY COALESCE(metadata->>'agent_id', plugin_id, 'unknown')
          ORDER BY COUNT(*) DESC
          LIMIT 50"#,
    )
    .fetch_all(pool)
    .await
}
