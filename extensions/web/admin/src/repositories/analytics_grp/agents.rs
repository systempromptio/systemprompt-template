use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug)]
pub struct AgentMessageRow {
    pub id: String,
    pub session_id: String,
    pub event_type: String,
    pub tool_name: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

pub async fn list_agent_messages(
    pool: &PgPool,
    agent_id: &str,
) -> Result<Vec<AgentMessageRow>, sqlx::Error> {
    let pattern = format!("%{agent_id}%");
    let rows = sqlx::query!(
        r#"SELECT id, session_id, event_type, tool_name,
                  COALESCE(metadata, '{}'::jsonb) AS "metadata!", created_at
           FROM plugin_usage_events
           WHERE metadata->>'agent_id' = $1
              OR metadata->>'agent' = $1
              OR tool_name LIKE $2
           ORDER BY created_at DESC
           LIMIT 100"#,
        agent_id,
        &pattern,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| AgentMessageRow {
            id: r.id,
            session_id: r.session_id,
            event_type: r.event_type,
            tool_name: r.tool_name,
            metadata: r.metadata,
            created_at: r.created_at,
        })
        .collect())
}

#[derive(Debug)]
pub struct AgentTraceRow {
    pub session_id: String,
    pub tool_name: String,
    pub decision: String,
    pub policy: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

pub async fn list_agent_traces(
    pool: &PgPool,
    agent_id: &str,
) -> Result<Vec<AgentTraceRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT session_id, tool_name, decision, policy, reason, created_at
           FROM governance_decisions
           WHERE agent_id = $1
           ORDER BY created_at DESC
           LIMIT 100"#,
        agent_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| AgentTraceRow {
            session_id: r.session_id,
            tool_name: r.tool_name,
            decision: r.decision,
            policy: r.policy,
            reason: r.reason,
            created_at: r.created_at,
        })
        .collect())
}

#[derive(Debug, Clone)]
pub struct AgentRow {
    pub agent_id: String,
    pub calls: i64,
    pub errors: i64,
    pub sessions: i64,
}

pub async fn list_agents(pool: &PgPool) -> Result<Vec<AgentRow>, sqlx::Error> {
    sqlx::query_as!(
        AgentRow,
        r#"SELECT
            COALESCE(metadata->>'agent_id', plugin_id, 'unknown') AS "agent_id!",
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
