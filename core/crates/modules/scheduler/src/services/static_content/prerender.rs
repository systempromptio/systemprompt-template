use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use systemprompt_core_blog::ContentRepository;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::AppContext;
use systemprompt_models::ContentConfig;
use tokio::fs;

use super::cards::{generate_blog_card, CardData};
use super::markdown::render_markdown;
use super::templates::{
    generate_footer_html, get_templates_path, load_web_config, prepare_template_data,
    TemplateEngine,
};

pub async fn prerender_content(
    db_pool: DbPool,
    logger: LogService,
    _app_context: Arc<AppContext>,
) -> Result<()> {
    println!("\n📄 Prerendering static content...\n");
    logger
        .info("content", "Starting static content prerendering")
        .await
        .ok();

    let web_dir = std::env::var("WEB_DIR").unwrap_or_else(|_| "/app/core/web/dist".to_string());

    let config_path = std::env::var("CONTENT_CONFIG_PATH")
        .unwrap_or_else(|_| "crates/services/content/config.yml".to_string());

    let config = ContentConfig::load_async(&config_path)
        .await
        .context("Failed to load content config")?;

    let web_config = load_web_config()
        .await
        .context("Failed to load web config")?;

    logger
        .debug("content", &format!("Loaded config: {config_path}"))
        .await
        .ok();

    let template_dir = get_templates_path(&web_config);
    let templates = TemplateEngine::new(&template_dir)
        .await
        .context("Failed to load templates")?;

    let dist_dir = PathBuf::from(&web_dir);

    let mut total_rendered = 0;

    for (source_name, source) in config.content_sources.iter() {
        if !source.enabled {
            println!("⏭️  Skipping disabled source: {}", source_name);
            continue;
        }

        let Some(sitemap_config) = &source.sitemap else {
            println!(
                "⏭️  Skipping source without sitemap config: {}",
                source_name
            );
            continue;
        };

        if !sitemap_config.enabled {
            println!("⏭️  Skipping source with disabled sitemap: {}", source_name);
            continue;
        }

        println!("\n📂 Processing source: {}", source_name);
        logger
            .debug("content", &format!("Processing source: {source_name}"))
            .await
            .ok();

        // Fetch content from database with retry logic (Docker race condition fix)
        // Note: Some sources (like 'skills') are not stored in markdown_content table,
        // they're stored in specialized tables (agent_skills). These will return empty
        // and should be skipped, not retried.
        let content_repo = ContentRepository::new(db_pool.clone());
        let mut retries = 0;
        let max_retries = 5;

        let contents = loop {
            match content_repo.list_by_source(&source.source_id).await {
                Ok(contents) if !contents.is_empty() => {
                    if retries > 0 {
                        println!("   ✅ Content available after {} retries", retries);
                    }
                    break contents;
                },
                Ok(_empty_contents) if retries < max_retries => {
                    // Only retry for sources that should have content
                    // Skills are stored in agent_skills table, not markdown_content
                    if source_name.contains("skill") {
                        println!(
                            "   ⏭️  Skipping {} (not in markdown_content table)",
                            source_name
                        );
                        logger
                            .info(
                                "content",
                                &format!("Skipping {} - uses specialized table", source_name),
                            )
                            .await
                            .ok();
                        break _empty_contents;
                    }

                    println!(
                        "   ⏳ Content not yet available, retrying... (attempt {}/{})",
                        retries + 1,
                        max_retries
                    );
                    logger
                        .warn(
                            "content",
                            &format!("Content not yet available for {}, retrying...", source_name),
                        )
                        .await
                        .ok();
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    retries += 1;
                    continue;
                },
                Ok(_empty_contents) => {
                    // Check if this is expected to be empty (like skills)
                    if source_name.contains("skill") {
                        println!(
                            "   ⏭️  Skipping {} (not in markdown_content table)",
                            source_name
                        );
                        logger
                            .info(
                                "content",
                                &format!("Skipping {} - uses specialized table", source_name),
                            )
                            .await
                            .ok();
                    } else {
                        logger
                            .error(
                                "content",
                                &format!(
                                    "No content found for {} after {} retries",
                                    source_name, max_retries
                                ),
                            )
                            .await
                            .ok();
                        println!("   ❌ No content found after {} retries", max_retries);
                    }
                    break _empty_contents;
                },
                Err(e) => {
                    logger
                        .warn(
                            "content",
                            &format!("Failed to fetch {source_name}: {e}"),
                        )
                        .await
                        .ok();
                    println!("   ⚠️  Failed to fetch content: {}", e);
                    break Vec::new();
                },
            }
        };

        if contents.is_empty() {
            continue;
        }

        // Convert Content models to JSON for template rendering
        let items: Vec<serde_json::Value> = contents
            .iter()
            .map(|c| {
                serde_json::json!({
                    "id": c.id,
                    "slug": c.slug,
                    "title": c.title,
                    "description": c.description,
                    "content": c.body,
                    "author": c.author,
                    "published_at": c.published_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                    "updated_at": c.updated_at.map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
                    "keywords": c.keywords,
                    "content_type": c.kind,
                    "image": c.image,
                    "category_id": c.category_id,
                    "source_id": c.source_id,
                    "links": c.links,
                })
            })
            .collect();

        println!("   Found {} items", items.len());

        if items.is_empty() {
            continue;
        }

        let popular_ids = if source_name == "blog" {
            content_repo
                .get_popular_content_ids(&source.source_id, 30, 20)
                .await
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        for item in &items {
            let markdown_content = item.get("content").and_then(|v| v.as_str()).unwrap_or("");

            let content_html =
                render_markdown(markdown_content).context("Failed to render markdown")?;

            let config_value = serde_yaml::to_value(&config)?;
            let template_data = prepare_template_data(
                &item,
                &items,
                &popular_ids,
                &config_value,
                &web_config,
                &content_html,
                db_pool.clone(),
            )
            .await
            .context("Failed to prepare template data")?;

            let template_name = match source_name.as_str() {
                "blog" => "blog-post",
                "papers" => "paper",
                _ => "page",
            };

            let html = templates
                .render(template_name, &template_data)
                .context("Failed to render template")?;

            let slug = item
                .get("slug")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            let output_dir = determine_output_dir(&dist_dir, &sitemap_config.url_pattern, slug);
            fs::create_dir_all(&output_dir).await?;

            let output_path = output_dir.join("index.html");
            fs::write(&output_path, &html).await?;

            println!(
                "   ✅ Generated: {}",
                sitemap_config.url_pattern.replace("{slug}", slug)
            );
            total_rendered += 1;
        }

        // Generate parent route index page if configured
        if let Some(parent_config) = &sitemap_config.parent_route {
            if parent_config.enabled {
                let template_name = match source_name.as_str() {
                    "blog" => "blog-list",
                    "papers" => "paper-list",
                    _ => "page-list",
                };

                let posts_html: Vec<String> = items
                    .iter()
                    .map(|item| {
                        let title = item
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Untitled");
                        let slug = item.get("slug").and_then(|v| v.as_str()).unwrap_or("");
                        let description = item
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let image = item.get("image").and_then(|v| v.as_str());
                        let date = item
                            .get("published_at")
                            .and_then(|v| v.as_str())
                            .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
                            .map(|dt| dt.format("%B %d, %Y").to_string())
                            .unwrap_or_default();

                        generate_blog_card(&CardData {
                            title,
                            slug,
                            description,
                            image,
                            date: &date,
                            url_prefix: &parent_config.url,
                        })
                    })
                    .collect();

                let footer_html = generate_footer_html(&web_config);

                let org = &config.metadata.structured_data.organization;
                let org_name = &org.name;
                let org_url = &org.url;
                let org_logo = &org.logo;

                let source_branding = source.branding.as_ref();
                let blog_name = source_branding
                    .and_then(|b| b.name.as_deref())
                    .or_else(|| {
                        web_config
                            .get("branding")
                            .and_then(|b| b.get("name"))
                            .and_then(|v| v.as_str())
                    })
                    .unwrap_or("Blog");
                let blog_description = source_branding
                    .and_then(|b| b.description.as_deref())
                    .or_else(|| {
                        web_config
                            .get("branding")
                            .and_then(|b| b.get("description"))
                            .and_then(|v| v.as_str())
                    })
                    .unwrap_or("");
                let blog_image = source_branding
                    .and_then(|b| b.image.as_deref())
                    .map(|img| format!("{org_url}{img}"))
                    .unwrap_or_default();
                let blog_keywords = source_branding
                    .and_then(|b| b.keywords.as_deref())
                    .unwrap_or("");
                let blog_url = format!("{}/blog", org_url);
                let blog_language = &config.metadata.language;

                let twitter_handle = web_config
                    .get("branding")
                    .and_then(|b| b.get("twitter_handle"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let display_sitename = web_config
                    .get("branding")
                    .and_then(|b| b.get("display_sitename"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                let parent_data = serde_json::json!({
                    "POSTS": posts_html.join("\n"),
                    "ITEMS": posts_html.join("\n"),
                    "FOOTER_NAV": footer_html,
                    "ORG_NAME": org_name,
                    "ORG_URL": org_url,
                    "ORG_LOGO": org_logo,
                    "BLOG_NAME": blog_name,
                    "BLOG_DESCRIPTION": blog_description,
                    "BLOG_IMAGE": blog_image,
                    "BLOG_KEYWORDS": blog_keywords,
                    "BLOG_URL": blog_url,
                    "BLOG_LANGUAGE": blog_language,
                    "TWITTER_HANDLE": twitter_handle,
                    "HEADER_CTA_URL": "/",
                    "DISPLAY_SITENAME": display_sitename,
                });

                let parent_html = templates
                    .render(template_name, &parent_data)
                    .context("Failed to render parent route")?;

                let parent_dir = dist_dir.join(parent_config.url.trim_start_matches('/'));
                fs::create_dir_all(&parent_dir).await?;
                fs::write(parent_dir.join("index.html"), &parent_html).await?;

                println!("   ✅ Generated parent route: {}", parent_config.url);
                total_rendered += 1;
            }
        }
    }

    println!("\n✅ Pre-rendered {} items\n", total_rendered);
    logger
        .info(
            "content",
            &format!("Prerendering completed: {} items rendered", total_rendered),
        )
        .await
        .ok();

    Ok(())
}

fn determine_output_dir(dist_dir: &PathBuf, url_pattern: &str, slug: &str) -> PathBuf {
    let path = url_pattern.replace("{slug}", slug);
    let path = path.trim_start_matches('/');
    dist_dir.join(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_output_dir() {
        let dist = PathBuf::from("/app/dist");
        let pattern = "/blog/{slug}";
        let result = determine_output_dir(&dist, pattern, "hello-world");
        assert_eq!(result, PathBuf::from("/app/dist/blog/hello-world"));
    }

    #[test]
    fn test_determine_output_dir_trailing_slash() {
        let dist = PathBuf::from("/app/dist");
        let pattern = "/blog/{slug}/";
        let result = determine_output_dir(&dist, pattern, "hello-world");
        assert_eq!(result, PathBuf::from("/app/dist/blog/hello-world/"));
    }

    #[test]
    fn test_determine_output_dir_root() {
        let dist = PathBuf::from("/app/dist");
        let pattern = "/{slug}";
        let result = determine_output_dir(&dist, pattern, "hello-world");
        assert_eq!(result, PathBuf::from("/app/dist/hello-world"));
    }
}
