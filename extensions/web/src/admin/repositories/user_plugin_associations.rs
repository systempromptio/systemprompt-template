use std::sync::Arc;

use sqlx::PgPool;

use super::super::types::UserPlugin;
use super::user_plugins::list_user_plugins;

pub async fn set_plugin_skills(
    pool: &Arc<PgPool>,
    user_plugin_id: &str,
    skill_ids: &[String],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM user_plugin_skills WHERE user_plugin_id = $1")
        .bind(user_plugin_id)
        .execute(&mut *tx)
        .await?;

    for (i, skill_id) in skill_ids.iter().enumerate() {
        let sort_order = i32::try_from(i).unwrap_or(0);
        sqlx::query(
            "INSERT INTO user_plugin_skills (user_plugin_id, user_skill_id, sort_order) VALUES ($1, $2, $3)",
        )
        .bind(user_plugin_id)
        .bind(skill_id)
        .bind(sort_order)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn set_plugin_agents(
    pool: &Arc<PgPool>,
    user_plugin_id: &str,
    agent_ids: &[String],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM user_plugin_agents WHERE user_plugin_id = $1")
        .bind(user_plugin_id)
        .execute(&mut *tx)
        .await?;

    for (i, agent_id) in agent_ids.iter().enumerate() {
        let sort_order = i32::try_from(i).unwrap_or(0);
        sqlx::query(
            "INSERT INTO user_plugin_agents (user_plugin_id, user_agent_id, sort_order) VALUES ($1, $2, $3)",
        )
        .bind(user_plugin_id)
        .bind(agent_id)
        .bind(sort_order)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn set_plugin_mcp_servers(
    pool: &Arc<PgPool>,
    user_plugin_id: &str,
    mcp_server_ids: &[String],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM user_plugin_mcp_servers WHERE user_plugin_id = $1")
        .bind(user_plugin_id)
        .execute(&mut *tx)
        .await?;

    for (i, mcp_server_id) in mcp_server_ids.iter().enumerate() {
        let sort_order = i32::try_from(i).unwrap_or(0);
        sqlx::query(
            "INSERT INTO user_plugin_mcp_servers (user_plugin_id, user_mcp_server_id, sort_order) VALUES ($1, $2, $3)",
        )
        .bind(user_plugin_id)
        .bind(mcp_server_id)
        .bind(sort_order)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn set_plugin_hooks(
    pool: &Arc<PgPool>,
    user_plugin_id: &str,
    hook_ids: &[String],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM user_plugin_hooks WHERE user_plugin_id = $1")
        .bind(user_plugin_id)
        .execute(&mut *tx)
        .await?;

    for (i, hook_id) in hook_ids.iter().enumerate() {
        let sort_order = i32::try_from(i).unwrap_or(0);
        sqlx::query(
            "INSERT INTO user_plugin_hooks (user_plugin_id, user_hook_id, sort_order) VALUES ($1, $2, $3)",
        )
        .bind(user_plugin_id)
        .bind(hook_id)
        .bind(sort_order)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn list_user_plugins_enriched(
    pool: &Arc<PgPool>,
    user_id: &str,
) -> Result<Vec<UserPluginEnriched>, sqlx::Error> {
    let plugins = list_user_plugins(pool, user_id).await?;
    if plugins.is_empty() {
        return Ok(vec![]);
    }

    let plugin_db_ids: Vec<String> = plugins.iter().map(|p| p.id.clone()).collect();

    let skill_rows: Vec<(String, String, String)> = sqlx::query_as(
        r"SELECT ups.user_plugin_id, us.id, us.name
          FROM user_plugin_skills ups
          JOIN user_skills us ON us.id = ups.user_skill_id
          WHERE ups.user_plugin_id = ANY($1)
          ORDER BY ups.sort_order",
    )
    .bind(&plugin_db_ids)
    .fetch_all(pool.as_ref())
    .await?;

    let agent_rows: Vec<(String, String, String)> = sqlx::query_as(
        r"SELECT upa.user_plugin_id, ua.id, ua.name
          FROM user_plugin_agents upa
          JOIN user_agents ua ON ua.id = upa.user_agent_id
          WHERE upa.user_plugin_id = ANY($1)
          ORDER BY upa.sort_order",
    )
    .bind(&plugin_db_ids)
    .fetch_all(pool.as_ref())
    .await?;

    let mcp_rows: Vec<(String, String, String)> = sqlx::query_as(
        r"SELECT upm.user_plugin_id, um.id, um.name
          FROM user_plugin_mcp_servers upm
          JOIN user_mcp_servers um ON um.id = upm.user_mcp_server_id
          WHERE upm.user_plugin_id = ANY($1)
          ORDER BY upm.sort_order",
    )
    .bind(&plugin_db_ids)
    .fetch_all(pool.as_ref())
    .await?;

    let hook_rows: Vec<(String, String, String, String, String, bool)> = sqlx::query_as(
        r"SELECT uph.user_plugin_id, uh.id, uh.name, uh.event, uh.matcher, uh.is_async
          FROM user_plugin_hooks uph
          JOIN user_hooks uh ON uh.id = uph.user_hook_id
          WHERE uph.user_plugin_id = ANY($1)
          ORDER BY uph.sort_order",
    )
    .bind(&plugin_db_ids)
    .fetch_all(pool.as_ref())
    .await?;

    let mut skill_map: std::collections::HashMap<String, Vec<AssociatedEntity>> =
        std::collections::HashMap::new();
    for (plugin_id, id, name) in skill_rows {
        skill_map
            .entry(plugin_id)
            .or_default()
            .push(AssociatedEntity { id, name });
    }

    let mut agent_map: std::collections::HashMap<String, Vec<AssociatedEntity>> =
        std::collections::HashMap::new();
    for (plugin_id, id, name) in agent_rows {
        agent_map
            .entry(plugin_id)
            .or_default()
            .push(AssociatedEntity { id, name });
    }

    let mut mcp_map: std::collections::HashMap<String, Vec<AssociatedEntity>> =
        std::collections::HashMap::new();
    for (plugin_id, id, name) in mcp_rows {
        mcp_map
            .entry(plugin_id)
            .or_default()
            .push(AssociatedEntity { id, name });
    }

    let mut hook_map: std::collections::HashMap<String, Vec<AssociatedHook>> =
        std::collections::HashMap::new();
    for (plugin_id, id, name, event, matcher, is_async) in hook_rows {
        hook_map.entry(plugin_id).or_default().push(AssociatedHook {
            id,
            name,
            event,
            matcher,
            is_async,
        });
    }

    let enriched = plugins
        .into_iter()
        .map(|p| {
            let skills = skill_map.remove(&p.id).unwrap_or_else(Vec::new);
            let agents = agent_map.remove(&p.id).unwrap_or_else(Vec::new);
            let mcp_servers = mcp_map.remove(&p.id).unwrap_or_else(Vec::new);
            let hooks = hook_map.remove(&p.id).unwrap_or_else(Vec::new);
            UserPluginEnriched {
                skill_count: skills.len(),
                agent_count: agents.len(),
                mcp_count: mcp_servers.len(),
                hook_count: hooks.len(),
                skills,
                agents,
                mcp_servers,
                hooks,
                plugin: p,
            }
        })
        .collect();

    Ok(enriched)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AssociatedEntity {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AssociatedHook {
    pub id: String,
    pub name: String,
    pub event: String,
    pub matcher: String,
    pub is_async: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UserPluginEnriched {
    pub plugin: UserPlugin,
    pub skills: Vec<AssociatedEntity>,
    pub agents: Vec<AssociatedEntity>,
    pub mcp_servers: Vec<AssociatedEntity>,
    pub hooks: Vec<AssociatedHook>,
    pub skill_count: usize,
    pub agent_count: usize,
    pub mcp_count: usize,
    pub hook_count: usize,
}
