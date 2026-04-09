use systemprompt::identifiers::{AgentId, McpServerId, SkillId, UserId};
use systemprompt_web_extension::admin::repositories::{
    find_plugin_with_associations, set_plugin_agents, set_plugin_mcp_servers, set_plugin_skills,
    user_plugins,
};

/// Appends `new_id` to `ids` if not already present (by string comparison).
fn push_if_absent<Id: AsRef<str>>(ids: &mut Vec<Id>, new_id: Id, entity_id: &str) {
    if !ids.iter().any(|id| id.as_ref() == entity_id) {
        ids.push(new_id);
    }
}

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
                    let mut ids = assoc.skill_ids;
                    push_if_absent(&mut ids, SkillId::new(entity_id), entity_id);
                    set_plugin_skills(&pool, &assoc.plugin.id, &ids).await
                }
                "agent" => {
                    let mut ids = assoc.agent_ids;
                    push_if_absent(&mut ids, AgentId::new(entity_id), entity_id);
                    set_plugin_agents(&pool, &assoc.plugin.id, &ids).await
                }
                "mcp_server" => {
                    let mut ids = assoc.mcp_server_ids;
                    push_if_absent(&mut ids, McpServerId::new(entity_id), entity_id);
                    set_plugin_mcp_servers(&pool, &assoc.plugin.id, &ids).await
                }
                _ => return None,
            };

            return match result {
                Ok(()) => Some(plugin_id.to_string()),
                Err(e) => {
                    tracing::warn!(error = %e, plugin_id = %plugin_id, entity_kind = %entity_kind, "Failed to add entity to target plugin");
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
        tracing::warn!(entity_kind = %entity_kind, "No plugins found for user, cannot auto-add entity");
        return None;
    }

    let default_plugin = plugins.last()?;
    let assoc = find_plugin_with_associations(&pool, user_id, &default_plugin.plugin_id)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, plugin_id = %default_plugin.plugin_id, "Failed to fetch plugin associations for default plugin");
        })
        .ok()
        .flatten()?;

    let result = match entity_kind {
        "skill" => {
            let mut ids = assoc.skill_ids;
            push_if_absent(&mut ids, SkillId::new(entity_id), entity_id);
            set_plugin_skills(&pool, &default_plugin.id, &ids).await
        }
        "agent" => {
            let mut ids = assoc.agent_ids;
            push_if_absent(&mut ids, AgentId::new(entity_id), entity_id);
            set_plugin_agents(&pool, &default_plugin.id, &ids).await
        }
        "mcp_server" => {
            let mut ids = assoc.mcp_server_ids;
            push_if_absent(&mut ids, McpServerId::new(entity_id), entity_id);
            set_plugin_mcp_servers(&pool, &default_plugin.id, &ids).await
        }
        _ => return None,
    };

    match result {
        Ok(()) => Some(default_plugin.plugin_id.clone()),
        Err(e) => {
            tracing::warn!(error = %e, plugin_id = %default_plugin.plugin_id, entity_kind = %entity_kind, "Failed to auto-add entity to plugin");
            None
        }
    }
}
