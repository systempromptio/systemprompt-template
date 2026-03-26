use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use sqlx::PgPool;
use systemprompt::models::ProfileBootstrap;

use super::export::{PluginBundle, PluginBundleCounts, PluginFile};

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub commit_hash: String,
    pub plugins_synced: u64,
    pub errors: u64,
    pub changed: bool,
    pub duration_ms: u64,
}

/// Sync a marketplace from its GitHub repository.
/// Clones/pulls the repo, parses marketplace.json, imports plugins, and updates associations.
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

    // Clone or pull
    if local_path.join(".git").exists() {
        git_pull(&local_path)?;
    } else {
        std::fs::create_dir_all(&local_path)?;
        git_clone_shallow(repo_url, &local_path)?;
    }

    // Check if content changed
    let current_hash = git_head_hash(&local_path)?;
    let last_hash = std::fs::read_to_string(&marker_path).unwrap_or_default();
    if current_hash.trim() == last_hash.trim() && !last_hash.is_empty() {
        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
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

    // Parse marketplace.json
    let marketplace_json_path = local_path.join(".claude-plugin/marketplace.json");
    let marketplace_content = std::fs::read_to_string(&marketplace_json_path)
        .map_err(|e| anyhow::anyhow!("Failed to read marketplace.json: {e}"))?;
    let marketplace: serde_json::Value = serde_json::from_str(&marketplace_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse marketplace.json: {e}"))?;

    let plugins = marketplace
        .get("plugins")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("marketplace.json missing 'plugins' array"))?;

    let mut plugin_ids = Vec::new();
    let mut success_count = 0u64;
    let mut error_count = 0u64;

    for plugin_entry in plugins {
        let Some(source) = plugin_entry.get("source").and_then(|v| v.as_str()) else {
            tracing::warn!("Plugin entry missing 'source' field, skipping");
            error_count += 1;
            continue;
        };

        let source_path = source.strip_prefix("./").unwrap_or(source);
        let plugin_dir = local_path.join(source_path);

        if !plugin_dir.exists() {
            tracing::warn!(path = %plugin_dir.display(), "Plugin source directory not found");
            error_count += 1;
            continue;
        }

        match build_bundle_from_directory(&plugin_dir) {
            Ok(bundle) => {
                let plugin_id = bundle.id.clone();
                match import_or_update_plugin(&services_path, &bundle) {
                    Ok(()) => {
                        plugin_ids.push(plugin_id.clone());
                        success_count += 1;
                        tracing::info!(plugin_id = %plugin_id, "Plugin synced from GitHub");
                    }
                    Err(e) => {
                        tracing::warn!(plugin_id = %plugin_id, error = %e, "Failed to import plugin");
                        error_count += 1;
                    }
                }
            }
            Err(e) => {
                tracing::warn!(source = %source, error = %e, "Failed to build bundle from directory");
                error_count += 1;
            }
        }
    }

    // Update org marketplace associations
    if !plugin_ids.is_empty() {
        if let Err(e) =
            super::org_marketplaces::set_marketplace_plugins(pool, marketplace_id, &plugin_ids)
                .await
        {
            tracing::error!(error = %e, "Failed to update marketplace plugin associations");
            error_count += 1;
        }

        // Mark all users dirty so their marketplace repos regenerate
        if let Err(e) = mark_all_users_dirty(pool).await {
            tracing::warn!(error = %e, "Failed to mark users dirty after sync");
        }
    }

    // Save commit hash
    if let Err(e) = std::fs::write(&marker_path, &current_hash) {
        tracing::warn!(error = %e, "Failed to save last commit marker");
    }

    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

    // Log the sync operation
    let _ = super::org_marketplaces::insert_sync_log(
        pool,
        marketplace_id,
        "sync",
        "success",
        Some(&current_hash),
        i64::try_from(success_count).unwrap_or(i64::MAX),
        i64::try_from(error_count).unwrap_or(i64::MAX),
        None,
        triggered_by,
        Some(i64::try_from(duration_ms).unwrap_or(i64::MAX)),
    )
    .await;

    tracing::info!(
        marketplace_id,
        plugins = success_count,
        errors = error_count,
        commit = &current_hash[..std::cmp::min(8, current_hash.len())],
        duration_ms,
        "GitHub marketplace sync completed"
    );

    Ok(SyncResult {
        commit_hash: current_hash,
        plugins_synced: success_count,
        errors: error_count,
        changed: true,
        duration_ms,
    })
}

/// Publish a marketplace to its GitHub repository.
/// Exports plugin bundles, writes them to the repo directory, commits, and pushes.
pub async fn publish_marketplace_to_github(
    pool: &Arc<PgPool>,
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

    // Ensure repo is cloned
    if local_path.join(".git").exists() {
        git_pull(&local_path)?;
    } else {
        std::fs::create_dir_all(&local_path)?;
        let push_url = build_authenticated_url(repo_url)?;
        git_clone_shallow(&push_url, &local_path)?;
    }

    // Generate export bundles
    let export = super::export::generate_org_marketplace_export_bundles(
        &services_path,
        pool,
        marketplace_id,
        "linux",
    )
    .await?;

    let mut plugin_count = 0u64;

    // Write each plugin bundle to the repo
    for bundle in &export.plugins {
        let plugin_dir = local_path.join(&bundle.id);

        // Clean existing plugin directory
        if plugin_dir.exists() {
            std::fs::remove_dir_all(&plugin_dir)?;
        }
        std::fs::create_dir_all(&plugin_dir)?;

        // Write all files
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

    // Write marketplace.json
    let marketplace_json_path = local_path.join(".claude-plugin/marketplace.json");
    if let Some(parent) = marketplace_json_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&marketplace_json_path, &export.marketplace.content)?;

    // Commit and push
    let push_url = build_authenticated_url(repo_url)?;
    git_add_all(&local_path)?;

    let has_changes = git_has_changes(&local_path)?;
    if !has_changes {
        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
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
        &format!("Marketplace update from admin ({})", marketplace_id),
    )?;
    git_push(&local_path, &push_url)?;

    let current_hash = git_head_hash(&local_path)?;
    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

    // Save commit hash marker
    let marker_path = local_path.join(".last-commit");
    let _ = std::fs::write(&marker_path, &current_hash);

    // Log the publish operation
    let _ = super::org_marketplaces::insert_sync_log(
        pool,
        marketplace_id,
        "publish",
        "success",
        Some(&current_hash),
        i64::try_from(plugin_count).unwrap_or(i64::MAX),
        0i64,
        None,
        triggered_by,
        Some(i64::try_from(duration_ms).unwrap_or(i64::MAX)),
    )
    .await;

    tracing::info!(
        marketplace_id,
        plugins = plugin_count,
        commit = &current_hash[..std::cmp::min(8, current_hash.len())],
        duration_ms,
        "GitHub marketplace publish completed"
    );

    Ok(SyncResult {
        commit_hash: current_hash,
        plugins_synced: plugin_count,
        errors: 0,
        changed: true,
        duration_ms,
    })
}

// --- Git helpers ---

pub(crate) fn git_clone_shallow(url: &str, target: &Path) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["clone", "--depth", "1", url, "."])
        .current_dir(target)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git clone failed: {stderr}");
    }
    Ok(())
}

pub(crate) fn git_pull(repo_path: &Path) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["pull", "--ff-only"])
        .current_dir(repo_path)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git pull failed: {stderr}");
    }
    Ok(())
}

pub(crate) fn git_head_hash(repo_path: &Path) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git rev-parse HEAD failed: {stderr}");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub(crate) fn build_bundle_from_directory(plugin_dir: &Path) -> Result<PluginBundle> {
    let plugin_json_path = plugin_dir.join(".claude-plugin/plugin.json");
    let manifest_content = std::fs::read_to_string(&plugin_json_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to read plugin.json at {}: {e}",
            plugin_json_path.display()
        )
    })?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest_content)?;

    let plugin_id = manifest
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("plugin.json missing 'name'"))?
        .to_string();

    let description = manifest
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let version = manifest
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("1.0.0")
        .to_string();

    let name = plugin_id.clone();

    let mut files = Vec::new();

    // plugin.json
    files.push(PluginFile {
        path: ".claude-plugin/plugin.json".to_string(),
        content: manifest_content,
        executable: false,
    });

    // hooks/hooks.json
    let hooks_path = plugin_dir.join("hooks/hooks.json");
    if hooks_path.exists() {
        let content = std::fs::read_to_string(&hooks_path)?;
        files.push(PluginFile {
            path: "hooks/hooks.json".to_string(),
            content,
            executable: false,
        });
    }

    // skills/*/SKILL.md
    let skills_dir = plugin_dir.join("skills");
    if skills_dir.exists() {
        collect_directory_files(&skills_dir, "skills", &mut files)?;
    }

    // agents/*.md
    let agents_dir = plugin_dir.join("agents");
    if agents_dir.exists() {
        collect_directory_files(&agents_dir, "agents", &mut files)?;
    }

    // .mcp.json
    let mcp_path = plugin_dir.join(".mcp.json");
    if mcp_path.exists() {
        let content = std::fs::read_to_string(&mcp_path)?;
        files.push(PluginFile {
            path: ".mcp.json".to_string(),
            content,
            executable: false,
        });
    }

    let mut skills_count = 0;
    let mut agents_count = 0;
    let mut hooks_count = 0;
    for f in &files {
        if f.path.starts_with("skills/") && f.path.ends_with("SKILL.md") {
            skills_count += 1;
        } else if f.path.starts_with("agents/") && f.path.ends_with(".md") {
            agents_count += 1;
        } else if f.path == "hooks/hooks.json" {
            hooks_count += 1;
        }
    }
    let total_files = files.len();

    Ok(PluginBundle {
        id: plugin_id,
        name,
        description,
        version,
        files,
        counts: PluginBundleCounts {
            skills: skills_count,
            agents: agents_count,
            mcp_servers: 0,
            scripts: 0,
            total_files,
        },
    })
}

pub(crate) fn collect_directory_files(
    dir: &Path,
    prefix: &str,
    files: &mut Vec<PluginFile>,
) -> Result<()> {
    for entry in walkdir::WalkDir::new(dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let rel_path = entry
            .path()
            .strip_prefix(dir)
            .map_err(|e| anyhow::anyhow!("Failed to strip prefix: {e}"))?;
        let path = format!("{prefix}/{}", rel_path.display());
        let content = std::fs::read_to_string(entry.path())
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {e}", entry.path().display()))?;
        files.push(PluginFile {
            path,
            content,
            executable: false,
        });
    }
    Ok(())
}

pub(crate) fn import_or_update_plugin(services_path: &Path, bundle: &PluginBundle) -> Result<()> {
    let plugin_dir = services_path.join("plugins").join(&bundle.id);

    if plugin_dir.exists() {
        std::fs::remove_dir_all(&plugin_dir)?;
    }

    super::import_plugin_bundle(services_path, bundle)?;
    Ok(())
}

/// Sync a marketplace from locally-available plugins.
/// Reads `storage/files/plugins/.claude-plugin/marketplace.json`, imports each plugin
/// into `services/plugins/`, and writes the associations to the database.
/// Always runs on every startup to ensure the marketplace is never empty.
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

    let mut plugin_ids = Vec::new();
    let mut success_count = 0u64;
    let mut error_count = 0u64;

    for plugin_entry in plugins {
        let Some(source) = plugin_entry.get("source").and_then(|v| v.as_str()) else {
            error_count += 1;
            continue;
        };

        let source_path = source.strip_prefix("./").unwrap_or(source);
        let plugin_dir = PathBuf::from(source_path);

        if !plugin_dir.exists() {
            tracing::debug!(
                path = %plugin_dir.display(),
                "Local plugin directory not found, skipping"
            );
            continue;
        }

        match build_bundle_from_directory(&plugin_dir) {
            Ok(bundle) => {
                let plugin_id = bundle.id.clone();
                match import_or_update_plugin(&services_path, &bundle) {
                    Ok(()) => {
                        plugin_ids.push(plugin_id.clone());
                        success_count += 1;
                        tracing::info!(plugin_id = %plugin_id, "Plugin synced from local");
                    }
                    Err(e) => {
                        tracing::warn!(
                            plugin_id = %plugin_id,
                            error = %e,
                            "Failed to import local plugin"
                        );
                        error_count += 1;
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    source = %source,
                    error = %e,
                    "Failed to build bundle from local directory"
                );
                error_count += 1;
            }
        }
    }

    // Update org marketplace associations
    if !plugin_ids.is_empty() {
        if let Err(e) =
            super::org_marketplaces::set_marketplace_plugins(pool, marketplace_id, &plugin_ids)
                .await
        {
            tracing::error!(error = %e, "Failed to update marketplace plugin associations");
            error_count += 1;
        }

        if let Err(e) = mark_all_users_dirty(pool).await {
            tracing::warn!(error = %e, "Failed to mark users dirty after local sync");
        }
    }

    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

    let _ = super::org_marketplaces::insert_sync_log(
        pool,
        marketplace_id,
        "sync",
        "success",
        None,
        i64::try_from(success_count).unwrap_or(i64::MAX),
        i64::try_from(error_count).unwrap_or(i64::MAX),
        None,
        "local",
        Some(i64::try_from(duration_ms).unwrap_or(i64::MAX)),
    )
    .await;

    tracing::info!(
        marketplace_id,
        plugins = success_count,
        errors = error_count,
        duration_ms,
        "Local marketplace sync completed"
    );

    Ok(SyncResult {
        commit_hash: String::new(),
        plugins_synced: success_count,
        errors: error_count,
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

fn git_add_all(repo_path: &Path) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(repo_path)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git add failed: {stderr}");
    }
    Ok(())
}

fn git_has_changes(repo_path: &Path) -> Result<bool> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_path)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git status failed: {stderr}");
    }
    Ok(!output.stdout.is_empty())
}

fn git_commit(repo_path: &Path, message: &str) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(repo_path)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git commit failed: {stderr}");
    }
    Ok(())
}

fn git_push(repo_path: &Path, remote_url: &str) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["push", remote_url, "HEAD"])
        .current_dir(repo_path)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git push failed: {stderr}");
    }
    Ok(())
}

fn build_authenticated_url(repo_url: &str) -> Result<String> {
    let token = std::env::var("GITHUB_MARKETPLACE_TOKEN").unwrap_or_default();
    if token.is_empty() {
        return Ok(repo_url.to_string());
    }

    // Rewrite https://github.com/... to https://{token}@github.com/...
    if let Some(rest) = repo_url.strip_prefix("https://") {
        Ok(format!("https://{token}@{rest}"))
    } else {
        Ok(repo_url.to_string())
    }
}
