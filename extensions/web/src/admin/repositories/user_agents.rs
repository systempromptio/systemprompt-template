use std::sync::Arc;

use sqlx::PgPool;

use super::super::types::{CreateUserAgentRequest, UpdateUserAgentRequest, UserAgent};

pub async fn list_user_agents(
    pool: &Arc<PgPool>,
    user_id: &str,
) -> Result<Vec<UserAgent>, sqlx::Error> {
    sqlx::query_as::<_, UserAgent>(
        r"
        SELECT id, user_id, agent_id, name, description, system_prompt, enabled, base_agent_id, created_at, updated_at
        FROM user_agents
        WHERE user_id = $1
        ORDER BY created_at DESC
        ",
    )
    .bind(user_id)
    .fetch_all(pool.as_ref())
    .await
}

pub async fn create_user_agent(
    pool: &Arc<PgPool>,
    user_id: &str,
    req: &CreateUserAgentRequest,
) -> Result<UserAgent, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query_as::<_, UserAgent>(
        r"
        INSERT INTO user_agents (id, user_id, agent_id, name, description, system_prompt, base_agent_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, user_id, agent_id, name, description, system_prompt, enabled, base_agent_id, created_at, updated_at
        ",
    )
    .bind(&id)
    .bind(user_id)
    .bind(&req.agent_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.system_prompt)
    .bind(&req.base_agent_id)
    .fetch_one(pool.as_ref())
    .await
}

pub async fn update_user_agent(
    pool: &Arc<PgPool>,
    user_id: &str,
    agent_id: &str,
    req: &UpdateUserAgentRequest,
) -> Result<Option<UserAgent>, sqlx::Error> {
    sqlx::query_as::<_, UserAgent>(
        r"
        UPDATE user_agents SET
            name = COALESCE($3, name),
            description = COALESCE($4, description),
            system_prompt = COALESCE($5, system_prompt),
            enabled = COALESCE($6, enabled),
            updated_at = NOW()
        WHERE user_id = $1 AND agent_id = $2
        RETURNING id, user_id, agent_id, name, description, system_prompt, enabled, base_agent_id, created_at, updated_at
        ",
    )
    .bind(user_id)
    .bind(agent_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.system_prompt)
    .bind(req.enabled)
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn delete_user_agent(
    pool: &Arc<PgPool>,
    user_id: &str,
    agent_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM user_agents WHERE user_id = $1 AND agent_id = $2")
        .bind(user_id)
        .bind(agent_id)
        .execute(pool.as_ref())
        .await?;
    Ok(result.rows_affected() > 0)
}
