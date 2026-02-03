---
title: "RSS & Sitemap Providers"
description: "Generate RSS feeds and sitemap entries for your content."
author: "SystemPrompt Team"
slug: "extensions/web-traits/rss-sitemap-provider"
keywords: "rss, sitemap, feeds, seo, extensions"
image: ""
kind: "reference"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# RSS & Sitemap Providers

RssFeedProvider and SitemapProvider generate RSS feeds and sitemap entries during the publish pipeline.

## RssFeedProvider

### The Trait

```rust
#[async_trait]
pub trait RssFeedProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;

    fn priority(&self) -> u32 {
        100
    }

    async fn provide_feeds(&self, ctx: &RssFeedContext<'_>) -> Result<Vec<RssFeedSpec>>;
}
```

### RssFeedContext

```rust
impl<'a> RssFeedContext<'a> {
    pub fn web_config(&self) -> &FullWebConfig;
    pub fn db_pool<T>(&self) -> Option<&T>;
}
```

### RssFeedSpec

```rust
pub struct RssFeedSpec {
    pub output_path: PathBuf,
    pub metadata: RssFeedMetadata,
    pub items: Vec<RssFeedItem>,
}

pub struct RssFeedMetadata {
    pub title: String,
    pub description: String,
    pub link: String,
    pub language: Option<String>,
}

pub struct RssFeedItem {
    pub title: String,
    pub link: String,
    pub description: Option<String>,
    pub pub_date: Option<String>,
    pub guid: Option<String>,
}
```

### Implementation

```rust
use systemprompt::extension::prelude::*;
use anyhow::Result;
use std::path::PathBuf;

pub struct BlogRssProvider;

#[async_trait]
impl RssFeedProvider for BlogRssProvider {
    fn provider_id(&self) -> &'static str {
        "blog-rss"
    }

    async fn provide_feeds(&self, ctx: &RssFeedContext<'_>) -> Result<Vec<RssFeedSpec>> {
        let pool = ctx.db_pool::<Arc<PgPool>>()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let posts = sqlx::query!(
            r#"SELECT slug, title, description, published_at
               FROM markdown_content
               WHERE source_id = 'blog' AND public = true
               ORDER BY published_at DESC
               LIMIT 20"#
        )
        .fetch_all(&*pool)
        .await?;

        let base_url = &ctx.web_config().base_url;

        let items: Vec<RssFeedItem> = posts.iter().map(|p| RssFeedItem {
            title: p.title.clone(),
            link: format!("{}/blog/{}", base_url, p.slug),
            description: p.description.clone(),
            pub_date: p.published_at.map(|d| d.to_rfc2822()),
            guid: Some(format!("{}/blog/{}", base_url, p.slug)),
        }).collect();

        Ok(vec![RssFeedSpec {
            output_path: PathBuf::from("feed.xml"),
            metadata: RssFeedMetadata {
                title: "Blog".to_string(),
                description: "Latest posts".to_string(),
                link: format!("{}/blog", base_url),
                language: Some("en".to_string()),
            },
            items,
        }])
    }
}
```

---

## SitemapProvider

### The Trait

```rust
#[async_trait]
pub trait SitemapProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;

    fn priority(&self) -> u32 {
        100
    }

    async fn provide_sitemap_sources(
        &self,
        ctx: &SitemapContext<'_>,
    ) -> Result<Vec<SitemapSourceSpec>>;
}
```

### SitemapContext

```rust
impl<'a> SitemapContext<'a> {
    pub fn web_config(&self) -> &FullWebConfig;
    pub fn db_pool<T>(&self) -> Option<&T>;
}
```

### SitemapSourceSpec

```rust
pub struct SitemapSourceSpec {
    pub source_id: String,
    pub entries: Vec<SitemapUrlEntry>,
}

pub struct SitemapUrlEntry {
    pub loc: String,
    pub lastmod: Option<String>,
    pub changefreq: Option<String>,
    pub priority: Option<f32>,
}
```

### Implementation

```rust
pub struct ContentSitemapProvider;

#[async_trait]
impl SitemapProvider for ContentSitemapProvider {
    fn provider_id(&self) -> &'static str {
        "content-sitemap"
    }

    async fn provide_sitemap_sources(
        &self,
        ctx: &SitemapContext<'_>,
    ) -> Result<Vec<SitemapSourceSpec>> {
        let pool = ctx.db_pool::<Arc<PgPool>>()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let content = sqlx::query!(
            r#"SELECT source_id, slug, updated_at
               FROM markdown_content
               WHERE public = true"#
        )
        .fetch_all(&*pool)
        .await?;

        let base_url = &ctx.web_config().base_url;

        let mut sources: HashMap<String, Vec<SitemapUrlEntry>> = HashMap::new();

        for item in content {
            let entry = SitemapUrlEntry {
                loc: format!("{}/{}", base_url, item.slug),
                lastmod: item.updated_at.map(|d| d.format("%Y-%m-%d").to_string()),
                changefreq: Some("weekly".to_string()),
                priority: Some(0.7),
            };

            sources
                .entry(item.source_id)
                .or_default()
                .push(entry);
        }

        Ok(sources.into_iter()
            .map(|(source_id, entries)| SitemapSourceSpec { source_id, entries })
            .collect())
    }
}
```

## Registration

```rust
impl Extension for WebExtension {
    fn rss_feed_providers(&self) -> Vec<Arc<dyn RssFeedProvider>> {
        vec![Arc::new(BlogRssProvider)]
    }

    fn sitemap_providers(&self) -> Vec<Arc<dyn SitemapProvider>> {
        vec![Arc::new(ContentSitemapProvider)]
    }
}
```

## Multiple Feeds

Return multiple feeds from a single provider:

```rust
async fn provide_feeds(&self, ctx: &RssFeedContext<'_>) -> Result<Vec<RssFeedSpec>> {
    Ok(vec![
        RssFeedSpec {
            output_path: PathBuf::from("blog/feed.xml"),
            // ...
        },
        RssFeedSpec {
            output_path: PathBuf::from("docs/feed.xml"),
            // ...
        },
    ])
}
```