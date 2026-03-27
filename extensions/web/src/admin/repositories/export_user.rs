use super::super::types::{UserAgent, UserSkill};
use super::export::{PluginBundle, PluginFile, SyncPluginsResponse};
use super::export_auth::generate_plugin_token;
use super::export_builders::PluginBuildContext;
use super::export_scripts::{build_marketplace, load_marketplace_identity};
use super::export_validation::{compute_bundle_counts, compute_export_totals};
use super::plugin_env::get_raw_env_vars_for_export;
use super::user_agents::list_user_agents;
use super::user_mcp_servers::list_user_mcp_servers;
use super::user_plugin_detail::get_plugin_with_associations;
use super::user_plugins::list_user_plugins;
use super::user_skills::list_user_skills;
use sqlx::PgPool;
use std::path::Path;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt::models::Config;

#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
pub async fn generate_export_bundles(
    services_path: &Path,
    pool: &Arc<PgPool>,
    user_id: &str,
    username: &str,
    email: &str,
    roles: &[String],
    department: &str,
    platform: &str,
) -> Result<SyncPluginsResponse, anyhow::Error> {
    let plugins_path = services_path.join("plugins");
    let skills_path = services_path.join("skills");

    let plugin_token = generate_plugin_token(user_id, username, email)?;
    let platform_url = Config::get().map_or_else(|_| String::new(), |c| c.api_external_url.clone());
    let uid = UserId::new(user_id);

    let is_admin = roles.iter().any(|r| r == "admin");
    let org_plugin_ids = super::org_marketplaces::resolve_authorized_org_plugin_ids(
        pool, roles, department, is_admin,
    )
    .await
    .unwrap_or_else(|_| std::collections::HashSet::new());

    let plugin_configs =
        super::export_auth::load_plugin_configs_by_ids(&plugins_path, &org_plugin_ids)?;

    let mut bundles = Vec::new();
    for (plugin_id, plugin) in &plugin_configs {
        let env_vars = match get_raw_env_vars_for_export(pool, &uid, plugin_id).await {
            Ok(vars) => vars,
            Err(e) => {
                tracing::warn!(error = %e, plugin_id = %plugin_id, "Failed to load env vars for export");
                std::collections::HashMap::new()
            }
        };
        let ctx = PluginBuildContext {
            plugin_id,
            plugin,
            plugins_path: &plugins_path,
            skills_path: &skills_path,
            services_path,
            plugin_token: &plugin_token,
            platform_url: &platform_url,
            platform,
            env_vars: &env_vars,
        };
        let files = super::export_builders::build_plugin_files(&ctx)?;
        let counts = compute_bundle_counts(&files);
        let version = files
            .iter()
            .find(|f| f.path == ".claude-plugin/plugin.json")
            .and_then(|f| {
                serde_json::from_str::<serde_json::Value>(&f.content)
                    .map_err(|e| {
                        tracing::warn!(error = %e, "Failed to parse plugin.json for version");
                    })
                    .ok()
            })
            .and_then(|v| v.get("version").and_then(|v| v.as_str()).map(String::from))
            .unwrap_or_else(|| plugin.base.version.clone());

        bundles.push(PluginBundle {
            id: plugin_id.to_string(),
            name: plugin.base.name.clone(),
            description: plugin.base.description.clone(),
            version,
            counts,
            files,
        });
    }

    let user_plugins = list_user_plugins(pool, &uid)
        .await
        .unwrap_or_else(|_| Vec::new());
    let all_user_skills = list_user_skills(pool, &uid)
        .await
        .unwrap_or_else(|_| Vec::new());
    let all_user_agents = list_user_agents(pool, &uid)
        .await
        .unwrap_or_else(|_| Vec::new());
    let all_user_mcp_servers = list_user_mcp_servers(pool, &uid)
        .await
        .unwrap_or_else(|_| Vec::new());
    let mut claimed_skill_ids = std::collections::HashSet::new();
    let mut claimed_agent_ids = std::collections::HashSet::new();
    let mut claimed_mcp_server_ids = std::collections::HashSet::new();
    for user_plugin in &user_plugins {
        if !user_plugin.enabled {
            continue;
        }
        let Ok(Some(assoc)) =
            get_plugin_with_associations(pool, &uid, &user_plugin.plugin_id).await
        else {
            continue;
        };

        let mut files = Vec::new();

        for skill_id in &assoc.skill_ids {
            claimed_skill_ids.insert(skill_id.as_str().to_owned());
            if let Some(skill) = all_user_skills.iter().find(|s| s.id == skill_id.as_str()) {
                let kebab_name = skill.skill_id.replace('_', "-");
                let skill_md = format!(
                    "---\nname: {}\ndescription: \"{}\"\n---\n\n{}\n",
                    kebab_name,
                    skill.description.replace('"', "\\\""),
                    skill.content.trim()
                );
                files.push(PluginFile {
                    path: format!("skills/{}/SKILL.md", skill.skill_id),
                    content: skill_md,
                    executable: false,
                });
            }
        }

        for agent_id in &assoc.agent_ids {
            claimed_agent_ids.insert(agent_id.as_str().to_owned());
            if let Some(agent) = all_user_agents.iter().find(|a| a.id == agent_id.as_str()) {
                let escaped_desc = agent.description.replace('"', "\\\"");
                let agent_md = format!(
                    "---\nname: {}\ndescription: \"{escaped_desc}\"\n---\n\n{}\n",
                    agent.agent_id,
                    agent.system_prompt.trim()
                );
                files.push(PluginFile {
                    path: format!("agents/{}.md", agent.agent_id),
                    content: agent_md,
                    executable: false,
                });
            }
        }

        let mut mcp_server_entries = serde_json::Map::new();
        for mcp_id in &assoc.mcp_server_ids {
            claimed_mcp_server_ids.insert(mcp_id.as_str().to_owned());
            if let Some(mcp) = all_user_mcp_servers.iter().find(|m| m.id == mcp_id.as_str()) {
                let url = if mcp.endpoint.is_empty() {
                    format!("{platform_url}/api/v1/mcp/{}/mcp", mcp.mcp_server_id)
                } else {
                    mcp.endpoint.clone()
                };
                let mut server = serde_json::Map::new();
                server.insert(
                    "type".to_string(),
                    serde_json::Value::String("http".to_string()),
                );
                server.insert("url".to_string(), serde_json::Value::String(url));
                mcp_server_entries
                    .insert(mcp.mcp_server_id.clone(), serde_json::Value::Object(server));
            }
        }
        if !mcp_server_entries.is_empty() {
            let mcp_json = serde_json::json!({ "mcpServers": mcp_server_entries });
            files.push(PluginFile {
                path: ".mcp.json".to_string(),
                content: serde_json::to_string_pretty(&mcp_json)?,
                executable: false,
            });
        }

        let mut manifest = serde_json::json!({
            "name": user_plugin.plugin_id,
            "description": user_plugin.description,
            "version": user_plugin.version
        });
        if let Some(ref base_id) = user_plugin.base_plugin_id {
            manifest["basePluginId"] = serde_json::json!(base_id);
        }
        files.push(PluginFile {
            path: ".claude-plugin/plugin.json".to_string(),
            content: serde_json::to_string_pretty(&manifest)?,
            executable: false,
        });

        let counts = compute_bundle_counts(&files);
        bundles.push(PluginBundle {
            id: user_plugin.plugin_id.clone(),
            name: user_plugin.name.clone(),
            description: user_plugin.description.clone(),
            version: user_plugin.version.clone(),
            counts,
            files,
        });
    }

    build_orphan_bundle(
        &all_user_skills,
        &all_user_agents,
        &claimed_skill_ids,
        &claimed_agent_ids,
        &mut bundles,
    )?;

    let identity = load_marketplace_identity(services_path);
    let marketplace = build_marketplace(&plugin_configs, &bundles, &identity)?;
    let totals = compute_export_totals(&bundles);

    Ok(SyncPluginsResponse {
        plugins: bundles,
        marketplace,
        totals,
    })
}

fn build_orphan_bundle(
    all_user_skills: &[UserSkill],
    all_user_agents: &[UserAgent],
    claimed_skill_ids: &std::collections::HashSet<String>,
    claimed_agent_ids: &std::collections::HashSet<String>,
    bundles: &mut Vec<PluginBundle>,
) -> Result<(), anyhow::Error> {
    let orphan_skills: Vec<_> = all_user_skills
        .iter()
        .filter(|s| !claimed_skill_ids.contains(&s.id))
        .collect();
    let orphan_agents: Vec<_> = all_user_agents
        .iter()
        .filter(|a| !claimed_agent_ids.contains(&a.id))
        .collect();

    if orphan_skills.is_empty() && orphan_agents.is_empty() {
        return Ok(());
    }

    let mut files = Vec::new();
    for skill in &orphan_skills {
        let kebab_name = skill.skill_id.replace('_', "-");
        let skill_md = format!(
            "---\nname: {}\ndescription: \"{}\"\n---\n\n{}\n",
            kebab_name,
            skill.description.replace('"', "\\\""),
            skill.content.trim()
        );
        files.push(PluginFile {
            path: format!("skills/{}/SKILL.md", skill.skill_id),
            content: skill_md,
            executable: false,
        });
    }
    for agent in &orphan_agents {
        let escaped_desc = agent.description.replace('"', "\\\"");
        let agent_md = format!(
            "---\nname: {}\ndescription: \"{escaped_desc}\"\n---\n\n{}\n",
            agent.agent_id,
            agent.system_prompt.trim()
        );
        files.push(PluginFile {
            path: format!("agents/{}.md", agent.agent_id),
            content: agent_md,
            executable: false,
        });
    }
    let manifest = serde_json::json!({
        "name": "custom",
        "description": "Your personal custom skills and agents",
        "version": "1.0.0"
    });
    files.push(PluginFile {
        path: ".claude-plugin/plugin.json".to_string(),
        content: serde_json::to_string_pretty(&manifest)?,
        executable: false,
    });
    let counts = super::export_validation::compute_bundle_counts(&files);
    bundles.push(PluginBundle {
        id: "custom".to_string(),
        name: "Custom".to_string(),
        description: "Your personal custom skills and agents".to_string(),
        version: "1.0.0".to_string(),
        counts,
        files,
    });
    Ok(())
}
