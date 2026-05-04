use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug)]
pub struct RecentToolExecution {
    pub tool_name: String,
    pub status: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct ToolByAgent {
    pub agent_id: String,
    pub tool_name: String,
    pub usage_count: i64,
}

pub async fn list_recent_tool_executions(
    pool: &PgPool,
) -> Result<Vec<RecentToolExecution>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT tool_name, status, user_id, created_at
           FROM mcp_tool_executions
           ORDER BY created_at DESC
           LIMIT 50"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| RecentToolExecution {
            tool_name: r.tool_name,
            status: r.status,
            user_id: r.user_id,
            created_at: r.created_at,
        })
        .collect())
}

pub async fn list_tools_by_agent(pool: &PgPool) -> Result<Vec<ToolByAgent>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT COALESCE(agent_id, 'unknown') AS "agent_id!",
                  COALESCE(tool_name, 'unknown') AS "tool_name!",
                  COUNT(*)::BIGINT AS "usage_count!"
           FROM governance_decisions
           WHERE agent_id IS NOT NULL AND decision = 'allow'
           GROUP BY agent_id, tool_name
           ORDER BY COUNT(*) DESC
           LIMIT 30"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ToolByAgent {
            agent_id: r.agent_id,
            tool_name: r.tool_name,
            usage_count: r.usage_count,
        })
        .collect())
}
