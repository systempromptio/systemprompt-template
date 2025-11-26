use anyhow::{anyhow, Result};
use handlebars::Handlebars;
use serde_json::{json, Value};
use std::path::Path;
use systemprompt_core_blog::services::LinkGenerationService;
use systemprompt_core_blog::ContentRepository;
use systemprompt_core_database::DbPool;
use tokio::fs;

use super::cards::{generate_related_card, get_absolute_image_url, normalize_image_url, CardData};

fn calculate_read_time(html_content: &str) -> u32 {
    let text_count = html_content
        .replace(|c: char| c == '<' || c == '>', " ")
        .split_whitespace()
        .count();

    let minutes = (text_count as f32 / 200.0).ceil() as u32;
    minutes.max(1)
}

pub async fn load_web_config() -> Result<serde_yaml::Value> {
    let web_config_path = std::env::var("WEB_CONFIG_PATH")
        .unwrap_or_else(|_| "crates/services/web/config.yml".to_string());

    let content = fs::read_to_string(&web_config_path)
        .await
        .map_err(|e| anyhow!("Failed to read web config: {}", e))?;

    serde_yaml::from_str(&content).map_err(|e| anyhow!("Failed to parse web config: {}", e))
}

pub fn generate_footer_html(web_config: &serde_yaml::Value) -> String {
    let navigation = web_config.get("navigation");
    let footer = navigation.and_then(|n| n.get("footer"));
    let social = navigation.and_then(|n| n.get("social"));

    let mut sections_html = Vec::new();

    if let Some(footer_config) = footer {
        if let Some(mapping) = footer_config.as_mapping() {
            for (section_name, links) in mapping {
                let section_title = section_name
                    .as_str()
                    .unwrap_or("Links")
                    .chars()
                    .next()
                    .map(|c| {
                        c.to_uppercase().collect::<String>()
                            + &section_name.as_str().unwrap_or("")[1..]
                    })
                    .unwrap_or_else(|| "Links".to_string());

                if let Some(links_seq) = links.as_sequence() {
                    let link_items: Vec<String> = links_seq
                        .iter()
                        .filter_map(|link| {
                            let path = link.get("path")?.as_str()?;
                            let label = link.get("label")?.as_str()?;
                            Some(format!(r#"<li><a href="{}">{}</a></li>"#, path, label))
                        })
                        .collect();

                    if !link_items.is_empty() {
                        sections_html.push(format!(
                            r#"<div class="footer-nav__section">
          <h4>{}</h4>
          <ul class="footer-nav__links">
            {}
          </ul>
        </div>"#,
                            section_title,
                            link_items.join("\n            ")
                        ));
                    }
                }
            }
        }
    }

    let mut social_html = String::new();
    if let Some(social_links) = social.and_then(|s| s.as_sequence()) {
        let social_items: Vec<String> = social_links
            .iter()
            .filter_map(|link| {
                let href = link.get("href")?.as_str()?;
                let label = link.get("label")?.as_str()?;
                let link_type = link.get("type")?.as_str()?;

                let icon = match link_type {
                    "github" => r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>"#,
                    "twitter" => r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/></svg>"#,
                    "email" => r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/></svg>"#,
                    "linkedin" => r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M19 0h-14c-2.761 0-5 2.239-5 5v14c0 2.761 2.239 5 5 5h14c2.762 0 5-2.239 5-5v-14c0-2.761-2.238-5-5-5zm-11 19h-3v-11h3v11zm-1.5-12.268c-.966 0-1.75-.79-1.75-1.764s.784-1.764 1.75-1.764 1.75.79 1.75 1.764-.783 1.764-1.75 1.764zm13.5 12.268h-3v-5.604c0-3.368-4-3.113-4 0v5.604h-3v-11h3v1.765c1.396-2.586 7-2.777 7 2.476v6.759z"/></svg>"#,
                    _ => "",
                };

                Some(format!(
                    r#"<a href="{}" target="_blank" rel="noopener noreferrer">{}{}</a>"#,
                    href, icon, label
                ))
            })
            .collect();

        if !social_items.is_empty() {
            social_html = format!(
                r#"<div class="footer-social">
        {}
      </div>"#,
                social_items.join("\n        ")
            );
        }
    }

    let nav_html = if sections_html.is_empty() {
        String::new()
    } else {
        format!(
            r#"<nav class="footer-nav">
        {}
      </nav>"#,
            sections_html.join("\n        ")
        )
    };

    format!(
        r#"{}

      {}

      <div class="footer-meta">
        <p>&copy; 2025 tyingshoelaces. Built with AI agents.</p>
      </div>"#,
        nav_html, social_html
    )
}

#[derive(Debug)]
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

impl TemplateEngine {
    pub async fn new(template_dir: &str) -> Result<Self> {
        let mut handlebars = Handlebars::new();

        let path = Path::new(template_dir);
        if !path.exists() {
            return Err(anyhow!("Template directory not found: {}", template_dir));
        }

        let mut entries = fs::read_dir(path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "html") {
                let template_name = path.file_stem().unwrap().to_string_lossy().to_string();

                let content = fs::read_to_string(&path).await?;
                handlebars.register_template_string(&template_name, content)?;
            }
        }

        Ok(Self { handlebars })
    }

    pub fn render(&self, template_name: &str, data: &Value) -> Result<String> {
        self.handlebars
            .render(template_name, data)
            .map_err(|e| anyhow!("Template render failed: {}", e))
    }
}

pub async fn prepare_template_data(
    item: &Value,
    all_items: &[Value],
    config: &serde_yaml::Value,
    web_config: &serde_yaml::Value,
    content_html: &str,
    db_pool: DbPool,
) -> Result<Value> {
    let footer_html = generate_footer_html(web_config);
    let org = config
        .get("metadata")
        .and_then(|m| m.get("structured_data"))
        .and_then(|s| s.get("organization"))
        .cloned()
        .unwrap_or_else(|| serde_yaml::Value::Null);

    let article_config = config
        .get("metadata")
        .and_then(|m| m.get("structured_data"))
        .and_then(|s| s.get("article"))
        .cloned()
        .unwrap_or_else(|| serde_yaml::Value::Null);

    let item_category = item
        .get("category_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let item_slug = item.get("slug").and_then(|v| v.as_str()).unwrap_or("");

    let related: Vec<_> = all_items
        .iter()
        .filter(|other| {
            let other_slug = other.get("slug").and_then(|v| v.as_str()).unwrap_or("");
            let other_category = other
                .get("category_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            other_slug != item_slug && other_category == item_category
        })
        .take(5)
        .collect();

    let published_date = item
        .get("published_at")
        .or_else(|| item.get("date"))
        .or_else(|| item.get("created_at"))
        .map(|v| v.as_str().unwrap_or(""))
        .unwrap_or("");

    // Format date to human-readable format (e.g., "November 13, 2025")
    // Also create ISO 8601 format for structured data (e.g., "2025-11-13")
    let (formatted_date, date_iso) = if !published_date.is_empty() {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(published_date) {
            (
                dt.format("%B %d, %Y").to_string(),
                dt.format("%Y-%m-%d").to_string(),
            )
        } else {
            (published_date.to_string(), published_date.to_string())
        }
    } else {
        (String::new(), String::new())
    };

    let (formatted_modified, date_modified_iso) = item
        .get("updated_at")
        .and_then(|v| v.as_str())
        .map(|date_str| {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
                (
                    dt.format("%B %d, %Y").to_string(),
                    dt.format("%Y-%m-%d").to_string(),
                )
            } else {
                (date_str.to_string(), date_str.to_string())
            }
        })
        .unwrap_or_else(|| (formatted_date.clone(), date_iso.clone()));

    // Render related posts as styled cards with images and server-side link tracking
    // Generate tracked /r/SHORTCODE links that redirect to /blog/SLUG
    let related_html = if !related.is_empty() {
        let source_slug = item
            .get("slug")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let source_id = item.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");

        let link_gen = LinkGenerationService::new(db_pool.clone());
        let mut cards = Vec::new();

        for (index, rel_item) in related.iter().enumerate() {
            let title = rel_item
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled");
            let excerpt = rel_item
                .get("description")
                .or_else(|| rel_item.get("excerpt"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let slug = rel_item.get("slug").and_then(|v| v.as_str());
            let image = rel_item.get("image").and_then(|v| v.as_str());

            let rel_date = rel_item
                .get("published_at")
                .or_else(|| rel_item.get("date"))
                .or_else(|| rel_item.get("created_at"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let formatted_rel_date = if !rel_date.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(rel_date) {
                    dt.format("%b %d, %Y").to_string()
                } else {
                    rel_date.to_string()
                }
            } else {
                String::new()
            };

            if let Some(slug) = slug {
                let target_url = format!("/blog/{}", slug);
                let source_page = format!("/blog/{}", source_slug);
                let link_position = format!("related-{}", index + 1);

                let card_data = CardData {
                    title,
                    slug,
                    description: excerpt,
                    image,
                    date: &formatted_rel_date,
                    url_prefix: "/blog",
                };

                match link_gen
                    .generate_internal_content_link(
                        &target_url,
                        source_id,
                        &source_page,
                        Some(title.to_string()),
                        Some(link_position),
                    )
                    .await
                {
                    Ok(tracked_link) => {
                        let redirect_url = format!("/r/{}", tracked_link.short_code);
                        cards.push(generate_related_card(&card_data, &redirect_url));
                    },
                    Err(e) => {
                        eprintln!("Failed to generate tracked link for {}: {}", slug, e);
                        let fallback_url = format!("/blog/{}", slug);
                        cards.push(generate_related_card(&card_data, &fallback_url));
                    },
                }
            }
        }

        if cards.is_empty() {
            String::new()
        } else {
            format!(
                r#"<div class="related-articles">
  <h3>Continue Reading</h3>
  <div class="related-grid">{}</div>
</div>"#,
                cards.join("\n")
            )
        }
    } else {
        String::new()
    };

    let source_slug = item
        .get("slug")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let source_id = item.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");

    let link_gen = LinkGenerationService::new(db_pool.clone());

    // Generate header CTA link
    let header_cta_url = match link_gen
        .generate_internal_content_link(
            "/",
            source_id,
            &format!("/blog/{}", source_slug),
            Some("Header Chat CTA".to_string()),
            Some("header-cta".to_string()),
        )
        .await
    {
        Ok(tracked_link) => format!("/r/{}", tracked_link.short_code),
        Err(_) => "/".to_string(),
    };

    // Generate banner CTA link
    let banner_cta_url = match link_gen
        .generate_internal_content_link(
            "/",
            source_id,
            &format!("/blog/{}", source_slug),
            Some("Banner Chat CTA".to_string()),
            Some("banner-cta".to_string()),
        )
        .await
    {
        Ok(tracked_link) => format!("/r/{}", tracked_link.short_code),
        Err(_) => "/".to_string(),
    };

    let read_time = calculate_read_time(content_html);

    let org_name = org.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let org_url = org.get("url").and_then(|v| v.as_str()).unwrap_or("");
    let org_logo = org.get("logo").and_then(|v| v.as_str()).unwrap_or("");

    let article_type = article_config
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("Article");
    let article_section = article_config
        .get("article_section")
        .and_then(|v| v.as_str())
        .unwrap_or("General");
    let article_language = article_config
        .get("language")
        .and_then(|v| v.as_str())
        .unwrap_or("en-US");

    // Generate references HTML from links (card-based layout)
    let references_html = if let Some(links_array) = item.get("links").and_then(|v| v.as_array()) {
        if !links_array.is_empty() {
            let cards: Vec<String> = links_array
                .iter()
                .enumerate()
                .filter_map(|(index, link)| {
                    let title = link.get("title")?.as_str()?;
                    let url = link.get("url")?.as_str()?;
                    let domain = url::Url::parse(url)
                        .ok()
                        .and_then(|u| u.host_str().map(|h| h.to_string()))
                        .unwrap_or_default();

                    Some(format!(
                        r#"<article class="reference-card">
  <span class="reference-card__number">[{}]</span>
  <a href="{}" class="reference-card__title" target="_blank" rel="noopener noreferrer">{}</a>
  <span class="reference-card__domain">{}</span>
</article>"#,
                        index + 1,
                        url,
                        title,
                        domain
                    ))
                })
                .collect();

            if cards.is_empty() {
                String::new()
            } else {
                format!(
                    r#"<section class="references">
  <h2>References &amp; Sources</h2>
  <div class="references-grid">{}</div>
</section>"#,
                    cards.join("\n")
                )
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Fetch and generate social content HTML (child content of blog posts)
    let content_repo = ContentRepository::new(db_pool.clone());
    let social_html = if let Some(content_id) = item.get("id").and_then(|v| v.as_str()) {
        match content_repo.get_social_content_by_parent(content_id).await {
            Ok(social_posts) if !social_posts.is_empty() => {
                let items: Vec<String> = social_posts
                    .iter()
                    .map(|post| {
                        let platform = match post.kind.as_str() {
                            "social_linkedin" => "LinkedIn",
                            "social_twitter" => "Twitter/X",
                            "social_reddit" => "Reddit",
                            _ => "Social",
                        };
                        format!(
                            r#"    <li><a href="/{}">{} Version</a></li>"#,
                            post.slug, platform
                        )
                    })
                    .collect();

                format!(
                    r#"<section class="social-content">
  <h2>Related Social Content</h2>
  <p>This article is also available in social media formats:</p>
  <ul>
{}
  </ul>
</section>"#,
                    items.join("\n")
                )
            },
            _ => String::new(),
        }
    } else {
        String::new()
    };

    let raw_image = item
        .get("image")
        .or_else(|| item.get("cover_image"))
        .and_then(|v| v.as_str());
    let featured_image = normalize_image_url(raw_image).unwrap_or_default();
    let absolute_image_url = get_absolute_image_url(raw_image, org_url).unwrap_or_default();

    let default_author = config["metadata"]["default_author"]
        .as_str()
        .unwrap_or("tyingshoelaces");

    let author = item
        .get("author")
        .and_then(|v| v.as_str())
        .filter(|a| !a.is_empty() && !a.contains("local"))
        .unwrap_or(default_author);

    let twitter_handle = web_config
        .get("branding")
        .and_then(|b| b.get("twitter_handle"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    Ok(json!({
        "TITLE": item.get("title").unwrap_or(&json!("")),
        "DESCRIPTION": item.get("description")
            .or_else(|| item.get("excerpt"))
            .unwrap_or(&json!("")),
        "AUTHOR": author,
        "DATE": formatted_date,
        "DATE_PUBLISHED": formatted_date,
        "DATE_MODIFIED": formatted_modified,
        "DATE_ISO": date_iso,
        "DATE_MODIFIED_ISO": date_modified_iso,
        "READ_TIME": read_time,
        "KEYWORDS": item.get("keywords")
            .or_else(|| item.get("tags"))
            .unwrap_or(&json!("")),
        "IMAGE": absolute_image_url,
        "FEATURED_IMAGE": featured_image,
        "CONTENT": content_html,
        "SLUG": item.get("slug").unwrap_or(&json!("")),
        "ORG_NAME": org_name,
        "ORG_URL": org_url,
        "ORG_LOGO": org_logo,
        "TWITTER_HANDLE": twitter_handle,
        "ARTICLE_TYPE": article_type,
        "ARTICLE_SECTION": article_section,
        "ARTICLE_LANGUAGE": article_language,
        "HEADER_CTA_URL": header_cta_url,
        "BANNER_CTA_URL": banner_cta_url,
        "RELATED_CONTENT": related_html,
        "REFERENCES": references_html,
        "SOCIAL_CONTENT": social_html,
        "FOOTER_NAV": footer_html,
        "DISPLAY_SITENAME": web_config.get("branding")
            .and_then(|b| b.get("display_sitename"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_template_engine_nonexistent_dir() {
        let result = TemplateEngine::new("/nonexistent/dir").await;
        assert!(result.is_err());
    }
}
