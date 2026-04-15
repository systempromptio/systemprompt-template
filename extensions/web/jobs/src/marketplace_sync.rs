use std::path::PathBuf;

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::identifiers::UserId;
use systemprompt::models::ProfileBootstrap;
use systemprompt::traits::{Job, JobContext, JobResult};

use systemprompt_web_admin::repositories;
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Clone, Copy, Default)]
pub struct MarketplaceSyncJob;

#[async_trait::async_trait]
impl Job for MarketplaceSyncJob {
    fn name(&self) -> &'static str {
        "marketplace_sync"
    }

    fn description(&self) -> &'static str {
        "Syncs marketplace files for dirty users to persistent storage"
    }

    fn schedule(&self) -> &'static str {
        "0 */5 * * * *"
    }

    fn run_on_startup(&self) -> bool {
        false
    }

    async fn execute(&self, ctx: &JobContext) -> anyhow::Result<JobResult> {
        let start = std::time::Instant::now();

        tracing::info!("Marketplace sync job started");

        let db = ctx.db_pool::<DbPool>().ok_or(MarketplaceError::Internal(
            "Database not available in job context".to_string(),
        ))?;

        let pool = db.pool().ok_or(MarketplaceError::Internal(
            "PgPool not available from database".to_string(),
        ))?;

        let dirty_users = repositories::marketplace_sync_status::get_dirty_users(&pool, 50)
            .await
            .map_err(|e| MarketplaceError::Internal(format!("Failed to query dirty users: {e}")))?;

        let total = dirty_users.len() as u64;
        let mut success_count = 0u64;
        let mut error_count = 0u64;

        for user_id in &dirty_users {
            match generate_and_persist_marketplace(&pool, user_id).await {
                Ok(()) => {
                    if let Err(e) =
                        repositories::marketplace_sync_status::mark_user_synced(&pool, user_id)
                            .await
                    {
                        tracing::warn!(user_id = %user_id, error = %e, "Failed to mark user synced");
                        error_count += 1;
                    } else {
                        success_count += 1;
                        tracing::debug!(user_id = %user_id, "Marketplace synced successfully");
                    }
                }
                Err(e) => {
                    error_count += 1;
                    let err_msg = format!("{e}");
                    tracing::warn!(user_id = %user_id, error = %e, "Failed to sync marketplace");
                    if let Err(mark_err) = repositories::marketplace_sync_status::mark_sync_error(
                        &pool, user_id, &err_msg,
                    )
                    .await
                    {
                        tracing::warn!(user_id = %user_id, error = %mark_err, "Failed to mark sync error");
                    }
                }
            }
        }

        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

        tracing::info!(
            total,
            success = success_count,
            errors = error_count,
            duration_ms,
            "Marketplace sync job completed"
        );

        Ok(JobResult::success()
            .with_stats(success_count, error_count)
            .with_duration(duration_ms))
    }
}

pub async fn generate_and_persist_marketplace(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<(), MarketplaceError> {
    let user_basic = repositories::marketplace_git::lookup_user_basic(pool, user_id)
        .await
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;

    let services_path = ProfileBootstrap::get()
        .map(|p| PathBuf::from(&p.paths.services))
        .map_err(|e| MarketplaceError::Internal(format!("Failed to get profile: {e}")))?;

    let params = repositories::ExportParams {
        services_path: &services_path,
        pool,
        user_id,
        username: &user_basic.display_name,
        email: &user_basic.email,
        roles: &user_basic.roles,
    };
    let response = repositories::generate_export_bundles(&params)
        .await
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;

    let base_dir = PathBuf::from("storage/marketplace-versions").join(user_id.as_str());
    let work_dir = base_dir.join("work");

    if work_dir.exists() {
        std::fs::remove_dir_all(&work_dir)?;
    }
    std::fs::create_dir_all(&work_dir)?;

    let marketplace_path = base_dir.join("marketplace.json");
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

    let work_marketplace_path = work_dir.join(&response.marketplace.path);
    if let Some(parent) = work_marketplace_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&work_marketplace_path, &response.marketplace.content)?;

    let repo_path = base_dir.join("repo.git");
    if repo_path.exists() {
        std::fs::remove_dir_all(&repo_path)?;
    }
    create_bare_repo(&work_dir, &repo_path)?;

    let _ = repositories::marketplace_sync::invalidate_git_cache(user_id);

    Ok(())
}

fn create_bare_repo(
    work_dir: &std::path::Path,
    repo_path: &std::path::Path,
) -> Result<(), MarketplaceError> {
    use std::process::Command;

    let (git_name, git_email) = Option::<systemprompt_web_shared::BrandingConfig>::None
        .map_or_else(
            || {
                (
                    "SystemPrompt".to_string(),
                    "support@systemprompt.io".to_string(),
                )
            },
            |b| {
                let name = if b.display_name.is_empty() {
                    "SystemPrompt".to_string()
                } else {
                    b.display_name
                };
                let email = if b.support_email.is_empty() {
                    "support@systemprompt.io".to_string()
                } else {
                    b.support_email
                };
                (name, email)
            },
        );

    let run = |args: &[&str], dir: &std::path::Path| -> Result<(), MarketplaceError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", &git_name)
            .env("GIT_AUTHOR_EMAIL", &git_email)
            .env("GIT_COMMITTER_NAME", &git_name)
            .env("GIT_COMMITTER_EMAIL", &git_email)
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
    run(
        &[
            "clone",
            "--bare",
            work_dir.to_str().ok_or_else(|| {
                MarketplaceError::Internal("work_dir path is not valid UTF-8".to_string())
            })?,
            repo_path.to_str().ok_or_else(|| {
                MarketplaceError::Internal("repo_path path is not valid UTF-8".to_string())
            })?,
        ],
        work_dir.parent().ok_or_else(|| {
            MarketplaceError::Internal("work_dir has no parent directory".to_string())
        })?,
    )?;
    run(&["update-server-info"], repo_path)?;

    std::fs::remove_dir_all(work_dir)?;

    Ok(())
}

systemprompt::traits::submit_job!(&MarketplaceSyncJob);
