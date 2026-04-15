mod bundle_files;
mod types;
mod user_bundles;

pub use types::*;

use sqlx::PgPool;
use std::path::Path;
use systemprompt::identifiers::UserId;
use systemprompt::models::Config;

use crate::types::PLUGIN_MANIFEST_PATH;

use crate::repositories::export_auth::load_plugin_configs_by_ids;
use crate::repositories::export_builders::{build_plugin_files, PluginBuildContext};
use crate::repositories::export_scripts::build_marketplace;
use crate::repositories::export_validation::{compute_bundle_counts, compute_export_totals};
use crate::repositories::user_agents::list_user_agents;
use crate::repositories::user_mcp_servers::list_user_mcp_servers;
use crate::repositories::user_plugins::list_user_plugins;
use crate::repositories::user_skills::list_user_skills;
use systemprompt_web_shared::error::MarketplaceError;
use user_bundles::UserBundleContext;

#[derive(Debug, Clone, Copy)]
pub struct ExportParams<'a> {
    pub services_path: &'a Path,
    pub pool: &'a PgPool,
    pub user_id: &'a UserId,
    pub username: &'a str,
    pub email: &'a str,
    pub roles: &'a [String],
}

pub async fn generate_export_bundles(
    params: &ExportParams<'_>,
) -> Result<SyncPluginsResponse, MarketplaceError> {
    let plugins_path = params.services_path.join("plugins");
    let skills_path = params.services_path.join("skills");
    let platform_url = Config::get().map_or_else(|_| String::new(), |c| c.api_external_url.clone());

    let master_key = crate::repositories::secret_crypto::load_master_key()
        .map_err(|e| {
            tracing::error!(error = %e, "ENCRYPTION_MASTER_KEY not configured — skill secrets will not be included in export");
            e
        })
        .ok();

    let org_plugin_ids = resolve_org_plugin_ids(params.pool, params.user_id).await;
    let all_plugin_configs = load_plugin_configs_by_ids(&plugins_path, &org_plugin_ids)?;

    let user_plugins = list_user_plugins(params.pool, params.user_id).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, user_id = %params.user_id, "Failed to list user plugins for export");
        Vec::new()
    });

    let plugin_configs = filter_forked_plugins(all_plugin_configs, &user_plugins);

    let plugin_tokens: std::collections::HashMap<String, String> = plugin_configs
        .iter()
        .filter_map(|(id, _)| {
            crate::repositories::plugin_jwt::generate_plugin_token(params.user_id, params.email, id)
                .map(|t| (id.clone(), t))
                .map_err(|e| tracing::warn!(error = %e, plugin_id = %id, "Failed to generate plugin JWT"))
                .ok()
        })
        .collect();

    let mut bundles = build_org_bundles(
        &plugin_configs,
        &plugins_path,
        &skills_path,
        params.services_path,
        &platform_url,
        &plugin_tokens,
    )?;
    generate_bundle_env_and_hook_files(&platform_url, &plugin_tokens, &mut bundles);

    if let Err(e) = build_user_bundles(
        params,
        &platform_url,
        master_key.as_ref(),
        &user_plugins,
        &mut bundles,
    )
    .await
    {
        tracing::warn!(error = %e, "Failed during user bundle generation");
    }

    let marketplace = build_marketplace(&plugin_configs, &bundles, params.username, params.email)?;
    let totals = compute_export_totals(&bundles);
    Ok(SyncPluginsResponse {
        plugins: bundles,
        marketplace,
        totals,
    })
}

async fn resolve_org_plugin_ids(
    pool: &PgPool,
    user_id: &UserId,
) -> std::collections::HashSet<String> {
    let mut ids = crate::repositories::org_marketplaces::resolve_authorized_org_plugin_ids(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to resolve authorized org plugin IDs");
            std::collections::HashSet::new()
        });
    let selected = crate::repositories::user_plugin_selections::list_selected_org_plugins(pool, user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list selected org plugins for user");
            Vec::new()
        });
    if !selected.is_empty() {
        let selected_set: std::collections::HashSet<String> = selected.into_iter().collect();
        ids.retain(|id| selected_set.contains(id));
    }
    ids.into_iter().collect()
}

fn filter_forked_plugins(
    all_configs: Vec<(String, crate::types::PlatformPluginConfig)>,
    user_plugins: &[crate::types::UserPlugin],
) -> Vec<(String, crate::types::PlatformPluginConfig)> {
    let forked_base_ids: std::collections::HashSet<String> = user_plugins
        .iter()
        .filter_map(|p| p.base_plugin_id.clone())
        .collect();
    all_configs
        .into_iter()
        .filter(|(id, _)| !forked_base_ids.contains(id.as_str()))
        .collect()
}

fn generate_bundle_env_and_hook_files(
    platform_url: &str,
    tokens: &std::collections::HashMap<String, String>,
    bundles: &mut [PluginBundle],
) {
    for bundle in bundles.iter_mut() {
        if let Some(token) = tokens.get(&bundle.id) {
            bundle.files.push(PluginFile {
                path: ".env.plugin".to_string(),
                content: format!(
                    "SYSTEMPROMPT_PLUGIN_TOKEN={token}\nSYSTEMPROMPT_API_URL={platform_url}\n"
                ),
                executable: false,
            });
            if let Err(e) =
                crate::repositories::export_builders::build_hook_files(platform_url, token, &mut bundle.files)
            {
                tracing::warn!(error = %e, plugin_id = %bundle.id, "Failed to generate hook files");
            }
        }
    }
}

fn build_org_bundles(
    plugin_configs: &[(String, crate::types::PlatformPluginConfig)],
    plugins_path: &Path,
    skills_path: &Path,
    services_path: &Path,
    platform_url: &str,
    tokens: &std::collections::HashMap<String, String>,
) -> Result<Vec<PluginBundle>, MarketplaceError> {
    let mut bundles = Vec::new();
    for (plugin_id, plugin) in plugin_configs {
        let ctx = PluginBuildContext {
            plugin_id,
            plugin,
            plugins_path,
            skills_path,
            services_path,
            platform_url,
            token: tokens.get(plugin_id.as_str()).map(String::as_str),
        };
        let files = build_plugin_files(&ctx)?;
        let counts = compute_bundle_counts(&files);
        let version = files
            .iter()
            .find(|f| f.path == PLUGIN_MANIFEST_PATH)
            .and_then(|f| {
                serde_json::from_str::<PluginManifest>(&f.content)
                    .map_err(|e| {
                        tracing::warn!(error = %e, "Failed to parse plugin.json for version");
                    })
                    .ok()
            })
            .map(|m| m.version)
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| plugin.base.version.clone());

        bundles.push(PluginBundle {
            id: plugin_id.clone(),
            name: plugin.base.name.clone(),
            description: plugin.base.description.clone(),
            version,
            counts,
            files,
        });
    }
    Ok(bundles)
}

async fn build_user_bundles(
    params: &ExportParams<'_>,
    platform_url: &str,
    master_key: Option<&[u8; 32]>,
    user_plugins: &[crate::types::UserPlugin],
    bundles: &mut Vec<PluginBundle>,
) -> Result<(), MarketplaceError> {
    let all_user_skills = list_user_skills(params.pool, params.user_id).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, user_id = %params.user_id, "Failed to list user skills for export");
        Vec::new()
    });
    let all_user_agents = list_user_agents(params.pool, params.user_id).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, user_id = %params.user_id, "Failed to list user agents for export");
        Vec::new()
    });
    let all_user_mcp_servers = list_user_mcp_servers(params.pool, params.user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, user_id = %params.user_id, "Failed to list user MCP servers for export");
            Vec::new()
        });

    let mut claimed_skill_ids = std::collections::HashSet::new();
    let mut claimed_agent_ids = std::collections::HashSet::new();
    let mut claimed_mcp_server_ids = std::collections::HashSet::new();

    let bundle_ctx = UserBundleContext {
        pool: params.pool,
        user_id: params.user_id,
        username: params.username,
        email: params.email,
        master_key,
        platform_url,
        all_user_skills: &all_user_skills,
        all_user_agents: &all_user_agents,
        all_user_mcp_servers: &all_user_mcp_servers,
    };

    for user_plugin in user_plugins {
        if let Some(user_bundle) = user_bundles::build_user_plugin_bundle(
            &bundle_ctx,
            user_plugin,
            &mut claimed_skill_ids,
            &mut claimed_agent_ids,
            &mut claimed_mcp_server_ids,
        )
        .await
        {
            bundles.push(user_bundle);
        }
    }

    Ok(())
}

pub async fn generate_org_marketplace_export_bundles(
    services_path: &Path,
    pool: &PgPool,
    marketplace_id: &str,
    _platform: &str,
) -> Result<SyncPluginsResponse, MarketplaceError> {
    let plugin_ids = crate::repositories::org_marketplaces::list_marketplace_plugin_ids(pool, marketplace_id)
        .await
        .map_err(|e| {
            MarketplaceError::Internal(format!("Failed to list marketplace plugins: {e}"))
        })?;

    let plugins_path = services_path.join("plugins");
    let skills_path = services_path.join("skills");
    let platform_url = Config::get().map_or_else(|_| String::new(), |c| c.api_external_url.clone());

    let all_configs =
        load_plugin_configs_by_ids(&plugins_path, &plugin_ids.iter().cloned().collect())?;
    let tokens = std::collections::HashMap::new();

    let bundles = build_org_bundles(
        &all_configs,
        &plugins_path,
        &skills_path,
        services_path,
        &platform_url,
        &tokens,
    )?;
    let totals = compute_export_totals(&bundles);
    let marketplace = build_marketplace(&all_configs, &bundles, marketplace_id, "")?;

    Ok(SyncPluginsResponse {
        plugins: bundles,
        marketplace,
        totals,
    })
}
