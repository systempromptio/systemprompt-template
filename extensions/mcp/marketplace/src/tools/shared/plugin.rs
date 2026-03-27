use systemprompt::identifiers::{AgentId, McpServerId, SkillId, UserId};
use systemprompt_web_extension::admin::repositories::{
    find_plugin_with_associations, set_plugin_agents, set_plugin_mcp_servers, set_plugin_skills,
    user_plugins,
};

pub async fn add_to_plugin(
    db_pool: &systemprompt::database::DbPool,
    user_id: &UserId,
    entity_id: &str,
    entity_kind: &str,
    target_plugin_id: Option<&str>,
) -> Option<String> {
    let pool = db_pool.write_pool()?;

    if let Some(plugin_id) = target_plugin_id {
        let assoc = find_plugin_with_associations(&pool, user_id, plugin_id)
            .await
            .map_err(|e| {
                tracing::warn!(error = %e, plugin_id = %plugin_id, "Failed to fetch plugin associations for target plugin");
            })
            .ok()
            .flatten();

        if let Some(assoc) = assoc {
            let result = match entity_kind {
                "skill" => {
                    let mut ids: Vec<SkillId> = assoc.skill_ids;
                    let new_id = SkillId::new(entity_id);
                    if !ids.iter().any(|id| id.as_ref() == entity_id) {
                        ids.push(new_id);
                    }
                    set_plugin_skills(&pool, &assoc.plugin.id, &ids).await
                }
                "agent" => {
                    let mut ids: Vec<AgentId> = assoc.agent_ids;
                    let new_id = AgentId::new(entity_id);
                    if !ids.iter().any(|id| id.as_ref() == entity_id) {
                        ids.push(new_id);
                    }
                    set_plugin_agents(&pool, &assoc.plugin.id, &ids).await
                }
                "mcp_server" => {
                    let mut ids: Vec<McpServerId> = assoc.mcp_server_ids;
                    let new_id = McpServerId::new(entity_id);
                    if !ids.iter().any(|id| id.as_ref() == entity_id) {
                        ids.push(new_id);
                    }
                    set_plugin_mcp_servers(&pool, &assoc.plugin.id, &ids).await
                }
                _ => return None,
            };

            return match result {
                Ok(()) => Some(plugin_id.to_string()),
                Err(e) => {
                    tracing::warn!(error = %e, plugin_id = %plugin_id, "Failed to add {entity_kind} to target plugin");
                    None
                }
            };
        }
        tracing::warn!(plugin_id = %plugin_id, "Target plugin not found, falling back to default");
    }

    auto_add_to_default_plugin(db_pool, user_id, entity_id, entity_kind).await
}

pub async fn auto_add_to_default_plugin(
    db_pool: &systemprompt::database::DbPool,
    user_id: &UserId,
    entity_id: &str,
    entity_kind: &str,
) -> Option<String> {
    let pool = db_pool.write_pool()?;

    let plugins = match user_plugins::list_user_plugins(&pool, user_id).await {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to list user plugins");
            return None;
        }
    };

    if plugins.is_empty() {
        tracing::warn!("No plugins found for user, cannot auto-add {entity_kind}");
        return None;
    }

    let first_plugin = plugins.last()?;
    let assoc = find_plugin_with_associations(&pool, user_id, &first_plugin.plugin_id)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, plugin_id = %first_plugin.plugin_id, "Failed to fetch plugin associations for default plugin");
        })
        .ok()
        .flatten()?;

    let result = match entity_kind {
        "skill" => {
            let mut ids: Vec<SkillId> = assoc.skill_ids;
            let new_id = SkillId::new(entity_id);
            if !ids.iter().any(|id| id.as_ref() == entity_id) {
                ids.push(new_id);
            }
            set_plugin_skills(&pool, &first_plugin.id, &ids).await
        }
        "agent" => {
            let mut ids: Vec<AgentId> = assoc.agent_ids;
            let new_id = AgentId::new(entity_id);
            if !ids.iter().any(|id| id.as_ref() == entity_id) {
                ids.push(new_id);
            }
            set_plugin_agents(&pool, &first_plugin.id, &ids).await
        }
        "mcp_server" => {
            let mut ids: Vec<McpServerId> = assoc.mcp_server_ids;
            let new_id = McpServerId::new(entity_id);
            if !ids.iter().any(|id| id.as_ref() == entity_id) {
                ids.push(new_id);
            }
            set_plugin_mcp_servers(&pool, &first_plugin.id, &ids).await
        }
        _ => return None,
    };

    match result {
        Ok(()) => Some(first_plugin.plugin_id.clone()),
        Err(e) => {
            tracing::warn!(error = %e, plugin_id = %first_plugin.plugin_id, "Failed to auto-add {entity_kind} to plugin");
            None
        }
    }
}
