use std::path::PathBuf;

use systemprompt::traits::{Job, JobContext, JobResult};

use crate::error::MarketplaceError;

#[derive(Debug, Clone, Copy, Default)]
pub struct BundleAdminCssJob;

impl BundleAdminCssJob {
    pub async fn execute_bundle() -> anyhow::Result<JobResult> {
        let start_time = std::time::Instant::now();

        tracing::info!("Bundle admin CSS job started");

        let css_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("storage")
            .join("files")
            .join("css")
            .join("admin");

        let bundle_path = css_dir
            .parent()
            .unwrap_or(&css_dir)
            .join("admin-bundle.css");

        let mut css_files: Vec<PathBuf> = Vec::new();
        let mut read_dir = tokio::fs::read_dir(&css_dir).await.map_err(|e| {
            MarketplaceError::Internal(format!(
                "Failed to read CSS directory: {}: {e}",
                css_dir.display()
            ))
        })?;

        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("css") {
                css_files.push(path);
            }
        }

        css_files.sort();

        let mut bundle = String::new();
        let mut bundled = 0u64;
        let mut failed = 0u64;

        for file_path in &css_files {
            let filename = file_path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new(""))
                .to_string_lossy();
            match tokio::fs::read_to_string(file_path).await {
                Ok(content) => {
                    if !bundle.is_empty() {
                        bundle.push('\n');
                    }
                    bundle.push_str(&content);
                    bundled += 1;
                }
                Err(e) => {
                    tracing::error!(
                        file = %filename,
                        error = %e,
                        "Failed to read CSS file for bundling"
                    );
                    failed += 1;
                }
            }
        }

        if failed > 0 {
            return Err(MarketplaceError::Internal(format!(
                "Failed to read {failed} CSS file(s) during bundling"
            ))
            .into());
        }

        tokio::fs::write(&bundle_path, &bundle).await.map_err(|e| {
            MarketplaceError::Internal(format!(
                "Failed to write bundle: {}: {e}",
                bundle_path.display()
            ))
        })?;

        let duration_ms = u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);

        tracing::info!(
            bundled,
            bundle_size = bundle.len(),
            duration_ms,
            "Bundle admin CSS job completed"
        );

        Ok(JobResult::success()
            .with_stats(bundled, failed)
            .with_duration(duration_ms))
    }
}

#[async_trait::async_trait]
impl Job for BundleAdminCssJob {
    fn name(&self) -> &'static str {
        "bundle_admin_css"
    }

    fn description(&self) -> &'static str {
        "Concatenates admin CSS modules into admin-bundle.css"
    }

    fn schedule(&self) -> &'static str {
        "0 */15 * * * *"
    }

    fn run_on_startup(&self) -> bool {
        true
    }

    async fn execute(&self, _ctx: &JobContext) -> anyhow::Result<JobResult> {
        Self::execute_bundle().await
    }
}

systemprompt::traits::submit_job!(&BundleAdminCssJob);
