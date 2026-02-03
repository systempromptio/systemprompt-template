---
title: "Content Data Provider"
description: "Enrich content items with computed fields, related content, and database lookups."
author: "SystemPrompt Team"
slug: "extensions/web-traits/content-data-provider"
keywords: "content, data, provider, enrichment, extensions"
image: ""
kind: "reference"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Content Data Provider

ContentDataProvider enriches content items during `contents_to_json()`, before PageDataProviders run. Use this to add computed fields, related content, or database lookups.

## When It Runs

```
Database Query
     |
=======================================
ContentDataProvider::enrich_content()  <- You are here
=======================================
     |
PageDataProvider::provide_page_data()
     |
ComponentRenderer::render()
     |
Template rendering
```

## The Trait

```rust
#[async_trait]
pub trait ContentDataProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;

    fn applies_to_sources(&self) -> Vec<String>;

    fn priority(&self) -> u32 {
        100
    }

    async fn enrich_content(
        &self,
        ctx: &ContentDataContext<'_>,
        item: &mut Value,
    ) -> Result<()>;
}
```

## ContentDataContext

```rust
impl<'a> ContentDataContext<'a> {
    pub fn content_id(&self) -> &str;
    pub fn source_name(&self) -> &str;
    pub fn db_pool<T>(&self) -> Option<&T>;
}
```

## Basic Implementation

```rust
use systemprompt::extension::prelude::{ContentDataContext, ContentDataProvider};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct DocsContentDataProvider;

#[async_trait]
impl ContentDataProvider for DocsContentDataProvider {
    fn provider_id(&self) -> &'static str {
        "docs-content-enricher"
    }

    fn applies_to_sources(&self) -> Vec<String> {
        vec!["documentation".to_string()]
    }

    fn priority(&self) -> u32 {
        100
    }

    async fn enrich_content(
        &self,
        ctx: &ContentDataContext<'_>,
        item: &mut Value,
    ) -> Result<()> {
        let slug = item.get("slug").and_then(|v| v.as_str()).unwrap_or("");

        // Add computed field
        if let Some(obj) = item.as_object_mut() {
            obj.insert("reading_time".to_string(), json!(self.calculate_reading_time(item)));
        }

        // Add children for index pages
        if item.get("kind").and_then(|v| v.as_str()) == Some("docs-index") {
            if let Some(pool) = ctx.db_pool::<Arc<PgPool>>() {
                let children = self.fetch_children(pool, ctx.source_name(), slug).await?;
                if let Some(obj) = item.as_object_mut() {
                    obj.insert("children".to_string(), json!(children));
                }
            }
        }

        Ok(())
    }
}

impl DocsContentDataProvider {
    fn calculate_reading_time(&self, item: &Value) -> u32 {
        let body = item.get("body").and_then(|v| v.as_str()).unwrap_or("");
        let words = body.split_whitespace().count();
        ((words as f32) / 200.0).ceil() as u32
    }

    async fn fetch_children(
        &self,
        pool: &PgPool,
        source: &str,
        parent_slug: &str,
    ) -> Result<Vec<Value>> {
        let children = sqlx::query!(
            r#"SELECT slug, title, description
               FROM markdown_content
               WHERE source_id = $1 AND slug LIKE $2
               ORDER BY slug"#,
            source,
            format!("{}/%", parent_slug)
        )
        .fetch_all(pool)
        .await?;

        Ok(children.into_iter().map(|c| json!({
            "slug": c.slug,
            "title": c.title,
            "description": c.description,
        })).collect())
    }
}
```

## Targeting Sources

Use `applies_to_sources()` to run only for specific content sources:

```rust
fn applies_to_sources(&self) -> Vec<String> {
    vec!["documentation".to_string(), "playbooks".to_string()]
}
```

Return an empty vector to run for ALL sources.

## Registration

```rust
impl Extension for WebExtension {
    fn content_data_providers(&self) -> Vec<Arc<dyn ContentDataProvider>> {
        vec![
            Arc::new(DocsContentDataProvider),
            Arc::new(RelatedPostsProvider::new(self.pool.clone())),
        ]
    }
}
```

## Common Patterns

### Related Content

```rust
async fn enrich_content(&self, ctx: &ContentDataContext<'_>, item: &mut Value) -> Result<()> {
    let tags = item.get("tags").and_then(|v| v.as_array());
    if let (Some(tags), Some(pool)) = (tags, ctx.db_pool::<Arc<PgPool>>()) {
        let related = self.find_by_tags(pool, tags).await?;
        if let Some(obj) = item.as_object_mut() {
            obj.insert("related_posts".to_string(), json!(related));
        }
    }
    Ok(())
}
```

### Breadcrumbs

```rust
async fn enrich_content(&self, ctx: &ContentDataContext<'_>, item: &mut Value) -> Result<()> {
    let slug = item.get("slug").and_then(|v| v.as_str()).unwrap_or("");
    let breadcrumbs = self.build_breadcrumbs(slug);
    if let Some(obj) = item.as_object_mut() {
        obj.insert("breadcrumbs".to_string(), json!(breadcrumbs));
    }
    Ok(())
}
```