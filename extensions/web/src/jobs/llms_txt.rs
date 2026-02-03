use anyhow::{anyhow, Context, Result};
use std::fmt::Write as FmtWrite;
use systemprompt::database::DbPool;
use systemprompt::generator::ContentConfigRaw;
use systemprompt::traits::{Job, JobContext, JobResult};

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

    async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
        let start = std::time::Instant::now();

        tracing::info!("llms.txt generation started");

        let db_pool = ctx
            .db_pool::<DbPool>()
            .ok_or_else(|| anyhow::anyhow!("Database not available in job context"))?;

        generate_llms_txt(db_pool.clone()).await?;

        #[allow(clippy::cast_possible_truncation)]
        let duration_ms = start.elapsed().as_millis() as u64;

        tracing::info!(duration_ms, "llms.txt generation completed");

        Ok(JobResult::success().with_duration(duration_ms))
    }
}

systemprompt::traits::submit_job!(&LlmsTxtGenerationJob);

pub async fn generate_llms_txt(db_pool: DbPool) -> Result<()> {
    use systemprompt::generator::ContentConfigRaw;
    use systemprompt::models::{AppPaths, Config};
    use tokio::fs;

    let global_config = Config::get()?;
    let paths = AppPaths::get().map_err(|e| anyhow!("{e}"))?;

    let config_path = paths.system().content_config();
    let yaml_content = fs::read_to_string(&config_path)
        .await
        .context("Failed to read content config")?;

    let content_config: ContentConfigRaw =
        serde_yaml::from_str(&yaml_content).context("Failed to parse content config")?;

    let web_dir = paths.web().dist().to_path_buf();
    let base_url = &global_config.api_external_url;

    let llms_content = build_llms_txt_content(db_pool, &content_config, base_url).await?;

    let llms_path = web_dir.join("llms.txt");
    fs::write(&llms_path, &llms_content).await?;

    tracing::info!(path = %llms_path.display(), "Generated llms.txt");

    Ok(())
}

#[allow(clippy::too_many_lines)]
async fn build_llms_txt_content(
    db_pool: DbPool,
    config: &ContentConfigRaw,
    base_url: &str,
) -> Result<String> {
    use systemprompt::content::ContentRepository;
    use systemprompt::identifiers::SourceId;

    let mut content = String::new();

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
    writeln!(content)?;

    let repo = ContentRepository::new(&db_pool).map_err(|e| anyhow!("{e}"))?;

    writeln!(content, "## Playbooks")?;
    writeln!(content)?;
    writeln!(content, "Operational guides and procedures.")?;
    writeln!(content)?;

    if let Some(source) = config.content_sources.get("playbooks") {
        if source.enabled {
            let source_id = SourceId::new(&source.source_id);
            if let Ok(playbooks) = repo.list_by_source(&source_id).await {
                let mut guides: Vec<_> = playbooks
                    .iter()
                    .filter(|p| p.slug.starts_with("guide"))
                    .collect();
                let mut cli: Vec<_> = playbooks
                    .iter()
                    .filter(|p| p.slug.starts_with("cli"))
                    .collect();
                let mut build: Vec<_> = playbooks
                    .iter()
                    .filter(|p| p.slug.starts_with("build"))
                    .collect();
                let mut config_pb: Vec<_> = playbooks
                    .iter()
                    .filter(|p| p.slug.starts_with("config"))
                    .collect();
                let mut domain: Vec<_> = playbooks
                    .iter()
                    .filter(|p| p.slug.starts_with("domain"))
                    .collect();
                let mut content_pb: Vec<_> = playbooks
                    .iter()
                    .filter(|p| p.slug.starts_with("content"))
                    .collect();

                guides.sort_by(|a, b| a.title.cmp(&b.title));
                cli.sort_by(|a, b| a.title.cmp(&b.title));
                build.sort_by(|a, b| a.title.cmp(&b.title));
                config_pb.sort_by(|a, b| a.title.cmp(&b.title));
                domain.sort_by(|a, b| a.title.cmp(&b.title));
                content_pb.sort_by(|a, b| a.title.cmp(&b.title));

                if !guides.is_empty() {
                    writeln!(content, "### Getting Started (Start Here)")?;
                    writeln!(content)?;
                    for playbook in &guides {
                        let url = format!("{}/playbooks/{}", base_url, playbook.slug);
                        writeln!(
                            content,
                            "- [{}]({}): {}",
                            playbook.title, url, playbook.description
                        )?;
                    }
                    writeln!(content)?;
                }

                if !cli.is_empty() {
                    writeln!(content, "### CLI Operations")?;
                    writeln!(content)?;
                    for playbook in &cli {
                        let url = format!("{}/playbooks/{}", base_url, playbook.slug);
                        writeln!(
                            content,
                            "- [{}]({}): {}",
                            playbook.title, url, playbook.description
                        )?;
                    }
                    writeln!(content)?;
                }

                if !build.is_empty() {
                    writeln!(content, "### Build & Development")?;
                    writeln!(content)?;
                    for playbook in &build {
                        let url = format!("{}/playbooks/{}", base_url, playbook.slug);
                        writeln!(
                            content,
                            "- [{}]({}): {}",
                            playbook.title, url, playbook.description
                        )?;
                    }
                    writeln!(content)?;
                }

                if !config_pb.is_empty() {
                    writeln!(content, "### Configuration")?;
                    writeln!(content)?;
                    for playbook in &config_pb {
                        let url = format!("{}/playbooks/{}", base_url, playbook.slug);
                        writeln!(
                            content,
                            "- [{}]({}): {}",
                            playbook.title, url, playbook.description
                        )?;
                    }
                    writeln!(content)?;
                }

                if !domain.is_empty() {
                    writeln!(content, "### Domain Operations")?;
                    writeln!(content)?;
                    for playbook in &domain {
                        let url = format!("{}/playbooks/{}", base_url, playbook.slug);
                        writeln!(
                            content,
                            "- [{}]({}): {}",
                            playbook.title, url, playbook.description
                        )?;
                    }
                    writeln!(content)?;
                }

                if !content_pb.is_empty() {
                    writeln!(content, "### Content Creation")?;
                    writeln!(content)?;
                    for playbook in &content_pb {
                        let url = format!("{}/playbooks/{}", base_url, playbook.slug);
                        writeln!(
                            content,
                            "- [{}]({}): {}",
                            playbook.title, url, playbook.description
                        )?;
                    }
                    writeln!(content)?;
                }
            }
        }
    }

    writeln!(content, "## Documentation")?;
    writeln!(content)?;
    writeln!(content, "Technical documentation and guides.")?;
    writeln!(content)?;

    if let Some(source) = config.content_sources.get("documentation") {
        if source.enabled {
            let source_id = SourceId::new(&source.source_id);
            if let Ok(docs) = repo.list_by_source(&source_id).await {
                let mut services: Vec<_> = docs
                    .iter()
                    .filter(|d| d.slug.starts_with("services"))
                    .collect();
                let mut extensions: Vec<_> = docs
                    .iter()
                    .filter(|d| d.slug.starts_with("extensions"))
                    .collect();
                let mut config_docs: Vec<_> = docs
                    .iter()
                    .filter(|d| d.slug.starts_with("config"))
                    .collect();
                let mut other: Vec<_> = docs
                    .iter()
                    .filter(|d| {
                        !d.slug.starts_with("services")
                            && !d.slug.starts_with("extensions")
                            && !d.slug.starts_with("config")
                    })
                    .collect();

                services.sort_by(|a, b| a.title.cmp(&b.title));
                extensions.sort_by(|a, b| a.title.cmp(&b.title));
                config_docs.sort_by(|a, b| a.title.cmp(&b.title));
                other.sort_by(|a, b| a.title.cmp(&b.title));

                if !services.is_empty() {
                    writeln!(content, "### Services")?;
                    writeln!(content)?;
                    for doc in &services {
                        let url = format!("{}/documentation/{}", base_url, doc.slug);
                        writeln!(content, "- [{}]({}): {}", doc.title, url, doc.description)?;
                    }
                    writeln!(content)?;
                }

                if !extensions.is_empty() {
                    writeln!(content, "### Extensions")?;
                    writeln!(content)?;
                    for doc in &extensions {
                        let url = format!("{}/documentation/{}", base_url, doc.slug);
                        writeln!(content, "- [{}]({}): {}", doc.title, url, doc.description)?;
                    }
                    writeln!(content)?;
                }

                if !config_docs.is_empty() {
                    writeln!(content, "### Configuration Reference")?;
                    writeln!(content)?;
                    for doc in &config_docs {
                        let url = format!("{}/documentation/{}", base_url, doc.slug);
                        writeln!(content, "- [{}]({}): {}", doc.title, url, doc.description)?;
                    }
                    writeln!(content)?;
                }

                if !other.is_empty() {
                    writeln!(content, "### General")?;
                    writeln!(content)?;
                    for doc in &other {
                        let url = format!("{}/documentation/{}", base_url, doc.slug);
                        writeln!(content, "- [{}]({}): {}", doc.title, url, doc.description)?;
                    }
                    writeln!(content)?;
                }
            }
        }
    }

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
