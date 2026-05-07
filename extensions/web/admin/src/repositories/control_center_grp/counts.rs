use sqlx::PgPool;

#[derive(Debug, Clone, Copy, Default)]
pub struct EntityCountsRow {
    pub plugins: i64,
    pub skills: i64,
    pub agents: i64,
    pub mcp_servers: i64,
}

pub async fn get_entity_counts(
    pool: &PgPool,
    user_id: &str,
) -> Result<EntityCountsRow, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT
            COALESCE((SELECT COUNT(*) FROM user_plugins WHERE user_id = $1), 0)::BIGINT AS "plugins!",
            COALESCE((SELECT COUNT(*) FROM user_skills WHERE user_id = $1), 0)::BIGINT AS "skills!",
            COALESCE((SELECT COUNT(*) FROM user_agents WHERE user_id = $1), 0)::BIGINT AS "agents!",
            COALESCE((SELECT COUNT(*) FROM user_mcp_servers WHERE user_id = $1), 0)::BIGINT AS "mcp_servers!""#,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(EntityCountsRow {
        plugins: row.plugins,
        skills: row.skills,
        agents: row.agents,
        mcp_servers: row.mcp_servers,
    })
}
