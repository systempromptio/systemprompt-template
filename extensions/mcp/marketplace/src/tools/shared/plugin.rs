use std::future::Future;

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

/// Adds an entity ID to a list (deduplicating) then persists via the provided setter.
async fn upsert_entity<Id, Fut>(
    current_ids: &[Id],
    new_id: Id,
    entity_id: &str,
    setter: impl FnOnce(&[Id]) -> Fut,
) -> Result<(), sqlx::Error>
where
    Id: AsRef<str> + Clone,
    Fut: Future<Output = Result<(), sqlx::Error>>,
{
    let mut ids = current_ids.to_vec();
    push_if_absent(&mut ids, new_id, entity_id);
    setter(&ids).await
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
            let plugin_row_id = &assoc.plugin.id;
            let result = match entity_kind {
                "skill" => {
                    Some(upsert_entity(&assoc.skill_ids, SkillId::new(entity_id), entity_id, |ids| {
                        set_plugin_skills(&pool, plugin_row_id, ids)
                    }).await)
                }
                "agent" => {
                    Some(upsert_entity(&assoc.agent_ids, AgentId::new(entity_id), entity_id, |ids| {
                        set_plugin_agents(&pool, plugin_row_id, ids)
                    }).await)
                }
                "mcp_server" => {
                    Some(upsert_entity(&assoc.mcp_server_ids, McpServerId::new(entity_id), entity_id, |ids| {
                        set_plugin_mcp_servers(&pool, plugin_row_id, ids)
                    }).await)
                }
                _ => None,
            };

            return match result {
                Some(Ok(())) => Some(plugin_id.to_string()),
                Some(Err(e)) => {
                    tracing::warn!(error = %e, plugin_id = %plugin_id, entity_kind = %entity_kind, "Failed to add entity to target plugin");
                    None
                }
                None => None,
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
    let default_plugin = find_default_plugin(&pool, user_id, entity_kind).await?;
    let assoc = find_plugin_with_associations(&pool, user_id, &default_plugin.plugin_id)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, plugin_id = %default_plugin.plugin_id, "Failed to fetch plugin associations for default plugin");
        })
        .ok()
        .flatten()?;

    let result = add_entity_to_plugin(&pool, &default_plugin.id, entity_id, entity_kind, &assoc).await;
    match result {
        Some(Ok(())) => Some(default_plugin.plugin_id.clone()),
        Some(Err(e)) => {
            tracing::warn!(error = %e, plugin_id = %default_plugin.plugin_id, entity_kind = %entity_kind, "Failed to auto-add entity to plugin");
            None
        }
        None => None,
    }
}

async fn find_default_plugin(
    pool: &sqlx::PgPool,
    user_id: &UserId,
    entity_kind: &str,
) -> Option<systemprompt_web_extension::admin::types::UserPlugin> {
    let plugins = match user_plugins::list_user_plugins(pool, user_id).await {
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
    plugins.into_iter().last()
}

async fn add_entity_to_plugin(
    pool: &sqlx::PgPool,
    plugin_row_id: &str,
    entity_id: &str,
    entity_kind: &str,
    assoc: &systemprompt_web_extension::admin::types::UserPluginWithAssociations,
) -> Option<Result<(), sqlx::Error>> {
    match entity_kind {
        "skill" => {
            Some(upsert_entity(&assoc.skill_ids, SkillId::new(entity_id), entity_id, |ids| {
                set_plugin_skills(pool, plugin_row_id, ids)
            }).await)
        }
        "agent" => {
            Some(upsert_entity(&assoc.agent_ids, AgentId::new(entity_id), entity_id, |ids| {
                set_plugin_agents(pool, plugin_row_id, ids)
            }).await)
        }
        "mcp_server" => {
            Some(upsert_entity(&assoc.mcp_server_ids, McpServerId::new(entity_id), entity_id, |ids| {
                set_plugin_mcp_servers(pool, plugin_row_id, ids)
            }).await)
        }
        _ => None,
    }
}
