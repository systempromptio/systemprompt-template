use std::collections::HashSet;
use std::path::Path;

use chrono::Utc;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories::marketplace_grp::org_marketplaces::resolve_authorized_marketplace_groups;
use crate::repositories::plugins_grp::plugin_crud_ops::find_plugin_detail;
use crate::repositories::users_grp::user_plugins::{list_user_plugins_enriched, AssociatedEntity};
use crate::repositories::users_grp::user_plugins::UserPluginEnriched;
use crate::types::UserPlugin;

fn synthesize_org_plugin(
    user_id: &UserId,
    services_path: &Path,
    plugin_id: &str,
    source_marketplace_id: &str,
) -> Option<UserPluginEnriched> {
    let detail = find_plugin_detail(services_path, plugin_id).ok().flatten()?;

    let skills_path = services_path.join("skills");
    let skills: Vec<AssociatedEntity> = detail
        .skills
        .iter()
        .map(|sid| {
            let name = read_entity_name(&skills_path.join(sid.as_str()), sid.as_str());
            AssociatedEntity {
                id: sid.as_str().to_string(),
                name,
            }
        })
        .collect();

    let agents_path = services_path.join("agents");
    let agents: Vec<AssociatedEntity> = detail
        .agents
        .iter()
        .map(|aid| {
            let name = read_entity_name(&agents_path.join(aid.as_str()), aid.as_str());
            AssociatedEntity {
                id: aid.as_str().to_string(),
                name,
            }
        })
        .collect();

    let mcp_servers: Vec<AssociatedEntity> = detail
        .mcp_servers
        .iter()
        .map(|mid| AssociatedEntity {
            id: mid.as_str().to_string(),
            name: mid.as_str().to_string(),
        })
        .collect();

    let skill_count = skills.len();
    let agent_count = agents.len();
    let mcp_count = mcp_servers.len();

    Some(UserPluginEnriched {
        plugin: UserPlugin {
            id: format!("org:{source_marketplace_id}:{plugin_id}"),
            user_id: user_id.clone(),
            plugin_id: detail.id.clone(),
            name: detail.name,
            description: detail.description,
            version: detail.version,
            enabled: detail.enabled,
            category: detail.category,
            keywords: detail.keywords,
            author_name: detail.author_name,
            base_plugin_id: Some(source_marketplace_id.to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        skills,
        agents,
        mcp_servers,
        skill_count,
        agent_count,
        mcp_count,
    })
}

fn read_entity_name(entity_dir: &Path, fallback_id: &str) -> String {
    let config_path = entity_dir.join("config.yaml");
    if let Ok(text) = std::fs::read_to_string(&config_path) {
        if let Ok(val) = serde_yaml::from_str::<serde_yaml::Value>(&text) {
            if let Some(name) = val.get("name").and_then(|v| v.as_str()) {
                return name.to_string();
            }
        }
    }
    fallback_id.to_string()
}

/// Returns the union of a user's own plugins and org-authorized plugins.
///
/// Org plugins are synthesized from on-disk plugin metadata so they appear in
/// the user's workspace views without requiring per-user rows. Plugins already
/// present in `user_plugins` (by `plugin_id`) take precedence so user
/// customizations are preserved.
pub async fn list_effective_enriched_plugins(
    pool: &PgPool,
    user_id: &UserId,
    services_path: &Path,
) -> Vec<UserPluginEnriched> {
    let mut enriched = list_user_plugins_enriched(pool, user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list user plugins enriched");
            vec![]
        });

    let existing_plugin_ids: HashSet<String> = enriched
        .iter()
        .map(|ep| ep.plugin.plugin_id.clone())
        .collect();

    let org_groups = resolve_authorized_marketplace_groups(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to resolve org marketplace groups");
            vec![]
        });

    let mut seen_org: HashSet<String> = HashSet::new();
    for (mkt, plugin_ids) in org_groups {
        for plugin_id in plugin_ids {
            if existing_plugin_ids.contains(&plugin_id) || !seen_org.insert(plugin_id.clone()) {
                continue;
            }
            if let Some(ep) =
                synthesize_org_plugin(user_id, services_path, &plugin_id, mkt.id.as_str())
            {
                enriched.push(ep);
            }
        }
    }

    enriched
}

pub async fn count_org_entity_additions(
    pool: &PgPool,
    services_path: &Path,
) -> (i64, i64, i64, i64) {
    let org_groups = resolve_authorized_marketplace_groups(pool)
        .await
        .unwrap_or_default();

    let mut seen: HashSet<String> = HashSet::new();
    let mut plugins = 0i64;
    let mut skills = 0i64;
    let mut agents = 0i64;
    let mut mcp_servers = 0i64;

    for (_mkt, plugin_ids) in org_groups {
        for plugin_id in plugin_ids {
            if !seen.insert(plugin_id.clone()) {
                continue;
            }
            if let Ok(Some(detail)) = find_plugin_detail(services_path, &plugin_id) {
                plugins += 1;
                skills += i64::try_from(detail.skills.len()).unwrap_or(0);
                agents += i64::try_from(detail.agents.len()).unwrap_or(0);
                mcp_servers += i64::try_from(detail.mcp_servers.len()).unwrap_or(0);
            }
        }
    }

    (plugins, skills, agents, mcp_servers)
}
