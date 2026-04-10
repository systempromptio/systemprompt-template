mod local_sync;
mod types;

use std::path::{Path, PathBuf};

use anyhow::Result;
use sqlx::PgPool;
use systemprompt::models::ProfileBootstrap;

use types::PluginImportTally;
pub use types::SyncResult;

pub(crate) use super::github_sync_bundle::{build_bundle_from_directory, import_or_update_plugin};
pub(crate) use super::github_sync_git::{git_clone_shallow, git_head_hash, git_pull};

pub use super::github_sync_publish::publish_marketplace_to_github;
pub use local_sync::sync_marketplace_from_local;

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
        import_single_plugin(
            plugin_entry,
            base_path,
            services_path,
            log_context,
            &mut tally,
        );
    }

    tally
}

fn import_single_plugin(
    plugin_entry: &serde_json::Value,
    base_path: &Path,
    services_path: &Path,
    log_context: &str,
    tally: &mut PluginImportTally,
) {
    let Some(source) = extract_plugin_source(plugin_entry, log_context) else {
        tally.error_count += 1;
        return;
    };

    let source_path = source.strip_prefix("./").unwrap_or(source);
    let plugin_dir = base_path.join(source_path);

    if !plugin_dir.exists() {
        log_missing_plugin_dir(&plugin_dir, log_context, tally);
        return;
    }

    match build_and_import_plugin(&plugin_dir, services_path, source, log_context) {
        Ok(plugin_id) => {
            tally.plugin_ids.push(plugin_id);
            tally.success_count += 1;
        }
        Err(()) => {
            tally.error_count += 1;
        }
    }
}

fn extract_plugin_source<'a>(
    plugin_entry: &'a serde_json::Value,
    log_context: &str,
) -> Option<&'a str> {
    let source = plugin_entry.get("source").and_then(|v| v.as_str());
    if source.is_none() && log_context == "github" {
        tracing::warn!("Plugin entry missing 'source' field, skipping");
    }
    source
}

fn build_and_import_plugin(
    plugin_dir: &Path,
    services_path: &Path,
    source: &str,
    log_context: &str,
) -> Result<String, ()> {
    let bundle = build_bundle_from_directory(plugin_dir).map_err(|e| {
        tracing::warn!(source = %source, error = %e, "Failed to build bundle from directory");
    })?;

    let plugin_id = bundle.id.clone();
    import_or_update_plugin(services_path, &bundle).map_err(|e| {
        tracing::warn!(plugin_id = %plugin_id, error = %e, "Failed to import plugin");
    })?;

    tracing::info!(plugin_id = %plugin_id, "Plugin synced from {log_context}");
    Ok(plugin_id)
}

fn log_missing_plugin_dir(plugin_dir: &Path, log_context: &str, tally: &mut PluginImportTally) {
    if log_context == "github" {
        tracing::warn!(path = %plugin_dir.display(), "Plugin source directory not found");
        tally.error_count += 1;
    } else {
        tracing::debug!(path = %plugin_dir.display(), "Local plugin directory not found, skipping");
    }
}

async fn finalize_sync(
    pool: &PgPool,
    marketplace_id: &str,
    plugin_ids: &[String],
    error_count: &mut u64,
) {
    if !plugin_ids.is_empty() {
        if let Err(e) =
            super::org_marketplaces::set_marketplace_plugins(pool, marketplace_id, plugin_ids).await
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
    pool: &PgPool,
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

    ensure_repo_up_to_date(repo_url, &local_path)?;

    let current_hash = git_head_hash(&local_path)?;
    if let Some(result) = check_unchanged(marketplace_id, &local_path, &current_hash, start) {
        return Ok(result);
    }

    let plugins = read_marketplace_plugins(&local_path)?;
    let mut tally = import_plugins_from_entries(&plugins, &local_path, &services_path, "github");

    finalize_sync(
        pool,
        marketplace_id,
        &tally.plugin_ids,
        &mut tally.error_count,
    )
    .await;

    if let Err(e) = std::fs::write(local_path.join(".last-commit"), &current_hash) {
        tracing::warn!(error = %e, "Failed to save last commit marker");
    }

    let duration_ms = elapsed_ms(start);

    log_sync_result(
        pool,
        marketplace_id,
        &current_hash,
        &tally,
        triggered_by,
        duration_ms,
    )
    .await;

    Ok(SyncResult {
        commit_hash: current_hash,
        plugins_synced: tally.success_count,
        errors: tally.error_count,
        changed: true,
        duration_ms,
    })
}

fn ensure_repo_up_to_date(repo_url: &str, local_path: &Path) -> Result<()> {
    if local_path.join(".git").exists() {
        git_pull(local_path)?;
    } else {
        std::fs::create_dir_all(local_path)?;
        git_clone_shallow(repo_url, local_path)?;
    }
    Ok(())
}

fn check_unchanged(
    marketplace_id: &str,
    local_path: &Path,
    current_hash: &str,
    start: std::time::Instant,
) -> Option<SyncResult> {
    let marker_path = local_path.join(".last-commit");
    let last_hash = std::fs::read_to_string(&marker_path).unwrap_or_else(|e| {
        tracing::debug!(error = %e, path = %marker_path.display(), "No marker file found, treating as first run");
        String::new()
    });
    if current_hash.trim() != last_hash.trim() || last_hash.is_empty() {
        return None;
    }
    let duration_ms = elapsed_ms(start);
    tracing::info!(
        marketplace_id,
        commit = &current_hash[..8.min(current_hash.len())],
        "Marketplace unchanged"
    );
    Some(SyncResult {
        commit_hash: current_hash.to_string(),
        plugins_synced: 0,
        errors: 0,
        changed: false,
        duration_ms,
    })
}

fn read_marketplace_plugins(local_path: &Path) -> Result<Vec<serde_json::Value>> {
    let marketplace_json_path = local_path.join(".claude-plugin/marketplace.json");
    let marketplace_content = std::fs::read_to_string(&marketplace_json_path)
        .map_err(|e| anyhow::anyhow!("Failed to read marketplace.json: {e}"))?;
    let marketplace: serde_json::Value = serde_json::from_str(&marketplace_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse marketplace.json: {e}"))?;

    marketplace
        .get("plugins")
        .and_then(|v| v.as_array())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("marketplace.json missing 'plugins' array"))
}

async fn log_sync_result(
    pool: &PgPool,
    marketplace_id: &str,
    current_hash: &str,
    tally: &PluginImportTally,
    triggered_by: &str,
    duration_ms: u64,
) {
    let _ = super::org_marketplaces::insert_sync_log(
        pool,
        &super::org_marketplaces::SyncLogEntry {
            marketplace_id,
            operation: "sync",
            status: "success",
            commit_hash: Some(current_hash),
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
}

pub(crate) async fn mark_all_users_dirty(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE marketplace_sync_status SET dirty = true, last_changed_at = NOW()")
        .execute(pool)
        .await?;
    Ok(())
}
