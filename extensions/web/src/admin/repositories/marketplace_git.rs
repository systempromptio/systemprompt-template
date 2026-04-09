use std::path::{Path, PathBuf};

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt::models::ProfileBootstrap;

use super::export::generate_export_bundles;
use crate::error::MarketplaceError;

pub const CACHE_DIR: &str = "/tmp/systemprompt-marketplace-repos";
const CACHE_TTL_SECS: u64 = 300;

pub async fn get_or_generate_marketplace_repo(
    pool: &PgPool,
    user_id: &UserId,
    platform: &str,
) -> Result<PathBuf, MarketplaceError> {
    generate_repo(pool, user_id, platform, "repo.git", true).await
}

pub async fn get_or_generate_cowork_repo(
    pool: &PgPool,
    user_id: &UserId,
    platform: &str,
) -> Result<PathBuf, MarketplaceError> {
    generate_repo(pool, user_id, platform, "cowork-repo.git", false).await
}

async fn generate_repo(
    pool: &PgPool,
    user_id: &UserId,
    platform: &str,
    repo_name: &str,
    strip_hooks: bool,
) -> Result<PathBuf, MarketplaceError> {
    let persistent_repo = PathBuf::from("storage/marketplace-versions")
        .join(user_id.as_str())
        .join(repo_name);
    if persistent_repo.join("HEAD").exists() {
        return Ok(persistent_repo);
    }

    let repo_path = PathBuf::from(CACHE_DIR)
        .join(user_id.as_str())
        .join(platform)
        .join(repo_name);

    if is_cache_valid(&repo_path) {
        return Ok(repo_path);
    }

    if let Err(e) = super::marketplace_sync_status::mark_user_dirty(pool, user_id).await {
        tracing::warn!(error = %e, "Failed to mark user dirty");
    }

    let user_info = lookup_user_basic(pool, user_id).await?;

    let services_path = ProfileBootstrap::get()
        .map(|p| PathBuf::from(&p.paths.services))
        .map_err(|e| MarketplaceError::Internal(format!("Failed to get profile: {e}")))?;

    let export_params = super::export::ExportParams {
        services_path: &services_path,
        pool,
        user_id,
        username: &user_info.display_name,
        email: &user_info.email,
        roles: &user_info.roles,
    };
    let response = generate_export_bundles(&export_params).await?;

    let base_dir = PathBuf::from(CACHE_DIR)
        .join(user_id.as_str())
        .join(platform);
    let work_dir = base_dir.join(format!("{repo_name}-work"));

    write_export_to_disk(&response, &work_dir, strip_hooks)?;
    create_bare_repo(
        &work_dir,
        &repo_path,
        &user_info.display_name,
        &user_info.email,
    )?;

    Ok(repo_path)
}

fn write_export_to_disk(
    response: &super::export::SyncPluginsResponse,
    work_dir: &Path,
    strip_hooks: bool,
) -> Result<(), MarketplaceError> {
    if work_dir.exists() {
        std::fs::remove_dir_all(work_dir)?;
    }
    std::fs::create_dir_all(work_dir)?;

    let marketplace_path = work_dir.join(&response.marketplace.path);
    if let Some(parent) = marketplace_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&marketplace_path, &response.marketplace.content)?;

    for bundle in &response.plugins {
        for file in &bundle.files {
            let file_path = work_dir.join("plugins").join(&bundle.id).join(&file.path);
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let content = if strip_hooks && file.path == ".claude-plugin/plugin.json" {
                strip_manifest_hooks(&file.content)
            } else {
                file.content.clone()
            };
            std::fs::write(&file_path, &content)?;
            if file.executable {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o755))?;
                }
            }
        }
    }
    Ok(())
}

fn strip_manifest_hooks(content: &str) -> String {
    match serde_json::from_str::<super::export::PluginManifest>(content) {
        Ok(mut manifest) => {
            manifest.hooks = None;
            serde_json::to_string_pretty(&manifest).unwrap_or_else(|_| content.to_string())
        }
        Err(_) => content.to_string(),
    }
}

fn is_cache_valid(repo_path: &Path) -> bool {
    let info_refs = repo_path.join("info").join("refs");
    if let Ok(metadata) = std::fs::metadata(&info_refs) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(elapsed) = modified.elapsed() {
                return elapsed.as_secs() < CACHE_TTL_SECS;
            }
        }
    }
    false
}

fn create_bare_repo(
    work_dir: &Path,
    repo_path: &Path,
    username: &str,
    email: &str,
) -> Result<(), MarketplaceError> {
    use std::process::Command;

    let author_name = if username.is_empty() {
        "systemprompt.io"
    } else {
        username
    };
    let author_email = if email.is_empty() {
        "ed@systemprompt.io"
    } else {
        email
    };

    let run = |args: &[&str], dir: &Path| -> Result<(), MarketplaceError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", author_name)
            .env("GIT_AUTHOR_EMAIL", author_email)
            .env("GIT_COMMITTER_NAME", author_name)
            .env("GIT_COMMITTER_EMAIL", author_email)
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(MarketplaceError::Internal(format!(
                "git {} failed: {}",
                args.join(" "),
                stderr
            )));
        }
        Ok(())
    };

    run(&["init"], work_dir)?;
    run(&["add", "-A"], work_dir)?;
    run(&["commit", "-m", "marketplace export"], work_dir)?;
    let work_dir_str = work_dir
        .to_str()
        .ok_or_else(|| MarketplaceError::Internal("work_dir path is not valid UTF-8".into()))?;
    let repo_path_str = repo_path
        .to_str()
        .ok_or_else(|| MarketplaceError::Internal("repo_path is not valid UTF-8".into()))?;
    let parent_dir = work_dir
        .parent()
        .ok_or_else(|| MarketplaceError::Internal("work_dir has no parent directory".into()))?;
    run(
        &["clone", "--bare", work_dir_str, repo_path_str],
        parent_dir,
    )?;
    run(&["update-server-info"], repo_path)?;

    std::fs::remove_dir_all(work_dir)?;

    Ok(())
}

pub async fn lookup_user_basic(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<crate::admin::types::UserBasicInfo, MarketplaceError> {
    let row = sqlx::query!(
        "SELECT COALESCE(display_name, full_name, name) as display_name, email, roles FROM users WHERE id = $1",
        user_id.as_str(),
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| MarketplaceError::NotFound(format!("User not found: {user_id}")))?;

    Ok(crate::admin::types::UserBasicInfo {
        display_name: row.display_name.unwrap_or_else(String::new),
        email: row.email,
        roles: row.roles,
    })
}
