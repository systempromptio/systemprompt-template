---
title: "Page Prerenderer"
description: "Generate static HTML pages at build time for list pages, index pages, and configured content."
author: "SystemPrompt Team"
slug: "extensions/web-traits/page-prerenderer"
keywords: "prerenderer, static, pages, build, extensions"
image: ""
kind: "reference"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Page Prerenderer

PagePrerenderer generates static HTML pages at build time. Use this for list pages, index pages, and other content that doesn't come from markdown files.

## When It Runs

PagePrerenderers run during the publish pipeline, after content is processed:

```
Content ingestion
     |
Content rendering
     |
=======================================
PagePrerenderer::prepare()  <- You are here
=======================================
     |
Template rendering
     |
HTML output
```

## The Trait

```rust
#[async_trait]
pub trait PagePrerenderer: Send + Sync {
    fn page_type(&self) -> &str;

    fn priority(&self) -> u32 {
        100
    }

    async fn prepare(&self, ctx: &PagePrepareContext<'_>) -> Result<Option<PageRenderSpec>>;
}
```

## PagePrepareContext

```rust
impl<'a> PagePrepareContext<'a> {
    pub fn web_config(&self) -> &FullWebConfig;
    pub fn config(&self) -> &ContentConfigRaw;
    pub fn db_pool<T>(&self) -> Option<&T>;
}
```

## PageRenderSpec

```rust
pub struct PageRenderSpec {
    pub template_name: String,
    pub template_data: Value,
    pub output_path: PathBuf,
}

impl PageRenderSpec {
    pub fn new(
        template_name: impl Into<String>,
        template_data: Value,
        output_path: impl Into<PathBuf>,
    ) -> Self;
}
```

## Basic Implementation

```rust
use systemprompt::template_provider::{PagePrepareContext, PagePrerenderer, PageRenderSpec};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;

pub struct BlogListPrerenderer;

#[async_trait]
impl PagePrerenderer for BlogListPrerenderer {
    fn page_type(&self) -> &str {
        "blog-list"
    }

    fn priority(&self) -> u32 {
        100
    }

    async fn prepare(&self, ctx: &PagePrepareContext<'_>) -> Result<Option<PageRenderSpec>> {
        let pool = ctx.db_pool::<Arc<PgPool>>()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let posts = sqlx::query!(
            r#"SELECT slug, title, description, published_at
               FROM markdown_content
               WHERE source_id = 'blog' AND public = true
               ORDER BY published_at DESC"#
        )
        .fetch_all(&*pool)
        .await?;

        let posts_html = self.render_post_cards(&posts);

        let template_data = json!({
            "TITLE": "Blog",
            "DESCRIPTION": "Latest posts from our blog",
            "POSTS": posts_html,
            "POST_COUNT": posts.len(),
        });

        Ok(Some(PageRenderSpec::new(
            "blog-list",
            template_data,
            PathBuf::from("blog/index.html"),
        )))
    }
}

impl BlogListPrerenderer {
    fn render_post_cards(&self, posts: &[Record]) -> String {
        posts.iter()
            .map(|p| format!(
                r#"<article class="post-card">
                    <a href="/blog/{}">
                        <h3>{}</h3>
                        <p>{}</p>
                    </a>
                </article>"#,
                p.slug, p.title, p.description.as_deref().unwrap_or("")
            ))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
```

## Returning None

Return `None` to skip rendering (e.g., when feature is disabled):

```rust
async fn prepare(&self, ctx: &PagePrepareContext<'_>) -> Result<Option<PageRenderSpec>> {
    if !self.config.blog_list_enabled {
        return Ok(None);
    }

    // ... render page
}
```

## Registration

```rust
impl Extension for WebExtension {
    fn page_prerenderers(&self) -> Vec<Arc<dyn PagePrerenderer>> {
        vec![
            Arc::new(HomepagePrerenderer::new(self.config.clone())),
            Arc::new(BlogListPrerenderer),
            Arc::new(DocsIndexPrerenderer),
            Arc::new(SitemapPrerenderer),
        ]
    }
}
```

## Common Patterns

### Homepage

```rust
async fn prepare(&self, ctx: &PagePrepareContext<'_>) -> Result<Option<PageRenderSpec>> {
    let pool = ctx.db_pool::<Arc<PgPool>>()?;

    let featured = self.fetch_featured_posts(pool).await?;
    let recent = self.fetch_recent_posts(pool, 5).await?;

    Ok(Some(PageRenderSpec::new(
        "homepage",
        json!({
            "FEATURED_POSTS": featured,
            "RECENT_POSTS": recent,
            "HERO_TITLE": ctx.web_config().hero.title,
        }),
        PathBuf::from("index.html"),
    )))
}
```

### Documentation Index

```rust
async fn prepare(&self, ctx: &PagePrepareContext<'_>) -> Result<Option<PageRenderSpec>> {
    let pool = ctx.db_pool::<Arc<PgPool>>()?;

    let sections = self.fetch_doc_sections(pool).await?;

    Ok(Some(PageRenderSpec::new(
        "docs-index",
        json!({
            "TITLE": "Documentation",
            "SECTIONS": sections,
        }),
        PathBuf::from("docs/index.html"),
    )))
}
```

### Multiple Pages

A single prerenderer can generate multiple pages by using a wrapper that calls it multiple times, or by returning specs for each variant.

## Priority

Lower priority values run first. Use this to ensure dependencies are generated before dependent pages:

```rust
fn priority(&self) -> u32 {
    50  // Runs before default (100)
}
```