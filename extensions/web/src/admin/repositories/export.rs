use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::models::Config;

use super::export_auth::load_plugin_configs_by_ids;
use super::export_builders::{build_plugin_files, PluginBuildContext};
use super::export_scripts::{build_marketplace, load_marketplace_identity};
use super::export_validation::{compute_bundle_counts, compute_export_totals};

pub use super::export_user::generate_export_bundles;

#[derive(Debug, Serialize)]
pub struct SyncPluginsResponse {
    pub plugins: Vec<PluginBundle>,
    pub marketplace: MarketplaceFile,
    pub totals: ExportTotals,
}

#[derive(Debug, Serialize)]
pub struct ExportTotals {
    pub plugins: usize,
    pub files: usize,
    pub skills: usize,
    pub agents: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginBundleCounts {
    pub skills: usize,
    pub agents: usize,
    pub hooks: usize,
    pub mcp_servers: usize,
    pub scripts: usize,
    pub total_files: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginBundle {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub files: Vec<PluginFile>,
    pub counts: PluginBundleCounts,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginFile {
    pub path: String,
    pub content: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub executable: bool,
}

#[derive(Debug, Serialize)]
pub struct MarketplaceFile {
    pub path: String,
    pub content: String,
}

pub async fn generate_org_marketplace_export_bundles(
    services_path: &Path,
    pool: &Arc<PgPool>,
    marketplace_id: &str,
    platform: &str,
) -> Result<SyncPluginsResponse, anyhow::Error> {
    let plugins_path = services_path.join("plugins");
    let skills_path = services_path.join("skills");

    let platform_url = Config::get().map_or_else(|_| String::new(), |c| c.api_external_url.clone());

    let plugin_ids: std::collections::HashSet<String> =
        super::org_marketplaces::list_marketplace_plugin_ids(pool, marketplace_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list marketplace plugins: {e}"))?
            .into_iter()
            .collect();

    if plugin_ids.is_empty() {
        anyhow::bail!("Marketplace '{marketplace_id}' has no plugins assigned");
    }

    let plugin_configs = load_plugin_configs_by_ids(&plugins_path, &plugin_ids)?;

    let mut bundles = Vec::new();
    for (plugin_id, plugin) in &plugin_configs {
        let env_vars = std::collections::HashMap::new();
        let ctx = PluginBuildContext {
            plugin_id,
            plugin,
            plugins_path: &plugins_path,
            skills_path: &skills_path,
            services_path,
            plugin_token: "",
            platform_url: &platform_url,
            platform,
            env_vars: &env_vars,
        };
        let files = build_plugin_files(&ctx)?;
        let counts = compute_bundle_counts(&files);
        let version = files
            .iter()
            .find(|f| f.path == ".claude-plugin/plugin.json")
            .and_then(|f| {
                serde_json::from_str::<serde_json::Value>(&f.content)
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

    let identity = load_marketplace_identity(services_path);
    let marketplace = build_marketplace(&plugin_configs, &bundles, &identity)?;
    let totals = compute_export_totals(&bundles);

    Ok(SyncPluginsResponse {
        plugins: bundles,
        marketplace,
        totals,
    })
}
