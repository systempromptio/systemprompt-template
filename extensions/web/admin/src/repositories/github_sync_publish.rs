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

    let (plugin_count, push_url) =
        prepare_publish_content(pool, marketplace_id, &services_path, &local_path, repo_url)
            .await?;

    let publish_ctx = PublishContext {
        pool,
        marketplace_id,
        local_path: &local_path,
        push_url: &push_url,
        plugin_count,
        triggered_by,
        start,
    };
    commit_and_push_if_changed(&publish_ctx).await
}

struct PublishContext<'a> {
    pool: &'a PgPool,
    marketplace_id: &'a str,
    local_path: &'a Path,
    push_url: &'a str,
    plugin_count: u64,
    triggered_by: &'a str,
    start: std::time::Instant,
}

async fn prepare_publish_content(
    pool: &PgPool,
    marketplace_id: &str,
    services_path: &Path,
    local_path: &Path,
    repo_url: &str,
) -> Result<(u64, String)> {
    let export = super::export::generate_org_marketplace_export_bundles(
        services_path,
        pool,
        marketplace_id,
        "linux",
    )
    .await?;

    let plugin_count = write_plugin_bundles_to_repo(&export.plugins, local_path)?;
    write_marketplace_json(local_path, &export.marketplace.content)?;

    let push_url = build_authenticated_url(repo_url);
    git_add_all(local_path)?;

    Ok((plugin_count, push_url))
}

async fn commit_and_push_if_changed(ctx: &PublishContext<'_>) -> Result<SyncResult> {
    if !git_has_changes(ctx.local_path)? {
        let duration_ms = elapsed_ms(ctx.start);
        tracing::info!(marketplace_id = ctx.marketplace_id, "No changes to publish");
        return Ok(SyncResult {
            commit_hash: git_head_hash(ctx.local_path)?,
            plugins_synced: ctx.plugin_count,
            errors: 0,
            changed: false,
            duration_ms,
        });
    }

    git_commit(
        ctx.local_path,
        &format!("Marketplace update from admin ({})", ctx.marketplace_id),
    )?;
    git_push(ctx.local_path, ctx.push_url)?;

    let current_hash = git_head_hash(ctx.local_path)?;
    let duration_ms = elapsed_ms(ctx.start);

    let _ = std::fs::write(ctx.local_path.join(".last-commit"), &current_hash);

    log_publish_result(&PublishLogInput {
        pool: ctx.pool,
        marketplace_id: ctx.marketplace_id,
        current_hash: &current_hash,
        plugin_count: ctx.plugin_count,
        triggered_by: ctx.triggered_by,
        duration_ms,
    })
    .await;

    Ok(SyncResult {
        commit_hash: current_hash,
        plugins_synced: ctx.plugin_count,
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

struct PublishLogInput<'a> {
    pool: &'a PgPool,
    marketplace_id: &'a str,
    current_hash: &'a str,
    plugin_count: u64,
    triggered_by: &'a str,
    duration_ms: u64,
}

async fn log_publish_result(input: &PublishLogInput<'_>) {
    let _ = super::org_marketplaces::insert_sync_log(
        input.pool,
        &super::org_marketplaces::SyncLogEntry {
            marketplace_id: input.marketplace_id,
            operation: "publish",
            status: "success",
            commit_hash: Some(input.current_hash),
            plugins_synced: i64::try_from(input.plugin_count).unwrap_or(i64::MAX),
            errors: 0,
            error_message: None,
            triggered_by: input.triggered_by,
            duration_ms: Some(i64::try_from(input.duration_ms).unwrap_or(i64::MAX)),
        },
    )
    .await;

    tracing::info!(
        marketplace_id = input.marketplace_id,
        plugins = input.plugin_count,
        commit = &input.current_hash[..std::cmp::min(8, input.current_hash.len())],
        duration_ms = input.duration_ms,
        "GitHub marketplace publish completed"
    );
}
