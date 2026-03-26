use anyhow::{Context, Result};
use std::path::PathBuf;
use systemprompt::traits::{Job, JobContext, JobResult};

#[derive(Debug, Clone, Copy, Default)]
pub struct BundleAdminJsJob;

impl BundleAdminJsJob {
    #[allow(clippy::too_many_lines)]
    pub async fn execute_bundle() -> Result<JobResult> {
        let start_time = std::time::Instant::now();

        tracing::info!("Bundle admin JS job started");

        let js_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("storage")
            .join("files")
            .join("js")
            .join("admin");

        let output_dir = js_dir.parent().unwrap_or(&js_dir).to_path_buf();

        let mut total_bundled = 0u64;
        let mut total_failed = 0u64;

        let bundles_dir = js_dir.join("bundles");
        if bundles_dir.is_dir() {
            let mut entries: Vec<_> = std::fs::read_dir(&bundles_dir)
                .with_context(|| format!("Failed to read bundles dir: {}", bundles_dir.display()))?
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "txt"))
                .collect();
            entries.sort_by_key(std::fs::DirEntry::file_name);

            for entry in &entries {
                let manifest_path = entry.path();
                let bundle_name = manifest_path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let manifest = tokio::fs::read_to_string(&manifest_path)
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to read bundle manifest: {}",
                            manifest_path.display()
                        )
                    })?;

                let files: Vec<&str> = manifest
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .collect();

                if files.is_empty() {
                    tracing::warn!(bundle = %bundle_name, "Bundle manifest is empty, skipping");
                    continue;
                }

                let mut bundle_content = String::new();
                let mut bundled = 0u64;
                let mut failed = 0u64;

                for filename in &files {
                    let file_path = js_dir.join(filename);
                    match tokio::fs::read_to_string(&file_path).await {
                        Ok(content) => {
                            if !bundle_content.is_empty() {
                                bundle_content.push('\n');
                            }
                            bundle_content.push_str(&content);
                            bundled += 1;
                        }
                        Err(e) => {
                            tracing::error!(
                                bundle = %bundle_name,
                                file = %filename,
                                error = %e,
                                "Failed to read JS file for bundle"
                            );
                            failed += 1;
                        }
                    }
                }

                if failed > 0 {
                    return Err(anyhow::anyhow!(
                        "Failed to read {failed} JS file(s) for bundle '{bundle_name}'"
                    ));
                }

                let bundle_path = output_dir.join(format!("admin-{bundle_name}.js"));
                tokio::fs::write(&bundle_path, &bundle_content)
                    .await
                    .with_context(|| {
                        format!("Failed to write bundle: {}", bundle_path.display())
                    })?;

                tracing::info!(
                    bundle = %bundle_name,
                    files = bundled,
                    size = bundle_content.len(),
                    "Per-page bundle written"
                );

                total_bundled += bundled;
            }
        }

        let manifest_path = js_dir.join("bundle-order.txt");
        let bundle_path = output_dir.join("admin-bundle.js");

        let manifest = tokio::fs::read_to_string(&manifest_path)
            .await
            .with_context(|| format!("Failed to read manifest: {}", manifest_path.display()))?;

        let files: Vec<&str> = manifest
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();

        if files.is_empty() {
            let duration_ms = u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);
            tracing::warn!("Bundle manifest is empty");
            return Ok(JobResult::success()
                .with_stats(total_bundled, total_failed)
                .with_duration(duration_ms));
        }

        let mut bundle = String::new();
        let mut bundled = 0u64;
        let mut failed = 0u64;

        for filename in &files {
            let file_path = js_dir.join(filename);
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
                        "Failed to read JS file for bundling"
                    );
                    failed += 1;
                }
            }
        }

        if failed > 0 {
            return Err(anyhow::anyhow!(
                "Failed to read {failed} JS file(s) during bundling"
            ));
        }

        tokio::fs::write(&bundle_path, &bundle)
            .await
            .with_context(|| format!("Failed to write bundle: {}", bundle_path.display()))?;

        total_bundled += bundled;
        total_failed += failed;

        let duration_ms = u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);

        tracing::info!(
            total_bundled,
            bundle_size = bundle.len(),
            duration_ms,
            "Bundle admin JS job completed"
        );

        Ok(JobResult::success()
            .with_stats(total_bundled, total_failed)
            .with_duration(duration_ms))
    }
}

#[async_trait::async_trait]
impl Job for BundleAdminJsJob {
    fn name(&self) -> &'static str {
        "bundle_admin_js"
    }

    fn description(&self) -> &'static str {
        "Concatenates admin JS files into per-page bundles and admin-bundle.js"
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

systemprompt::traits::submit_job!(&BundleAdminJsJob);
