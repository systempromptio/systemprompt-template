use systemprompt::identifiers::{AgentId, McpServerId, SkillId, UserId};
use systemprompt_web_extension::admin::repositories::{
    find_plugin_with_associations, set_plugin_agents, set_plugin_mcp_servers, set_plugin_skills,
    user_plugins,
};

/// Deduplicates and appends a new entity ID, then persists via the provided setter.
async fn add_entity_to_association<Id, MkId, F, Fut>(
    pool: &std::sync::Arc<sqlx::PgPool>,
    plugin_id: &str,
    mut existing_ids: Vec<Id>,
    new_entity_id: &str,
    make_id: MkId,
    setter: F,
) -> Result<(), sqlx::Error>
where
    Id: AsRef<str>,
    MkId: FnOnce(&str) -> Id,
    F: FnOnce(&std::sync::Arc<sqlx::PgPool>, &str, &[Id]) -> Fut,
    Fut: std::future::Future<Output = Result<(), sqlx::Error>>,
{
    if !existing_ids.iter().any(|id| id.as_ref() == new_entity_id) {
        existing_ids.push(make_id(new_entity_id));
    }
    setter(pool, plugin_id, &existing_ids).await
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
                    add_entity_to_association(
                        &pool,
                        &assoc.plugin.id,
                        assoc.skill_ids,
                        entity_id,
                        |id| SkillId::new(id),
                        |p, pid, ids| set_plugin_skills(p, pid, ids),
                    )
                    .await
                }
                "agent" => {
                    add_entity_to_association(
                        &pool,
                        &assoc.plugin.id,
                        assoc.agent_ids,
                        entity_id,
                        |id| AgentId::new(id),
                        |p, pid, ids| set_plugin_agents(p, pid, ids),
                    )
                    .await
                }
                "mcp_server" => {
                    add_entity_to_association(
                        &pool,
                        &assoc.plugin.id,
                        assoc.mcp_server_ids,
                        entity_id,
                        |id| McpServerId::new(id),
                        |p, pid, ids| set_plugin_mcp_servers(p, pid, ids),
                    )
                    .await
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
            add_entity_to_association(
                &pool,
                &default_plugin.id,
                assoc.skill_ids,
                entity_id,
                |id| SkillId::new(id),
                |p, pid, ids| set_plugin_skills(p, pid, ids),
            )
            .await
        }
        "agent" => {
            add_entity_to_association(
                &pool,
                &default_plugin.id,
                assoc.agent_ids,
                entity_id,
                |id| AgentId::new(id),
                |p, pid, ids| set_plugin_agents(p, pid, ids),
            )
            .await
        }
        "mcp_server" => {
            add_entity_to_association(
                &pool,
                &default_plugin.id,
                assoc.mcp_server_ids,
                entity_id,
                |id| McpServerId::new(id),
                |p, pid, ids| set_plugin_mcp_servers(p, pid, ids),
            )
            .await
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
