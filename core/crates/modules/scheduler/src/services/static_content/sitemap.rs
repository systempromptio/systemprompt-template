use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use std::sync::Arc;
use systemprompt_core_database::{DatabaseProvider, DbPool};
use systemprompt_core_logging::LogService;
use systemprompt_core_system::AppContext;
use systemprompt_models::ContentConfig;
use tokio::fs;

const MAX_URLS_PER_SITEMAP: usize = 50_000;

#[derive(Debug, Clone)]
struct SitemapUrl {
    loc: String,
    lastmod: String,
    changefreq: String,
    priority: f32,
}

pub async fn generate_sitemap(
    db_pool: DbPool,
    logger: LogService,
    _app_context: Arc<AppContext>,
) -> Result<()> {
    println!("\n🗺️  Generating sitemap...\n");
    logger
        .info("content", "Starting sitemap generation")
        .await
        .ok();

    let config_path = std::env::var("CONTENT_CONFIG_PATH")
        .unwrap_or_else(|_| "crates/services/content/config.yml".to_string());

    let config = ContentConfig::load_async(&config_path)
        .await
        .context("Failed to load content config")?;

    let web_dir = std::env::var("WEB_DIR").unwrap_or_else(|_| "/app/core/web/dist".to_string());

    let base_url = std::env::var("SITEMAP_BASE_URL")
        .or_else(|_| std::env::var("API_EXTERNAL_URL"))
        .unwrap_or_else(|_| "https://tyingshoelaces.com".to_string());

    println!("📍 Using base URL: {}\n", base_url);
    logger
        .debug("content", &format!("Using base URL: {base_url}"))
        .await
        .ok();

    let mut all_urls = Vec::new();

    for (source_name, source) in config.content_sources.iter() {
        if !source.enabled {
            continue;
        }

        let Some(sitemap_config) = &source.sitemap else {
            continue;
        };

        if !sitemap_config.enabled {
            continue;
        }

        logger
            .debug("content", &format!("Processing source: {source_name}"))
            .await
            .ok();

        let urls = fetch_urls_from_database(
            &db_pool,
            &source.source_id,
            &sitemap_config.url_pattern,
            sitemap_config.priority,
            &sitemap_config.changefreq,
            &base_url,
        )
        .await
        .context(format!("Failed to fetch URLs for {source_name}"))?;

        all_urls.extend(urls);

        // Add parent route to sitemap if configured
        if let Some(parent_config) = &sitemap_config.parent_route {
            if parent_config.enabled {
                all_urls.push(SitemapUrl {
                    loc: format!("{}{}", base_url, parent_config.url),
                    lastmod: Utc::now().format("%Y-%m-%d").to_string(),
                    changefreq: parent_config.changefreq.clone(),
                    priority: parent_config.priority,
                });
            }
        }
    }

    let sitemap_chunks: Vec<Vec<_>> = all_urls
        .chunks(MAX_URLS_PER_SITEMAP)
        .map(|chunk| chunk.to_vec())
        .collect();

    // For single sitemap, write directly to dist/sitemap.xml
    // For multiple sitemaps, use sitemaps/ subdirectory with index
    if sitemap_chunks.len() == 1 {
        let sitemap_xml = build_sitemap_xml(&sitemap_chunks[0])?;
        let path = format!("{}/sitemap.xml", web_dir);
        fs::write(&path, sitemap_xml).await?;

        logger
            .debug(
                "content",
                &format!("Generated sitemap.xml: {} URLs", sitemap_chunks[0].len()),
            )
            .await
            .ok();
    } else {
        let sitemap_dir = format!("{}/sitemaps", web_dir);
        fs::create_dir_all(&sitemap_dir).await?;

        for (idx, chunk) in sitemap_chunks.iter().enumerate() {
            let filename = format!("sitemap-{}.xml", idx + 1);
            let sitemap_xml = build_sitemap_xml(chunk)?;
            let path = format!("{sitemap_dir}/{filename}");
            fs::write(&path, sitemap_xml).await?;

            logger
                .debug(
                    "content",
                    &format!("Generated {}: {} URLs", filename, chunk.len()),
                )
                .await
                .ok();
        }

        let index_xml = build_sitemap_index(&sitemap_chunks, &base_url)?;
        let path = format!("{}/sitemap.xml", web_dir);
        fs::write(&path, index_xml).await?;
        logger
            .debug(
                "content",
                &format!(
                    "Generated sitemap index with {} files",
                    sitemap_chunks.len()
                ),
            )
            .await
            .ok();
    }

    println!("✅ Sitemap generated: {} URLs total\n", all_urls.len());
    logger
        .info(
            "content",
            &format!("Sitemap generation completed: {} URLs", all_urls.len()),
        )
        .await
        .ok();

    Ok(())
}

async fn fetch_urls_from_database(
    db_pool: &DbPool,
    source_id: &str,
    url_pattern: &str,
    priority: f32,
    changefreq: &str,
    base_url: &str,
) -> Result<Vec<SitemapUrl>> {
    let query = r#"
        SELECT slug, updated_at, published_at
        FROM markdown_content
        WHERE source_id = $1
        ORDER BY published_at DESC
    "#;

    let rows = db_pool.fetch_all(&query, &[&source_id]).await?;

    let mut urls = Vec::new();
    for row in rows {
        let slug = row
            .get("slug")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing slug in database row"))?;

        // Use published_at for lastmod (when content was published), fallback to
        // updated_at
        let lastmod = row
            .get("published_at")
            .and_then(|v| v.as_str())
            .or_else(|| row.get("updated_at").and_then(|v| v.as_str()))
            .unwrap_or("");

        // Normalize to YYYY-MM-DD format for sitemap consistency
        let lastmod_normalized = normalize_date(lastmod);

        let relative_url = url_pattern.replace("{slug}", slug);
        let absolute_url = format!("{base_url}{relative_url}");
        urls.push(SitemapUrl {
            loc: absolute_url,
            lastmod: lastmod_normalized,
            changefreq: changefreq.to_string(),
            priority,
        });
    }

    Ok(urls)
}

fn normalize_date(date_str: &str) -> String {
    // If already in YYYY-MM-DD format, return as-is
    if date_str.len() == 10 && date_str.chars().nth(4) == Some('-') {
        return date_str.to_string();
    }
    // Extract YYYY-MM-DD from ISO timestamp (e.g.,
    // "2025-11-21T16:49:27.092196+00:00")
    if date_str.len() >= 10 {
        return date_str[..10].to_string();
    }
    date_str.to_string()
}

fn build_sitemap_xml(urls: &[SitemapUrl]) -> Result<String> {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
"#,
    );

    for url in urls {
        xml.push_str(&format!(
            r#"  <url>
    <loc>{}</loc>
    <lastmod>{}</lastmod>
    <changefreq>{}</changefreq>
    <priority>{:.1}</priority>
  </url>
"#,
            escape_xml(&url.loc),
            escape_xml(&url.lastmod),
            escape_xml(&url.changefreq),
            url.priority
        ));
    }

    xml.push_str("</urlset>");
    Ok(xml)
}

fn build_sitemap_index(chunks: &[Vec<SitemapUrl>], base_url: &str) -> Result<String> {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
"#,
    );

    for (idx, _chunk) in chunks.iter().enumerate() {
        let filename = format!("sitemap-{}.xml", idx + 1);

        xml.push_str(&format!(
            r#"  <sitemap>
    <loc>{}/sitemaps/{}</loc>
    <lastmod>{}</lastmod>
  </sitemap>
"#,
            base_url,
            filename,
            Utc::now().format("%Y-%m-%d")
        ));
    }

    xml.push_str("</sitemapindex>");
    Ok(xml)
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_xml_ampersand() {
        assert_eq!(escape_xml("a&b"), "a&amp;b");
    }

    #[test]
    fn test_escape_xml_tags() {
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
    }

    #[test]
    fn test_escape_xml_quotes() {
        assert_eq!(escape_xml("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_escape_xml_apostrophe() {
        assert_eq!(escape_xml("it's"), "it&apos;s");
    }

    #[test]
    fn test_build_sitemap_xml() {
        let urls = vec![SitemapUrl {
            loc: "https://example.com/blog/test".to_string(),
            lastmod: "2024-01-01".to_string(),
            changefreq: "weekly".to_string(),
            priority: 0.8,
        }];

        let xml = build_sitemap_xml(&urls).unwrap();
        assert!(xml.contains("<url>"));
        assert!(xml.contains("https://example.com/blog/test"));
        assert!(xml.contains("<changefreq>weekly</changefreq>"));
        assert!(xml.contains("<priority>0.8</priority>"));
        assert!(xml.contains("</urlset>"));
    }

    #[test]
    fn test_build_sitemap_xml_escaping() {
        let urls = vec![SitemapUrl {
            loc: "https://example.com/blog/test?a=1&b=2".to_string(),
            lastmod: "2024-01-01".to_string(),
            changefreq: "weekly".to_string(),
            priority: 0.8,
        }];

        let xml = build_sitemap_xml(&urls).unwrap();
        assert!(xml.contains("&amp;"));
        assert!(!xml.contains("?a=1&b=2"));
    }

    #[test]
    fn test_build_sitemap_index() {
        let chunk1 = vec![SitemapUrl {
            loc: "url1".to_string(),
            lastmod: "2024-01-01".to_string(),
            changefreq: "weekly".to_string(),
            priority: 0.8,
        }];

        let chunk2 = vec![SitemapUrl {
            loc: "url2".to_string(),
            lastmod: "2024-01-02".to_string(),
            changefreq: "weekly".to_string(),
            priority: 0.8,
        }];

        let base_url = "https://example.com";
        let index = build_sitemap_index(&vec![chunk1, chunk2], base_url).unwrap();
        assert!(index.contains("<sitemapindex"));
        assert!(index.contains("https://example.com/sitemaps/sitemap-1.xml"));
        assert!(index.contains("https://example.com/sitemaps/sitemap-2.xml"));
        assert!(index.contains("</sitemapindex>"));
    }
}
