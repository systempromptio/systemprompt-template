use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::identifiers::{AgentId, UserId};

use super::super::types::{CreateUserAgentRequest, UpdateUserAgentRequest, UserAgent};

pub async fn list_user_agents(
    pool: &Arc<PgPool>,
    user_id: &UserId,
) -> Result<Vec<UserAgent>, sqlx::Error> {
    sqlx::query_as!(
        UserAgent,
        r#"SELECT id, user_id as "user_id: UserId", agent_id as "agent_id: AgentId", name, description, system_prompt, enabled, base_agent_id as "base_agent_id: AgentId", created_at, updated_at FROM user_agents WHERE user_id = $1 ORDER BY created_at DESC"#,
        user_id as &UserId,
    )
    .fetch_all(pool.as_ref())
    .await
}

pub async fn create_user_agent(
    pool: &Arc<PgPool>,
    user_id: &UserId,
    req: &CreateUserAgentRequest,
) -> Result<UserAgent, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query_as!(
        UserAgent,
        r#"INSERT INTO user_agents (id, user_id, agent_id, name, description, system_prompt, base_agent_id) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id, user_id as "user_id: UserId", agent_id as "agent_id: AgentId", name, description, system_prompt, enabled, base_agent_id as "base_agent_id: AgentId", created_at, updated_at"#,
        &id,
        user_id as &UserId,
        &req.agent_id as &AgentId,
        &req.name,
        &req.description,
        &req.system_prompt,
        req.base_agent_id.as_ref() as Option<&AgentId>,
    )
    .fetch_one(pool.as_ref())
    .await
}

pub async fn get_or_create_user_agent(
    pool: &Arc<PgPool>,
    user_id: &UserId,
    req: &CreateUserAgentRequest,
) -> Result<UserAgent, sqlx::Error> {
    match create_user_agent(pool, user_id, req).await {
        Ok(agent) => Ok(agent),
        Err(_) => {
            sqlx::query_as!(
                UserAgent,
                r#"SELECT id, user_id as "user_id: UserId", agent_id as "agent_id: AgentId", name, description, system_prompt, enabled, base_agent_id as "base_agent_id: AgentId", created_at, updated_at FROM user_agents WHERE user_id = $1 AND agent_id = $2"#,
                user_id as &UserId,
                &req.agent_id as &AgentId,
            )
            .fetch_one(pool.as_ref())
            .await
        }
    }
}

pub async fn update_user_agent(
    pool: &Arc<PgPool>,
    user_id: &UserId,
    agent_id: &AgentId,
    req: &UpdateUserAgentRequest,
) -> Result<Option<UserAgent>, sqlx::Error> {
    sqlx::query_as!(
        UserAgent,
        r#"UPDATE user_agents SET name = COALESCE($3, name), description = COALESCE($4, description), system_prompt = COALESCE($5, system_prompt), updated_at = NOW() WHERE user_id = $1 AND agent_id = $2 RETURNING id, user_id as "user_id: UserId", agent_id as "agent_id: AgentId", name, description, system_prompt, enabled, base_agent_id as "base_agent_id: AgentId", created_at, updated_at"#,
        user_id as &UserId,
        agent_id as &AgentId,
        req.name.as_deref(),
        req.description.as_deref(),
        req.system_prompt.as_deref(),
    )
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn fetch_agent_plugin_assignments(
    pool: &Arc<PgPool>,
    user_id: &UserId,
) -> Result<std::collections::HashMap<String, Vec<String>>, sqlx::Error> {
    let rows = sqlx::query!(
        "SELECT ua.agent_id, up.name FROM user_plugin_agents upa JOIN user_plugins up ON up.id = upa.user_plugin_id JOIN user_agents ua ON ua.id = upa.user_agent_id WHERE up.user_id = $1",
        user_id as &UserId,
    )
    .fetch_all(pool.as_ref())
    .await?;

    let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for row in rows {
        map.entry(row.agent_id).or_default().push(row.name);
    }
    Ok(map)
}

pub async fn delete_user_agent(
    pool: &Arc<PgPool>,
    user_id: &UserId,
    agent_id: &AgentId,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM user_agents WHERE user_id = $1 AND agent_id = $2",
        user_id as &UserId,
        agent_id as &AgentId,
    )
    .execute(pool.as_ref())
    .await?;
    Ok(result.rows_affected() > 0)
}
