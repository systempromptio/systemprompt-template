use std::fmt::Write as FmtWrite;
use std::sync::Arc;

use systemprompt::database::DbPool;
use systemprompt::generator::ContentConfigRaw;
use systemprompt::models::AppPaths;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::error::JobError;

#[derive(Debug, Clone, Copy, Default)]
pub struct LlmsTxtGenerationJob;

#[async_trait::async_trait]
impl Job for LlmsTxtGenerationJob {
    fn name(&self) -> &'static str {
        "llms_txt_generation"
    }

    fn description(&self) -> &'static str {
        "Generates llms.txt for AI/LLM crawlers"
    }

    fn schedule(&self) -> &'static str {
        "0 0 * * * *"
    }

    fn run_on_startup(&self) -> bool {
        false
    }

    async fn execute(&self, ctx: &JobContext) -> Result<JobResult, systemprompt::traits::ProviderError> {
        Ok(execute_inner(ctx).await?)
    }
}

async fn execute_inner(ctx: &JobContext) -> Result<JobResult, JobError> {
    let start = std::time::Instant::now();

    tracing::info!("llms.txt generation started");

    let db_pool = ctx
        .db_pool::<DbPool>()
        .ok_or(JobError::MissingContext("DbPool"))?;
    let paths = ctx
        .app_paths::<Arc<AppPaths>>()
        .ok_or(JobError::MissingContext("AppPaths"))?
        .as_ref();

    generate_llms_txt(DbPool::clone(db_pool), paths).await?;

    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

    tracing::info!(duration_ms, "llms.txt generation completed");

    Ok(JobResult::success().with_duration(duration_ms))
}

systemprompt::traits::submit_job!(&LlmsTxtGenerationJob);

pub async fn generate_llms_txt(db_pool: DbPool, paths: &AppPaths) -> Result<(), JobError> {
    use systemprompt::models::Config;
    use tokio::fs;

    let global_config =
        Config::get().map_err(|e| JobError::config(format!("Config error: {e}")))?;

    let config_path = paths.system().content_config();
    let yaml_content = fs::read_to_string(&config_path).await?;

    let content_config: ContentConfigRaw = serde_yaml::from_str(&yaml_content)?;

    let web_dir = paths.web().dist().to_path_buf();
    let base_url = &global_config.api_external_url;

    let llms_content = build_llms_txt_content(db_pool, &content_config, base_url).await?;

    let llms_path = web_dir.join("llms.txt");
    fs::write(&llms_path, &llms_content).await?;

    tracing::info!(path = %llms_path.display(), "Generated llms.txt");

    Ok(())
}

fn write_section<T: AsRef<str>>(
    content: &mut String,
    heading: &str,
    items: &[(T, T, T)],
) -> std::fmt::Result {
    if items.is_empty() {
        return Ok(());
    }
    writeln!(content, "### {heading}")?;
    writeln!(content)?;
    for (title, url, description) in items {
        writeln!(
            content,
            "- [{}]({}): {}",
            title.as_ref(),
            url.as_ref(),
            description.as_ref()
        )?;
    }
    writeln!(content)
}

fn sort_entries_in_place(items: &mut [(String, String, String)]) {
    items.sort_by(|a, b| a.0.cmp(&b.0));
}

async fn build_llms_txt_content(
    db_pool: DbPool,
    config: &ContentConfigRaw,
    base_url: &str,
) -> Result<String, JobError> {
    use systemprompt::content::ContentRepository;

    let mut content = String::new();

    write_header(&mut content, base_url)?;

    let repo = ContentRepository::new(&db_pool)
        .map_err(|e| JobError::other(format!("ContentRepository error: {e}")))?;

    write_documentation_section(&mut content, config, &repo, base_url).await?;
    write_blog_section(&mut content, config, &repo, base_url).await?;

    writeln!(content, "## Resources")?;
    writeln!(content)?;
    writeln!(
        content,
        "- [Sitemap]({base_url}/sitemap.xml): Complete URL index"
    )?;
    writeln!(
        content,
        "- [Documentation]({base_url}/documentation): All documentation"
    )?;

    Ok(content)
}

fn write_header(content: &mut String, base_url: &str) -> std::fmt::Result {
    writeln!(content, "# Your Project Name")?;
    writeln!(content)?;
    writeln!(
        content,
        "> Add your project description here. This file helps AI assistants understand your project."
    )?;
    writeln!(content)?;
    writeln!(content, "## Quick Links")?;
    writeln!(content)?;
    writeln!(content, "- Homepage: {base_url}")?;
    writeln!(content, "- Documentation: {base_url}/documentation")?;
    writeln!(content, "- Blog: {base_url}/blog")?;
    writeln!(content)
}

async fn write_documentation_section(
    content: &mut String,
    config: &ContentConfigRaw,
    repo: &systemprompt::content::ContentRepository,
    base_url: &str,
) -> Result<(), JobError> {
    use systemprompt::identifiers::SourceId;

    writeln!(content, "## Documentation")?;
    writeln!(content)?;
    writeln!(content, "Technical documentation and guides.")?;
    writeln!(content)?;

    if let Some(source) = config.content_sources.get("documentation") {
        if source.enabled {
            let source_id = SourceId::new(&source.source_id);
            if let Ok(docs) = repo.list_by_source(&source_id).await {
                let prefixes = [
                    ("services", "Services"),
                    ("extensions", "Extensions"),
                    ("config", "Configuration Reference"),
                ];
                for (prefix, heading) in &prefixes {
                    let mut filtered: Vec<_> = docs
                        .iter()
                        .filter(|d| d.slug.starts_with(prefix))
                        .map(|d| {
                            (
                                d.title.clone(),
                                format!("{}/documentation/{}", base_url, d.slug),
                                d.description.clone(),
                            )
                        })
                        .collect();
                    sort_entries_in_place(&mut filtered);
                    write_section(content, heading, &filtered)?;
                }
                let mut other: Vec<_> = docs
                    .iter()
                    .filter(|d| {
                        !d.slug.starts_with("services")
                            && !d.slug.starts_with("extensions")
                            && !d.slug.starts_with("config")
                    })
                    .map(|d| {
                        (
                            d.title.clone(),
                            format!("{}/documentation/{}", base_url, d.slug),
                            d.description.clone(),
                        )
                    })
                    .collect();
                sort_entries_in_place(&mut other);
                write_section(content, "General", &other)?;
            }
        }
    }
    Ok(())
}

async fn write_blog_section(
    content: &mut String,
    config: &ContentConfigRaw,
    repo: &systemprompt::content::ContentRepository,
    base_url: &str,
) -> Result<(), JobError> {
    use systemprompt::identifiers::SourceId;

    writeln!(content, "## Blog")?;
    writeln!(content)?;
    writeln!(content, "Articles and updates.")?;
    writeln!(content)?;

    if let Some(source) = config.content_sources.get("blog") {
        if source.enabled {
            let source_id = SourceId::new(&source.source_id);
            if let Ok(posts) = repo.list_by_source(&source_id).await {
                for post in posts.iter().take(15) {
                    let url = format!("{}/blog/{}", base_url, post.slug);
                    writeln!(content, "- [{}]({}): {}", post.title, url, post.description)?;
                }
            }
        }
    }
    writeln!(content)?;
    Ok(())
}
