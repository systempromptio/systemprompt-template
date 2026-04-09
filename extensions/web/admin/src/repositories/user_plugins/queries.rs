use sqlx::PgPool;
use systemprompt::identifiers::{AgentId, McpServerId, SkillId, UserId};

use super::super::super::types::{UserPlugin, UserPluginWithAssociations};
use super::types::{AssociatedEntity, UserPluginEnriched};

pub async fn list_user_plugins(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<UserPlugin>, sqlx::Error> {
    sqlx::query_as!(
        UserPlugin,
        r#"
        SELECT id, user_id AS "user_id: _", plugin_id, name, description, version, enabled, category, COALESCE(keywords, '{}') as "keywords!", author_name, base_plugin_id, created_at, updated_at
        FROM user_plugins
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

pub async fn find_user_plugin(
    pool: &PgPool,
    user_id: &UserId,
    plugin_id: &str,
) -> Result<Option<UserPlugin>, sqlx::Error> {
    sqlx::query_as!(
        UserPlugin,
        r#"
        SELECT id, user_id AS "user_id: _", plugin_id, name, description, version, enabled, category, COALESCE(keywords, '{}') as "keywords!", author_name, base_plugin_id, created_at, updated_at
        FROM user_plugins
        WHERE user_id = $1 AND plugin_id = $2
        "#,
        user_id.as_str(),
        plugin_id,
    )
    .fetch_optional(pool)
    .await
}

pub async fn count_user_plugin_items(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<crate::types::UserPluginCounts, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT
            COUNT(DISTINCT up.id)::BIGINT AS "plugins!",
            COUNT(DISTINCT ups.user_skill_id)::BIGINT AS "skills!",
            COUNT(DISTINCT upa.user_agent_id)::BIGINT AS "agents!",
            COUNT(DISTINCT upm.user_mcp_server_id)::BIGINT AS "mcp_servers!"
          FROM user_plugins up
          LEFT JOIN user_plugin_skills ups ON ups.user_plugin_id = up.id
          LEFT JOIN user_plugin_agents upa ON upa.user_plugin_id = up.id
          LEFT JOIN user_plugin_mcp_servers upm ON upm.user_plugin_id = up.id
          WHERE up.user_id = $1"#,
        user_id.as_str(),
    )
    .fetch_one(pool)
    .await?;
    Ok(crate::types::UserPluginCounts {
        plugins: usize::try_from(row.plugins).unwrap_or(0),
        skills: usize::try_from(row.skills).unwrap_or(0),
        agents: usize::try_from(row.agents).unwrap_or(0),
        mcp_servers: usize::try_from(row.mcp_servers).unwrap_or(0),
    })
}

struct PluginEntityRow {
    plugin_id: String,
    entity_id: String,
    entity_name: String,
}

pub async fn list_user_plugins_enriched(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<UserPluginEnriched>, sqlx::Error> {
    let plugins = list_user_plugins(pool, user_id).await?;
    if plugins.is_empty() {
        return Ok(vec![]);
    }

    let plugin_db_ids: Vec<String> = plugins.iter().map(|p| p.id.clone()).collect();

    let skill_rows = sqlx::query_as!(
        PluginEntityRow,
        r#"SELECT ups.user_plugin_id AS "plugin_id!", us.skill_id AS "entity_id!", us.name AS "entity_name!"
          FROM user_plugin_skills ups
          JOIN user_skills us ON us.id = ups.user_skill_id
          WHERE ups.user_plugin_id = ANY($1)
          ORDER BY ups.sort_order"#,
        &plugin_db_ids,
    )
    .fetch_all(pool)
    .await?;

    let agent_rows = sqlx::query_as!(
        PluginEntityRow,
        r#"SELECT upa.user_plugin_id AS "plugin_id!", ua.agent_id AS "entity_id!", ua.name AS "entity_name!"
          FROM user_plugin_agents upa
          JOIN user_agents ua ON ua.id = upa.user_agent_id
          WHERE upa.user_plugin_id = ANY($1)
          ORDER BY upa.sort_order"#,
        &plugin_db_ids,
    )
    .fetch_all(pool)
    .await?;

    let mcp_rows = sqlx::query_as!(
        PluginEntityRow,
        r#"SELECT upm.user_plugin_id AS "plugin_id!", um.mcp_server_id AS "entity_id!", um.name AS "entity_name!"
          FROM user_plugin_mcp_servers upm
          JOIN user_mcp_servers um ON um.id = upm.user_mcp_server_id
          WHERE upm.user_plugin_id = ANY($1)
          ORDER BY upm.sort_order"#,
        &plugin_db_ids,
    )
    .fetch_all(pool)
    .await?;

    let mut skill_map = build_entity_map(skill_rows);
    let mut agent_map = build_entity_map(agent_rows);
    let mut mcp_map = build_entity_map(mcp_rows);

    let enriched = plugins
        .into_iter()
        .map(|p| {
            let skills = skill_map.remove(&p.id).unwrap_or_else(Vec::new);
            let agents = agent_map.remove(&p.id).unwrap_or_else(Vec::new);
            let mcp_servers = mcp_map.remove(&p.id).unwrap_or_else(Vec::new);
            UserPluginEnriched {
                skill_count: skills.len(),
                agent_count: agents.len(),
                mcp_count: mcp_servers.len(),
                skills,
                agents,
                mcp_servers,
                plugin: p,
            }
        })
        .collect();

    Ok(enriched)
}

fn build_entity_map(
    rows: Vec<PluginEntityRow>,
) -> std::collections::HashMap<String, Vec<AssociatedEntity>> {
    let mut map: std::collections::HashMap<String, Vec<AssociatedEntity>> =
        std::collections::HashMap::new();
    for row in rows {
        map.entry(row.plugin_id)
            .or_default()
            .push(AssociatedEntity {
                id: row.entity_id,
                name: row.entity_name,
            });
    }
    map
}

pub async fn is_entity_in_platform_plugin(
    pool: &PgPool,
    user_id: &UserId,
    entity_id: &str,
    entity_kind: &str,
) -> bool {
    match entity_kind {
        "skill" => sqlx::query_scalar!(
            r#"SELECT 1 AS "v!" FROM user_plugin_skills ups
               JOIN user_plugins up ON up.id = ups.user_plugin_id
               WHERE up.user_id = $1 AND ups.user_skill_id = $2
               AND up.base_plugin_id = 'systemprompt'"#,
            user_id.as_str(),
            entity_id,
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user_id.as_str(), entity_id = %entity_id, "Failed to check skill entity ownership");
        })
        .ok()
        .flatten()
        .is_some(),
        "agent" => sqlx::query_scalar!(
            r#"SELECT 1 AS "v!" FROM user_plugin_agents upa
               JOIN user_plugins up ON up.id = upa.user_plugin_id
               WHERE up.user_id = $1 AND upa.user_agent_id = $2
               AND up.base_plugin_id = 'systemprompt'"#,
            user_id.as_str(),
            entity_id,
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user_id.as_str(), entity_id = %entity_id, "Failed to check agent entity ownership");
        })
        .ok()
        .flatten()
        .is_some(),
        _ => false,
    }
}

pub async fn find_plugin_with_associations(
    pool: &PgPool,
    user_id: &UserId,
    plugin_id: &str,
) -> Result<Option<UserPluginWithAssociations>, sqlx::Error> {
    let plugin = find_user_plugin(pool, user_id, plugin_id).await?;
    let Some(plugin) = plugin else {
        return Ok(None);
    };

    let skill_rows = sqlx::query!(
        "SELECT user_skill_id FROM user_plugin_skills WHERE user_plugin_id = $1 ORDER BY sort_order",
        &plugin.id,
    )
    .fetch_all(pool)
    .await?;
    let skill_ids: Vec<SkillId> = skill_rows
        .into_iter()
        .map(|r| SkillId::new(r.user_skill_id))
        .collect();

    let agent_rows = sqlx::query!(
        "SELECT user_agent_id FROM user_plugin_agents WHERE user_plugin_id = $1 ORDER BY sort_order",
        &plugin.id,
    )
    .fetch_all(pool)
    .await?;
    let agent_ids: Vec<AgentId> = agent_rows
        .into_iter()
        .map(|r| AgentId::new(r.user_agent_id))
        .collect();

    let mcp_rows = sqlx::query!(
        "SELECT user_mcp_server_id FROM user_plugin_mcp_servers WHERE user_plugin_id = $1 ORDER BY sort_order",
        &plugin.id,
    )
    .fetch_all(pool)
    .await?;
    let mcp_server_ids: Vec<McpServerId> = mcp_rows
        .into_iter()
        .map(|r| McpServerId::new(r.user_mcp_server_id))
        .collect();

    Ok(Some(UserPluginWithAssociations {
        plugin,
        skill_ids,
        agent_ids,
        mcp_server_ids,
    }))
}
