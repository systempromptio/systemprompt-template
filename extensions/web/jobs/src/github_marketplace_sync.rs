use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use systemprompt_web_admin::repositories;
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Clone, Copy, Default)]
pub struct GitHubMarketplaceSyncJob;

#[async_trait::async_trait]
impl Job for GitHubMarketplaceSyncJob {
    fn name(&self) -> &'static str {
        "github_marketplace_sync"
    }

    fn description(&self) -> &'static str {
        "Syncs all GitHub-connected org marketplaces into local plugin storage"
    }

    fn schedule(&self) -> &'static str {
        "0 0 */6 * * *"
    }

    fn run_on_startup(&self) -> bool {
        true
    }

    async fn execute(&self, ctx: &JobContext) -> anyhow::Result<JobResult> {
        let start = std::time::Instant::now();

        tracing::info!("GitHub marketplace sync job started");

        let db = ctx.db_pool::<DbPool>().ok_or(MarketplaceError::Internal(
            "Database not available in job context".to_string(),
        ))?;

        let pool = db.pool().ok_or(MarketplaceError::Internal(
            "PgPool not available from database".to_string(),
        ))?;

        let marketplaces = repositories::org_marketplaces::list_github_marketplaces(&pool)
            .await
            .map_err(|e| {
                MarketplaceError::Internal(format!("Failed to list GitHub marketplaces: {e}"))
            })?;

        if marketplaces.is_empty() {
            tracing::info!("No GitHub-connected marketplaces found");
            let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
            return Ok(JobResult::success()
                .with_stats(0, 0)
                .with_duration(duration_ms));
        }

        let mut total_success = 0u64;
        let mut total_errors = 0u64;

        for mkt in &marketplaces {
            let Some(ref repo_url) = mkt.github_repo_url else {
                continue;
            };

            match repositories::github_sync::sync_marketplace_from_github(
                &pool, &mkt.id, repo_url, "cron",
            )
            .await
            {
                Ok(result) => {
                    total_success += result.plugins_synced;
                    total_errors += result.errors;
                }
                Err(e) => {
                    tracing::error!(
                        marketplace_id = %mkt.id,
                        error = %e,
                        "Failed to sync marketplace from GitHub"
                    );
                    let _ = repositories::org_marketplaces::insert_sync_log(
                        &pool,
                        &repositories::org_marketplaces::SyncLogEntry {
                            marketplace_id: &mkt.id,
                            operation: "sync",
                            status: "error",
                            commit_hash: None,
                            plugins_synced: 0,
                            errors: 1,
                            error_message: Some(&e.to_string()),
                            triggered_by: "cron",
                            duration_ms: None,
                        },
                    )
                    .await;
                    total_errors += 1;
                }
            }
        }

        for mkt in &marketplaces {
            match repositories::github_sync::sync_marketplace_from_local(&pool, &mkt.id).await {
                Ok(result) => {
                    total_success += result.plugins_synced;
                    total_errors += result.errors;
                }
                Err(e) => {
                    tracing::warn!(
                        marketplace_id = %mkt.id,
                        error = %e,
                        "Failed to sync marketplace from local plugins"
                    );
                }
            }
        }

        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

        tracing::info!(
            marketplaces = marketplaces.len(),
            plugins = total_success,
            errors = total_errors,
            duration_ms,
            "GitHub marketplace sync job completed"
        );

        Ok(JobResult::success()
            .with_stats(total_success, total_errors)
            .with_duration(duration_ms))
    }
}

systemprompt::traits::submit_job!(&GitHubMarketplaceSyncJob);
