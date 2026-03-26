use std::path::{Path, PathBuf};
use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::models::ProfileBootstrap;

use super::export::{generate_export_bundles, generate_org_marketplace_export_bundles};

pub const CACHE_DIR: &str = "/tmp/foodles-marketplace-repos";
const CACHE_TTL_SECS: u64 = 300;

pub async fn get_or_generate_marketplace_repo(
    pool: &Arc<PgPool>,
    user_id: &str,
    platform: &str,
) -> Result<PathBuf, anyhow::Error> {
    let persistent_repo = PathBuf::from("storage/marketplace-versions")
        .join(user_id)
        .join("repo.git");
    if persistent_repo.join("HEAD").exists() {
        return Ok(persistent_repo);
    }

    let repo_path = PathBuf::from(CACHE_DIR)
        .join(user_id)
        .join(platform)
        .join("repo.git");

    if is_cache_valid(&repo_path) {
        return Ok(repo_path);
    }

    let _ = super::marketplace_sync_status::mark_user_dirty(pool, user_id).await;

    let (username, email, roles, department) = lookup_user_basic(pool, user_id).await?;

    let services_path = ProfileBootstrap::get()
        .map(|p| PathBuf::from(&p.paths.services))
        .map_err(|e| anyhow::anyhow!("Failed to get profile: {e}"))?;

    let response = generate_export_bundles(
        &services_path,
        pool,
        user_id,
        &username,
        &email,
        &roles,
        &department,
        platform,
    )
    .await?;

    let base_dir = PathBuf::from(CACHE_DIR).join(user_id).join(platform);
    let work_dir = base_dir.join("work");

    if base_dir.exists() {
        std::fs::remove_dir_all(&base_dir)?;
    }
    std::fs::create_dir_all(&work_dir)?;

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
            std::fs::write(&file_path, &file.content)?;
            if file.executable {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o755))?;
                }
            }
        }
    }

    create_bare_repo(&work_dir, &repo_path)?;

    Ok(repo_path)
}

pub async fn get_or_generate_org_marketplace_repo(
    pool: &Arc<PgPool>,
    marketplace_id: &str,
    platform: &str,
) -> Result<PathBuf, anyhow::Error> {
    let repo_path = PathBuf::from(CACHE_DIR)
        .join("org")
        .join(marketplace_id)
        .join(platform)
        .join("repo.git");

    if is_cache_valid(&repo_path) {
        return Ok(repo_path);
    }

    let services_path = ProfileBootstrap::get()
        .map(|p| PathBuf::from(&p.paths.services))
        .map_err(|e| anyhow::anyhow!("Failed to get profile: {e}"))?;

    let response = generate_org_marketplace_export_bundles(
        &services_path,
        pool,
        marketplace_id,
        platform,
    )
    .await?;

    let base_dir = PathBuf::from(CACHE_DIR)
        .join("org")
        .join(marketplace_id)
        .join(platform);
    let work_dir = base_dir.join("work");

    if base_dir.exists() {
        std::fs::remove_dir_all(&base_dir)?;
    }
    std::fs::create_dir_all(&work_dir)?;

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
            std::fs::write(&file_path, &file.content)?;
            if file.executable {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o755))?;
                }
            }
        }
    }

    create_bare_repo(&work_dir, &repo_path)?;

    Ok(repo_path)
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

fn create_bare_repo(work_dir: &Path, repo_path: &Path) -> Result<(), anyhow::Error> {
    use std::process::Command;

    let run = |args: &[&str], dir: &Path| -> Result<(), anyhow::Error> {
        let output = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "Foodles")
            .env("GIT_AUTHOR_EMAIL", "support@foodles.com")
            .env("GIT_COMMITTER_NAME", "Foodles")
            .env("GIT_COMMITTER_EMAIL", "support@foodles.com")
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git {} failed: {}", args.join(" "), stderr);
        }
        Ok(())
    };

    run(&["init"], work_dir)?;
    run(&["add", "-A"], work_dir)?;
    run(&["commit", "-m", "marketplace export"], work_dir)?;
    run(
        &[
            "clone",
            "--bare",
            work_dir.to_str().expect("work_dir path is valid UTF-8"),
            repo_path.to_str().expect("repo_path path is valid UTF-8"),
        ],
        work_dir.parent().expect("work_dir has a parent directory"),
    )?;
    run(&["update-server-info"], repo_path)?;

    std::fs::remove_dir_all(work_dir)?;

    Ok(())
}

#[allow(clippy::type_complexity)]
pub async fn lookup_user_basic(
    pool: &Arc<PgPool>,
    user_id: &str,
) -> Result<(String, String, Vec<String>, String), anyhow::Error> {
    let row: Option<(Option<String>, Option<String>, Vec<String>, String)> = sqlx::query_as(
        "SELECT COALESCE(display_name, full_name, name), email, roles, COALESCE(department, '') FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool.as_ref())
    .await?;

    match row {
        Some((name, email, roles, department)) => Ok((
            name.unwrap_or_else(String::new),
            email.unwrap_or_else(String::new),
            roles,
            department,
        )),
        None => anyhow::bail!("User not found: {user_id}"),
    }
}
