use anyhow::{anyhow, Result};
use std::fmt::Write as FmtWrite;
use systemprompt::traits::{Job, JobContext, JobResult};

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

    async fn execute(&self, _ctx: &JobContext) -> Result<JobResult> {
        let start = std::time::Instant::now();

        tracing::info!("robots.txt generation started");

        generate_robots_txt().await?;

        #[allow(clippy::cast_possible_truncation)]
        let duration_ms = start.elapsed().as_millis() as u64;

        tracing::info!(duration_ms, "robots.txt generation completed");

        Ok(JobResult::success().with_duration(duration_ms))
    }
}

systemprompt::traits::submit_job!(&RobotsTxtGenerationJob);

pub async fn generate_robots_txt() -> Result<()> {
    use systemprompt::models::{AppPaths, Config};
    use tokio::fs;

    let global_config = Config::get()?;
    let paths = AppPaths::get().map_err(|e| anyhow!("{e}"))?;

    let web_dir = paths.web().dist().to_path_buf();
    let base_url = &global_config.api_external_url;

    let robots_content = build_robots_txt_content(base_url)?;

    let robots_path = web_dir.join("robots.txt");
    fs::write(&robots_path, &robots_content).await?;

    tracing::info!(path = %robots_path.display(), "Generated robots.txt");

    Ok(())
}

fn build_robots_txt_content(base_url: &str) -> Result<String> {
    let mut content = String::new();

    writeln!(content, "User-agent: *")?;
    writeln!(content, "Allow: /")?;
    writeln!(content)?;
    writeln!(content, "Disallow: /api/")?;
    writeln!(content, "Disallow: /console/")?;
    writeln!(content, "Disallow: /_/")?;
    writeln!(content)?;
    writeln!(content, "Sitemap: {base_url}/sitemap.xml")?;

    Ok(content)
}
