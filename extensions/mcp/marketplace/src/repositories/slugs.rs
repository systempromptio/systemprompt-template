#[derive(Debug)]
pub struct SkillSlugRow {
    pub skill_id: String,
    pub id: String,
}

pub async fn list_user_skill_slugs<'e, E>(
    executor: E,
    user_id: &str,
    slugs: &[String],
) -> Result<Vec<SkillSlugRow>, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let rows = sqlx::query!(
        "SELECT skill_id, id FROM user_skills WHERE user_id = $1 AND skill_id = ANY($2)",
        user_id,
        slugs,
    )
    .fetch_all(executor)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| SkillSlugRow {
            skill_id: r.skill_id,
            id: r.id,
        })
        .collect())
}

#[derive(Debug)]
pub struct AgentSlugRow {
    pub agent_id: String,
    pub id: String,
}

pub async fn list_user_agent_slugs<'e, E>(
    executor: E,
    user_id: &str,
    slugs: &[String],
) -> Result<Vec<AgentSlugRow>, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let rows = sqlx::query!(
        "SELECT agent_id, id FROM user_agents WHERE user_id = $1 AND agent_id = ANY($2)",
        user_id,
        slugs,
    )
    .fetch_all(executor)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| AgentSlugRow {
            agent_id: r.agent_id,
            id: r.id,
        })
        .collect())
}

#[derive(Debug)]
pub struct McpServerSlugRow {
    pub mcp_server_id: String,
    pub id: String,
}

pub async fn list_user_mcp_server_slugs<'e, E>(
    executor: E,
    user_id: &str,
    slugs: &[String],
) -> Result<Vec<McpServerSlugRow>, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let rows = sqlx::query!(
        "SELECT mcp_server_id, id FROM user_mcp_servers WHERE user_id = $1 AND mcp_server_id = ANY($2)",
        user_id,
        slugs,
    )
    .fetch_all(executor)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| McpServerSlugRow {
            mcp_server_id: r.mcp_server_id,
            id: r.id,
        })
        .collect())
}

#[derive(Debug)]
pub struct UuidSlugRow {
    pub id: String,
    pub slug: String,
}

pub async fn list_user_skill_uuids<'e, E>(
    executor: E,
    uuids: &[String],
) -> Result<Vec<UuidSlugRow>, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let rows = sqlx::query!(
        "SELECT id, skill_id FROM user_skills WHERE id = ANY($1)",
        uuids
    )
    .fetch_all(executor)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| UuidSlugRow {
            id: r.id,
            slug: r.skill_id,
        })
        .collect())
}

pub async fn list_user_agent_uuids<'e, E>(
    executor: E,
    uuids: &[String],
) -> Result<Vec<UuidSlugRow>, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let rows = sqlx::query!(
        "SELECT id, agent_id FROM user_agents WHERE id = ANY($1)",
        uuids
    )
    .fetch_all(executor)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| UuidSlugRow {
            id: r.id,
            slug: r.agent_id,
        })
        .collect())
}

pub async fn list_user_mcp_server_uuids<'e, E>(
    executor: E,
    uuids: &[String],
) -> Result<Vec<UuidSlugRow>, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let rows = sqlx::query!(
        "SELECT id, mcp_server_id FROM user_mcp_servers WHERE id = ANY($1)",
        uuids
    )
    .fetch_all(executor)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| UuidSlugRow {
            id: r.id,
            slug: r.mcp_server_id,
        })
        .collect())
}
