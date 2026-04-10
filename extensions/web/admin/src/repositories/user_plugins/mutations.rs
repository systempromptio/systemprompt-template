use sqlx::PgPool;
use systemprompt::identifiers::{AgentId, McpServerId, SkillId, UserId};

use super::super::super::types::{CreateUserPluginRequest, UpdateUserPluginRequest, UserPlugin};

pub async fn create_user_plugin<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
    pool: E,
    user_id: &UserId,
    req: &CreateUserPluginRequest,
) -> Result<UserPlugin, sqlx::Error> {
    if req.plugin_id == "systemprompt" {
        return Err(sqlx::Error::Protocol(
            "The plugin_id 'systemprompt' is reserved for the platform plugin".into(),
        ));
    }
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query_as!(
        UserPlugin,
        r#"
        INSERT INTO user_plugins (id, user_id, plugin_id, name, description, version, category, keywords, author_name, base_plugin_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, user_id AS "user_id: _", plugin_id, name, description, version, enabled, category, COALESCE(keywords, '{}') as "keywords!", author_name, base_plugin_id, created_at, updated_at
        "#,
        id,
        user_id.as_str(),
        &req.plugin_id,
        &req.name,
        &req.description,
        &req.version,
        &req.category,
        &req.keywords,
        &req.author_name,
        req.base_plugin_id.as_deref(),
    )
    .fetch_one(pool)
    .await
}

pub async fn update_user_plugin<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
    pool: E,
    user_id: &UserId,
    plugin_id: &str,
    req: &UpdateUserPluginRequest,
) -> Result<Option<UserPlugin>, sqlx::Error> {
    sqlx::query_as!(
        UserPlugin,
        r#"
        UPDATE user_plugins SET
            name = COALESCE($3, name),
            description = COALESCE($4, description),
            version = COALESCE($5, version),
            enabled = COALESCE($6, enabled),
            category = COALESCE($7, category),
            keywords = COALESCE($8, keywords),
            author_name = COALESCE($9, author_name),
            updated_at = NOW()
        WHERE user_id = $1 AND plugin_id = $2
        RETURNING id, user_id AS "user_id: _", plugin_id, name, description, version, enabled, category, COALESCE(keywords, '{}') as "keywords!", author_name, base_plugin_id, created_at, updated_at
        "#,
        user_id.as_str(),
        plugin_id,
        req.name.as_deref(),
        req.description.as_deref(),
        req.version.as_deref(),
        req.enabled,
        req.category.as_deref(),
        req.keywords.as_deref(),
        req.author_name.as_deref(),
    )
    .fetch_optional(pool)
    .await
}

pub async fn delete_user_plugin(
    pool: &PgPool,
    user_id: &UserId,
    plugin_id: &str,
) -> Result<bool, sqlx::Error> {
    let plugin = super::queries::find_user_plugin(pool, user_id, plugin_id).await?;
    let Some(plugin) = plugin else {
        return Ok(false);
    };

    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"DELETE FROM user_skills
        WHERE id IN (
            SELECT ups.user_skill_id FROM user_plugin_skills ups
            WHERE ups.user_plugin_id = $1
            AND NOT EXISTS (
                SELECT 1 FROM user_plugin_skills ups2
                WHERE ups2.user_skill_id = ups.user_skill_id
                AND ups2.user_plugin_id != $1
            )
        ) AND updated_at <= created_at"#,
        plugin.id,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"DELETE FROM user_agents
        WHERE id IN (
            SELECT upa.user_agent_id FROM user_plugin_agents upa
            WHERE upa.user_plugin_id = $1
            AND NOT EXISTS (
                SELECT 1 FROM user_plugin_agents upa2
                WHERE upa2.user_agent_id = upa.user_agent_id
                AND upa2.user_plugin_id != $1
            )
        ) AND updated_at <= created_at"#,
        plugin.id,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"DELETE FROM user_mcp_servers
        WHERE id IN (
            SELECT upm.user_mcp_server_id FROM user_plugin_mcp_servers upm
            WHERE upm.user_plugin_id = $1
            AND NOT EXISTS (
                SELECT 1 FROM user_plugin_mcp_servers upm2
                WHERE upm2.user_mcp_server_id = upm.user_mcp_server_id
                AND upm2.user_plugin_id != $1
            )
        ) AND updated_at <= created_at"#,
        plugin.id,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!("DELETE FROM user_plugins WHERE id = $1", plugin.id)
        .execute(&mut *tx)
        .await?;

    sqlx::query!(
        "DELETE FROM plugin_env_vars WHERE user_id = $1 AND plugin_id = $2",
        user_id.as_str(),
        plugin.plugin_id,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(true)
}

pub async fn set_plugin_skills<'a, A: sqlx::Acquire<'a, Database = sqlx::Postgres>>(
    db: A,
    user_plugin_id: &str,
    skill_ids: &[SkillId],
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    sqlx::query!(
        "DELETE FROM user_plugin_skills WHERE user_plugin_id = $1",
        user_plugin_id,
    )
    .execute(&mut *tx)
    .await?;

    for (i, skill_id) in skill_ids.iter().enumerate() {
        sqlx::query!(
            "INSERT INTO user_plugin_skills (user_plugin_id, user_skill_id, sort_order) VALUES ($1, $2, $3)",
            user_plugin_id,
            skill_id.as_str(),
            i32::try_from(i).unwrap_or(0),
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn set_plugin_agents<'a, A: sqlx::Acquire<'a, Database = sqlx::Postgres>>(
    db: A,
    user_plugin_id: &str,
    agent_ids: &[AgentId],
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    sqlx::query!(
        "DELETE FROM user_plugin_agents WHERE user_plugin_id = $1",
        user_plugin_id,
    )
    .execute(&mut *tx)
    .await?;

    for (i, agent_id) in agent_ids.iter().enumerate() {
        sqlx::query!(
            "INSERT INTO user_plugin_agents (user_plugin_id, user_agent_id, sort_order) VALUES ($1, $2, $3)",
            user_plugin_id,
            agent_id.as_str(),
            i32::try_from(i).unwrap_or(0),
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn set_plugin_mcp_servers<'a, A: sqlx::Acquire<'a, Database = sqlx::Postgres>>(
    db: A,
    user_plugin_id: &str,
    mcp_server_ids: &[McpServerId],
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    sqlx::query!(
        "DELETE FROM user_plugin_mcp_servers WHERE user_plugin_id = $1",
        user_plugin_id,
    )
    .execute(&mut *tx)
    .await?;

    for (i, mcp_server_id) in mcp_server_ids.iter().enumerate() {
        sqlx::query!(
            "INSERT INTO user_plugin_mcp_servers (user_plugin_id, user_mcp_server_id, sort_order) VALUES ($1, $2, $3)",
            user_plugin_id,
            mcp_server_id.as_str(),
            i32::try_from(i).unwrap_or(0),
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
