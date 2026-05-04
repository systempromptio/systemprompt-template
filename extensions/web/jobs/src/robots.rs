use std::fmt::Write as FmtWrite;
use std::sync::Arc;

use systemprompt::models::AppPaths;
use systemprompt::traits::{Job, JobContext, JobResult};

use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Clone, Copy, Default)]
pub struct RobotsTxtGenerationJob;

#[async_trait::async_trait]
impl Job for RobotsTxtGenerationJob {
    fn name(&self) -> &'static str {
        "robots_txt_generation"
    }

    fn description(&self) -> &'static str {
        "Generates robots.txt for search engine crawlers"
    }

    fn schedule(&self) -> &'static str {
        "0 0 * * * *"
    }

    fn run_on_startup(&self) -> bool {
        true
    }

    async fn execute(&self, ctx: &JobContext) -> anyhow::Result<JobResult> {
        let start = std::time::Instant::now();

        tracing::info!("robots.txt generation started");

        let paths = ctx
            .app_paths::<Arc<AppPaths>>()
            .ok_or(MarketplaceError::Internal(
                "AppPaths not available in job context".to_string(),
            ))?
            .as_ref();
        generate_robots_txt(paths).await?;

        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

        tracing::info!(duration_ms, "robots.txt generation completed");

        Ok(JobResult::success().with_duration(duration_ms))
    }
}

systemprompt::traits::submit_job!(&RobotsTxtGenerationJob);

pub async fn generate_robots_txt(paths: &AppPaths) -> Result<(), MarketplaceError> {
    use systemprompt::models::Config;
    use tokio::fs;

    let global_config =
        Config::get().map_err(|e| MarketplaceError::Internal(format!("Config error: {e}")))?;

    let web_dir = paths.web().dist().to_path_buf();
    let base_url = &global_config.api_external_url;

    let robots_content = build_robots_txt_content(base_url)?;

    let robots_path = web_dir.join("robots.txt");
    fs::write(&robots_path, &robots_content).await?;

    tracing::info!(path = %robots_path.display(), "Generated robots.txt");

    Ok(())
}

fn build_robots_txt_content(base_url: &str) -> Result<String, MarketplaceError> {
    let mut content = String::new();

    writeln!(content, "User-agent: *")
        .map_err(|e| MarketplaceError::Internal(format!("fmt error: {e}")))?;
    writeln!(content, "Allow: /")
        .map_err(|e| MarketplaceError::Internal(format!("fmt error: {e}")))?;
    writeln!(content).map_err(|e| MarketplaceError::Internal(format!("fmt error: {e}")))?;
    writeln!(content, "Disallow: /api/")
        .map_err(|e| MarketplaceError::Internal(format!("fmt error: {e}")))?;
    writeln!(content, "Disallow: /console/")
        .map_err(|e| MarketplaceError::Internal(format!("fmt error: {e}")))?;
    writeln!(content, "Disallow: /_/")
        .map_err(|e| MarketplaceError::Internal(format!("fmt error: {e}")))?;
    writeln!(content).map_err(|e| MarketplaceError::Internal(format!("fmt error: {e}")))?;
    writeln!(content, "Sitemap: {base_url}/sitemap.xml")
        .map_err(|e| MarketplaceError::Internal(format!("fmt error: {e}")))?;

    Ok(content)
}
