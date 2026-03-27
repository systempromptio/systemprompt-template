use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use sqlx::PgPool;
use systemprompt::models::ProfileBootstrap;

pub(crate) use super::github_sync_bundle::{
    build_bundle_from_directory, import_or_update_plugin,
};
pub(crate) use super::github_sync_git::{git_clone_shallow, git_head_hash, git_pull};

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub commit_hash: String,
    pub plugins_synced: u64,
    pub errors: u64,
    pub changed: bool,
    pub duration_ms: u64,
}

struct PluginImportTally {
    plugin_ids: Vec<String>,
    success_count: u64,
    error_count: u64,
}

fn import_plugins_from_entries(
    plugins: &[serde_json::Value],
    base_path: &Path,
    services_path: &Path,
    log_context: &str,
) -> PluginImportTally {
    let mut tally = PluginImportTally {
        plugin_ids: Vec::new(),
        success_count: 0,
        error_count: 0,
    };

    for plugin_entry in plugins {
        let Some(source) = plugin_entry.get("source").and_then(|v| v.as_str()) else {
            if log_context == "github" {
                tracing::warn!("Plugin entry missing 'source' field, skipping");
            }
            tally.error_count += 1;
            continue;
        };

        let source_path = source.strip_prefix("./").unwrap_or(source);
        let plugin_dir = base_path.join(source_path);

        if !plugin_dir.exists() {
            if log_context == "github" {
                tracing::warn!(path = %plugin_dir.display(), "Plugin source directory not found");
                tally.error_count += 1;
            } else {
                tracing::debug!(path = %plugin_dir.display(), "Local plugin directory not found, skipping");
            }
            continue;
        }

        match build_bundle_from_directory(&plugin_dir) {
            Ok(bundle) => {
                let plugin_id = bundle.id.clone();
                match import_or_update_plugin(services_path, &bundle) {
                    Ok(()) => {
                        tally.plugin_ids.push(plugin_id.clone());
                        tally.success_count += 1;
                        tracing::info!(plugin_id = %plugin_id, "Plugin synced from {log_context}");
                    }
                    Err(e) => {
                        tracing::warn!(plugin_id = %plugin_id, error = %e, "Failed to import plugin");
                        tally.error_count += 1;
                    }
                }
            }
            Err(e) => {
                tracing::warn!(source = %source, error = %e, "Failed to build bundle from directory");
                tally.error_count += 1;
            }
        }
    }

    tally
}

async fn finalize_sync(
    pool: &Arc<PgPool>,
    marketplace_id: &str,
    plugin_ids: &[String],
    error_count: &mut u64,
) {
    if !plugin_ids.is_empty() {
        if let Err(e) =
            super::org_marketplaces::set_marketplace_plugins(pool, marketplace_id, plugin_ids)
                .await
        {
            tracing::error!(error = %e, "Failed to update marketplace plugin associations");
            *error_count += 1;
        }

        if let Err(e) = mark_all_users_dirty(pool).await {
            tracing::warn!(error = %e, "Failed to mark users dirty after sync");
        }
    }
}

pub(super) fn elapsed_ms(start: std::time::Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

pub async fn sync_marketplace_from_github(
    pool: &Arc<PgPool>,
    marketplace_id: &str,
    repo_url: &str,
    triggered_by: &str,
) -> Result<SyncResult> {
    let start = std::time::Instant::now();

    tracing::info!(marketplace_id, repo_url, "Starting GitHub marketplace sync");

    let services_path = ProfileBootstrap::get()
        .map(|p| PathBuf::from(&p.paths.services))
        .map_err(|e| anyhow::anyhow!("Failed to get profile: {e}"))?;

    let local_path = PathBuf::from("storage/github-marketplaces").join(marketplace_id);
    let marker_path = local_path.join(".last-commit");

    if local_path.join(".git").exists() {
        git_pull(&local_path)?;
    } else {
        std::fs::create_dir_all(&local_path)?;
        git_clone_shallow(repo_url, &local_path)?;
    }

    let current_hash = git_head_hash(&local_path)?;
    let last_hash = std::fs::read_to_string(&marker_path).unwrap_or_else(|_| String::new());
    if current_hash.trim() == last_hash.trim() && !last_hash.is_empty() {
        let duration_ms = elapsed_ms(start);
        tracing::info!(
            marketplace_id,
            commit = &current_hash[..8.min(current_hash.len())],
            "Marketplace unchanged"
        );
        return Ok(SyncResult {
            commit_hash: current_hash,
            plugins_synced: 0,
            errors: 0,
            changed: false,
            duration_ms,
        });
    }

    let marketplace_json_path = local_path.join(".claude-plugin/marketplace.json");
    let marketplace_content = std::fs::read_to_string(&marketplace_json_path)
        .map_err(|e| anyhow::anyhow!("Failed to read marketplace.json: {e}"))?;
    let marketplace: serde_json::Value = serde_json::from_str(&marketplace_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse marketplace.json: {e}"))?;

    let plugins = marketplace
        .get("plugins")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("marketplace.json missing 'plugins' array"))?;

    let mut tally = import_plugins_from_entries(plugins, &local_path, &services_path, "github");

    finalize_sync(pool, marketplace_id, &tally.plugin_ids, &mut tally.error_count).await;

    if let Err(e) = std::fs::write(&marker_path, &current_hash) {
        tracing::warn!(error = %e, "Failed to save last commit marker");
    }

    let duration_ms = elapsed_ms(start);

    let _ = super::org_marketplaces::insert_sync_log(
        pool,
        &super::org_marketplaces::SyncLogEntry {
            marketplace_id,
            operation: "sync",
            status: "success",
            commit_hash: Some(&current_hash),
            plugins_synced: i64::try_from(tally.success_count).unwrap_or(i64::MAX),
            errors: i64::try_from(tally.error_count).unwrap_or(i64::MAX),
            error_message: None,
            triggered_by,
            duration_ms: Some(i64::try_from(duration_ms).unwrap_or(i64::MAX)),
        },
    )
    .await;

    tracing::info!(
        marketplace_id,
        plugins = tally.success_count,
        errors = tally.error_count,
        commit = &current_hash[..std::cmp::min(8, current_hash.len())],
        duration_ms,
        "GitHub marketplace sync completed"
    );

    Ok(SyncResult {
        commit_hash: current_hash,
        plugins_synced: tally.success_count,
        errors: tally.error_count,
        changed: true,
        duration_ms,
    })
}

pub use super::github_sync_publish::publish_marketplace_to_github;

pub async fn sync_marketplace_from_local(
    pool: &Arc<PgPool>,
    marketplace_id: &str,
) -> Result<SyncResult> {
    let start = std::time::Instant::now();

    let marketplace_json_path =
        PathBuf::from("storage/files/plugins/.claude-plugin/marketplace.json");
    if !marketplace_json_path.exists() {
        anyhow::bail!("Local marketplace.json not found");
    }

    let services_path = ProfileBootstrap::get()
        .map(|p| PathBuf::from(&p.paths.services))
        .map_err(|e| anyhow::anyhow!("Failed to get profile: {e}"))?;

    let content = std::fs::read_to_string(&marketplace_json_path)
        .map_err(|e| anyhow::anyhow!("Failed to read local marketplace.json: {e}"))?;
    let marketplace: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse local marketplace.json: {e}"))?;

    let plugins = marketplace
        .get("plugins")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Local marketplace.json missing 'plugins' array"))?;

    let base_path = PathBuf::from(".");
    let mut tally = import_plugins_from_entries(plugins, &base_path, &services_path, "local");

    finalize_sync(pool, marketplace_id, &tally.plugin_ids, &mut tally.error_count).await;

    let duration_ms = elapsed_ms(start);

    let _ = super::org_marketplaces::insert_sync_log(
        pool,
        &super::org_marketplaces::SyncLogEntry {
            marketplace_id,
            operation: "sync",
            status: "success",
            commit_hash: None,
            plugins_synced: i64::try_from(tally.success_count).unwrap_or(i64::MAX),
            errors: i64::try_from(tally.error_count).unwrap_or(i64::MAX),
            error_message: None,
            triggered_by: "local",
            duration_ms: Some(i64::try_from(duration_ms).unwrap_or(i64::MAX)),
        },
    )
    .await;

    tracing::info!(
        marketplace_id,
        plugins = tally.success_count,
        errors = tally.error_count,
        duration_ms,
        "Local marketplace sync completed"
    );

    Ok(SyncResult {
        commit_hash: String::new(),
        plugins_synced: tally.success_count,
        errors: tally.error_count,
        changed: true,
        duration_ms,
    })
}

pub(crate) async fn mark_all_users_dirty(pool: &Arc<PgPool>) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE marketplace_sync_status SET dirty = true, last_changed_at = NOW()")
        .execute(pool.as_ref())
        .await?;
    Ok(())
}
