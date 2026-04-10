use std::path::Path;

use systemprompt::extension::{AssetDefinition, ExtensionRegistry};
use systemprompt::models::AppPaths;
use systemprompt::traits::{Job, JobContext, JobResult};

use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Clone, Copy, Default)]
pub struct CopyExtensionAssetsJob;

impl CopyExtensionAssetsJob {
    pub async fn execute_copy() -> Result<JobResult, MarketplaceError> {
        let start_time = std::time::Instant::now();

        tracing::info!("Copy extension assets job started");

        let paths = AppPaths::get()
            .map_err(|e| MarketplaceError::Internal(format!("AppPaths not initialized: {e}")))?;

        let registry = ExtensionRegistry::discover();
        let assets = registry.all_required_assets(paths);

        if assets.is_empty() {
            let duration_ms = u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);
            tracing::info!(duration_ms, "No extension assets to copy");
            return Ok(JobResult::success()
                .with_stats(0, 0)
                .with_duration(duration_ms));
        }

        let (copied, failed) = copy_all_assets(paths.web().dist(), assets).await?;

        let duration_ms = u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);

        tracing::info!(
            copied,
            failed,
            duration_ms,
            "Copy extension assets job completed"
        );

        Ok(JobResult::success()
            .with_stats(copied, failed)
            .with_duration(duration_ms))
    }
}

async fn copy_all_assets(
    dist_dir: &Path,
    assets: Vec<(&str, AssetDefinition)>,
) -> Result<(u64, u64), MarketplaceError> {
    let mut copied = 0u64;
    let mut failed = 0u64;

    for (ext_id, asset) in assets {
        match copy_asset(dist_dir, ext_id, &asset).await {
            Ok(()) => copied += 1,
            Err(e) => {
                if asset.is_required() {
                    return Err(e);
                }
                tracing::warn!(
                    extension = %ext_id,
                    asset = %asset.source().display(),
                    error = %e,
                    "Optional asset copy failed"
                );
                failed += 1;
            }
        }
    }

    Ok((copied, failed))
}

async fn copy_asset(
    dist_dir: &Path,
    ext_id: &str,
    asset: &AssetDefinition,
) -> Result<(), MarketplaceError> {
    let dest_path = dist_dir.join(asset.destination());

    if let Some(parent) = dest_path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            MarketplaceError::Internal(format!(
                "Failed to create directory: {}: {e}",
                parent.display()
            ))
        })?;
    }

    tokio::fs::copy(asset.source(), &dest_path)
        .await
        .map_err(|e| {
            MarketplaceError::Internal(format!(
                "Failed to copy asset from {} to {}: {e}",
                asset.source().display(),
                dest_path.display()
            ))
        })?;

    tracing::debug!(
        extension = %ext_id,
        source = %asset.source().display(),
        destination = %dest_path.display(),
        "Copied extension asset"
    );

    Ok(())
}

#[async_trait::async_trait]
impl Job for CopyExtensionAssetsJob {
    fn name(&self) -> &'static str {
        "copy_extension_assets"
    }

    fn description(&self) -> &'static str {
        "Copies extension assets (CSS, JS) to web dist directory"
    }

    fn schedule(&self) -> &'static str {
        "0 */15 * * * *"
    }

    async fn execute(&self, _ctx: &JobContext) -> anyhow::Result<JobResult> {
        Ok(Self::execute_copy().await?)
    }
}

systemprompt::traits::submit_job!(&CopyExtensionAssetsJob);
