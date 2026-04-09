use std::path::{Path, PathBuf};

use anyhow::Result;
use sqlx::PgPool;
use systemprompt::models::ProfileBootstrap;

use super::export::PluginBundle;
use super::github_sync::{elapsed_ms, SyncResult};
use super::github_sync_git::{
    build_authenticated_url, git_add_all, git_clone_shallow, git_commit, git_has_changes,
    git_head_hash, git_pull, git_push,
};

fn write_plugin_bundles_to_repo(bundles: &[PluginBundle], local_path: &Path) -> Result<u64> {
    let mut plugin_count = 0u64;

    for bundle in bundles {
        let plugin_dir = local_path.join(&bundle.id);

        if plugin_dir.exists() {
            std::fs::remove_dir_all(&plugin_dir)?;
        }
        std::fs::create_dir_all(&plugin_dir)?;

        for file in &bundle.files {
            let file_path = plugin_dir.join(&file.path);
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&file_path, &file.content)?;

            #[cfg(unix)]
            if file.executable {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o755))?;
            }
        }

        plugin_count += 1;
    }

    Ok(plugin_count)
}

pub async fn publish_marketplace_to_github(
    pool: &PgPool,
    marketplace_id: &str,
    repo_url: &str,
    triggered_by: &str,
) -> Result<SyncResult> {
    let start = std::time::Instant::now();

    tracing::info!(
        marketplace_id,
        repo_url,
        "Starting GitHub marketplace publish"
    );

    let services_path = ProfileBootstrap::get()
        .map(|p| PathBuf::from(&p.paths.services))
        .map_err(|e| anyhow::anyhow!("Failed to get profile: {e}"))?;

    let local_path = PathBuf::from("storage/github-marketplaces").join(marketplace_id);

    ensure_publish_repo(repo_url, &local_path)?;

    let export = super::export::generate_org_marketplace_export_bundles(
        &services_path,
        pool,
        marketplace_id,
        "linux",
    )
    .await?;

    let plugin_count = write_plugin_bundles_to_repo(&export.plugins, &local_path)?;
    write_marketplace_json(&local_path, &export.marketplace.content)?;

    let push_url = build_authenticated_url(repo_url);
    git_add_all(&local_path)?;

    if !git_has_changes(&local_path)? {
        let duration_ms = elapsed_ms(start);
        tracing::info!(marketplace_id, "No changes to publish");
        return Ok(SyncResult {
            commit_hash: git_head_hash(&local_path)?,
            plugins_synced: plugin_count,
            errors: 0,
            changed: false,
            duration_ms,
        });
    }

    git_commit(
        &local_path,
        &format!("Marketplace update from admin ({marketplace_id})"),
    )?;
    git_push(&local_path, &push_url)?;

    let current_hash = git_head_hash(&local_path)?;
    let duration_ms = elapsed_ms(start);

    let _ = std::fs::write(local_path.join(".last-commit"), &current_hash);

    log_publish_result(pool, marketplace_id, &current_hash, plugin_count, triggered_by, duration_ms).await;

    Ok(SyncResult {
        commit_hash: current_hash,
        plugins_synced: plugin_count,
        errors: 0,
        changed: true,
        duration_ms,
    })
}

fn ensure_publish_repo(repo_url: &str, local_path: &Path) -> Result<()> {
    if local_path.join(".git").exists() {
        git_pull(local_path)?;
    } else {
        std::fs::create_dir_all(local_path)?;
        let push_url = build_authenticated_url(repo_url);
        git_clone_shallow(&push_url, local_path)?;
    }
    Ok(())
}

fn write_marketplace_json(local_path: &Path, content: impl AsRef<[u8]>) -> Result<()> {
    let marketplace_json_path = local_path.join(".claude-plugin/marketplace.json");
    if let Some(parent) = marketplace_json_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&marketplace_json_path, content)?;
    Ok(())
}

async fn log_publish_result(
    pool: &PgPool,
    marketplace_id: &str,
    current_hash: &str,
    plugin_count: u64,
    triggered_by: &str,
    duration_ms: u64,
) {
    let _ = super::org_marketplaces::insert_sync_log(
        pool,
        &super::org_marketplaces::SyncLogEntry {
            marketplace_id,
            operation: "publish",
            status: "success",
            commit_hash: Some(current_hash),
            plugins_synced: i64::try_from(plugin_count).unwrap_or(i64::MAX),
            errors: 0,
            error_message: None,
            triggered_by,
            duration_ms: Some(i64::try_from(duration_ms).unwrap_or(i64::MAX)),
        },
    )
    .await;

    tracing::info!(
        marketplace_id,
        plugins = plugin_count,
        commit = &current_hash[..std::cmp::min(8, current_hash.len())],
        duration_ms,
        "GitHub marketplace publish completed"
    );
}
