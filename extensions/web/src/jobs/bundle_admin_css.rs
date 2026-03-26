use anyhow::{Context, Result};
use std::path::PathBuf;
use systemprompt::traits::{Job, JobContext, JobResult};

const CSS_MODULE_ORDER: &[&str] = &[
    "tokens.css",
    "base.css",
    "sidebar.css",
    "tables.css",
    "dashboard.css",
    "components.css",
    "hooks.css",
    "panels.css",
    "plugins.css",
    "gamification.css",
    "access-control.css",
    "org-views.css",
    "login.css",
    "responsive.css",
];

#[derive(Debug, Clone, Copy, Default)]
pub struct BundleAdminCssJob;

impl BundleAdminCssJob {
    pub async fn execute_bundle() -> Result<JobResult> {
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

        let mut bundle = String::new();
        let mut bundled = 0u64;
        let mut failed = 0u64;

        for filename in CSS_MODULE_ORDER {
            let file_path = css_dir.join(filename);
            match tokio::fs::read_to_string(&file_path).await {
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
            return Err(anyhow::anyhow!(
                "Failed to read {failed} CSS file(s) during bundling"
            ));
        }

        tokio::fs::write(&bundle_path, &bundle)
            .await
            .with_context(|| format!("Failed to write bundle: {}", bundle_path.display()))?;

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

    async fn execute(&self, _ctx: &JobContext) -> Result<JobResult> {
        Self::execute_bundle().await
    }
}

systemprompt::traits::submit_job!(&BundleAdminCssJob);
