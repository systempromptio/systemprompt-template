use std::path::PathBuf;

use systemprompt::traits::{Job, JobContext, JobResult};

use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Clone, Copy, Default)]
pub struct BundleAdminJsJob;

impl BundleAdminJsJob {
    pub async fn execute_bundle() -> Result<JobResult, MarketplaceError> {
        let start_time = std::time::Instant::now();

        tracing::info!("Bundle admin JS job started");

        let js_dir = std::env::current_dir()
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Failed to get current directory, using fallback");
                PathBuf::from(".")
            })
            .join("storage")
            .join("files")
            .join("js")
            .join("admin");

        let output_dir = js_dir.parent().unwrap_or(&js_dir).to_path_buf();

        let mut total_bundled = 0u64;

        total_bundled += build_per_page_bundles(&js_dir, &output_dir).await?;

        let (bundled, total_failed) = build_main_bundle(&js_dir, &output_dir).await?;
        total_bundled += bundled;

        let duration_ms = u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);

        tracing::info!(total_bundled, duration_ms, "Bundle admin JS job completed");

        Ok(JobResult::success()
            .with_stats(total_bundled, total_failed)
            .with_duration(duration_ms))
    }
}

async fn build_per_page_bundles(
    js_dir: &std::path::Path,
    output_dir: &std::path::Path,
) -> Result<u64, MarketplaceError> {
    let bundles_dir = js_dir.join("bundles");
    if !bundles_dir.is_dir() {
        return Ok(0);
    }

    let entries = read_bundle_manifests(&bundles_dir)?;
    let mut total_bundled = 0u64;

    for entry in &entries {
        total_bundled += process_single_bundle(entry, js_dir, output_dir).await?;
    }

    Ok(total_bundled)
}

fn read_bundle_manifests(
    bundles_dir: &std::path::Path,
) -> Result<Vec<std::fs::DirEntry>, MarketplaceError> {
    let mut entries: Vec<_> = std::fs::read_dir(bundles_dir)
        .map_err(|e| {
            MarketplaceError::Internal(format!(
                "Failed to read bundles dir: {}: {e}",
                bundles_dir.display()
            ))
        })?
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "txt"))
        .collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    Ok(entries)
}

async fn process_single_bundle(
    entry: &std::fs::DirEntry,
    js_dir: &std::path::Path,
    output_dir: &std::path::Path,
) -> Result<u64, MarketplaceError> {
    let manifest_path = entry.path();
    let bundle_name = manifest_path
        .file_stem()
        .unwrap_or_else(|| std::ffi::OsStr::new(""))
        .to_string_lossy()
        .to_string();

    let manifest = tokio::fs::read_to_string(&manifest_path)
        .await
        .map_err(|e| {
            MarketplaceError::Internal(format!(
                "Failed to read bundle manifest: {}: {e}",
                manifest_path.display()
            ))
        })?;

    let files: Vec<&str> = manifest
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();

    if files.is_empty() {
        tracing::warn!(bundle = %bundle_name, "Bundle manifest is empty, skipping");
        return Ok(0);
    }

    let (content, bundled, failed) = concatenate_files(js_dir, &files).await;

    if failed > 0 {
        return Err(MarketplaceError::Internal(format!(
            "Failed to read {failed} JS file(s) for bundle '{bundle_name}'"
        )));
    }

    let bundle_path = output_dir.join(format!("admin-{bundle_name}.js"));
    tokio::fs::write(&bundle_path, &content)
        .await
        .map_err(|e| {
            MarketplaceError::Internal(format!(
                "Failed to write bundle: {}: {e}",
                bundle_path.display()
            ))
        })?;

    tracing::info!(
        bundle = %bundle_name,
        files = bundled,
        size = content.len(),
        "Per-page bundle written"
    );

    Ok(bundled)
}

async fn build_main_bundle(
    js_dir: &std::path::Path,
    output_dir: &std::path::Path,
) -> Result<(u64, u64), MarketplaceError> {
    let manifest_path = js_dir.join("bundle-order.txt");
    let bundle_path = output_dir.join("admin-bundle.js");

    let manifest = tokio::fs::read_to_string(&manifest_path)
        .await
        .map_err(|e| {
            MarketplaceError::Internal(format!(
                "Failed to read manifest: {}: {e}",
                manifest_path.display()
            ))
        })?;

    let files: Vec<&str> = manifest
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();

    if files.is_empty() {
        tracing::warn!("Bundle manifest is empty");
        return Ok((0, 0));
    }

    let (content, bundled, failed) = concatenate_files(js_dir, &files).await;

    if failed > 0 {
        return Err(MarketplaceError::Internal(format!(
            "Failed to read {failed} JS file(s) during bundling"
        )));
    }

    tokio::fs::write(&bundle_path, &content)
        .await
        .map_err(|e| {
            MarketplaceError::Internal(format!(
                "Failed to write bundle: {}: {e}",
                bundle_path.display()
            ))
        })?;

    Ok((bundled, failed))
}

async fn concatenate_files(js_dir: &std::path::Path, files: &[&str]) -> (String, u64, u64) {
    let mut content = String::new();
    let mut bundled = 0u64;
    let mut failed = 0u64;

    for filename in files {
        let file_path = js_dir.join(filename);
        match tokio::fs::read_to_string(&file_path).await {
            Ok(file_content) => {
                if !content.is_empty() {
                    content.push('\n');
                }
                content.push_str(&file_content);
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

    (content, bundled, failed)
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

    async fn execute(&self, _ctx: &JobContext) -> anyhow::Result<JobResult> {
        Ok(Self::execute_bundle().await?)
    }
}

systemprompt::traits::submit_job!(&BundleAdminJsJob);
