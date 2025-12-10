use anyhow::{Context, Result};
use std::path::PathBuf;
use systemprompt_models::{ContentConfig, SitemapConfig};
use tokio::fs;

use super::cards::{generate_blog_card, CardData};
use super::templates::{generate_footer_html, TemplateEngine};

pub async fn generate_parent_index(
    source_name: &str,
    sitemap_config: &SitemapConfig,
    items: &[serde_json::Value],
    config: &ContentConfig,
    web_config: &serde_yaml::Value,
    templates: &TemplateEngine,
    dist_dir: &PathBuf,
) -> Result<bool> {
    let parent_config = match &sitemap_config.parent_route {
        Some(c) if c.enabled => c,
        _ => return Ok(false),
    };

    let template_name = match source_name {
        "blog" => "blog-list",
        "papers" => "paper-list",
        _ => "page-list",
    };

    let posts_html = build_posts_html(items, &parent_config.url);
    let parent_data = build_parent_template_data(
        &posts_html,
        config,
        web_config,
        source_name,
    );

    let parent_html = templates
        .render(template_name, &parent_data)
        .context("Failed to render parent route")?;

    let parent_dir = dist_dir.join(parent_config.url.trim_start_matches('/'));
    fs::create_dir_all(&parent_dir).await?;
    fs::write(parent_dir.join("index.html"), &parent_html).await?;

    println!("   [OK] Generated parent route: {}", parent_config.url);
    Ok(true)
}

fn build_posts_html(items: &[serde_json::Value], url_prefix: &str) -> Vec<String> {
    items
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
                url_prefix,
            })
        })
        .collect()
}

fn build_parent_template_data(
    posts_html: &[String],
    config: &ContentConfig,
    web_config: &serde_yaml::Value,
    source_name: &str,
) -> serde_json::Value {
    let footer_html = generate_footer_html(web_config);

    let org = &config.metadata.structured_data.organization;
    let org_name = &org.name;
    let org_url = &org.url;
    let org_logo = &org.logo;

    let source_config = config.content_sources.get(source_name);
    let source_branding = source_config.and_then(|s| s.branding.as_ref());

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

    serde_json::json!({
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
    })
}
