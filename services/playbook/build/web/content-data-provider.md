---
title: "Creating Content Data Providers"
description: "Step-by-step guide to creating ContentDataProvider implementations for content enrichment."
author: "SystemPrompt Team"
slug: "build-content-data-provider"
keywords: "content data, providers, enrichment, extensions"
image: "/files/images/playbooks/build-enricher.svg"
kind: "playbook"
public: true
tags: ["build", "providers", "enrichment"]
published_at: "2026-01-31"
updated_at: "2026-01-31"
after_reading_this:
  - "Enrich content items with custom data"
  - "Add computed fields during content fetch"
  - "Target specific sources with applies_to_sources()"
related_docs:
  - title: "Web Extensions Overview"
    url: "/documentation/extensions/web"
  - title: "Lifecycle Hooks Reference"
    url: "/documentation/extensions/hooks"
---

# Creating Content Data Providers

This playbook walks you through creating a ContentDataProvider that enriches content items with additional data before rendering.

## When It Runs

ContentDataProvider runs during `contents_to_json()`, early in the rendering pipeline:

```
Database Query
     ↓
═══════════════════════════════════════
ContentDataProvider::enrich_content()  ← You are here
═══════════════════════════════════════
     ↓
PageDataProvider::provide_page_data()
     ↓
ComponentRenderer::render()
     ↓
Template rendering
```

This is the right place to:
- Add computed fields to content items
- Fetch related content from the database
- Add children for index pages
- Enrich with external data

## Prerequisites

- Existing extension crate in `extensions/`
- Understanding of what data needs to be added to content
- Source name(s) you want to target

## Step 1: Define Your Provider Struct

Create a new file in your extension's `enrichers/` directory:

```rust
use systemprompt::extension::prelude::{ContentDataContext, ContentDataProvider};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;

pub struct DocsContentDataProvider;

impl DocsContentDataProvider {
    pub fn new() -> Self {
        Self
    }
}
```

## Step 2: Implement the Trait

Implement `ContentDataProvider`:

```rust
#[async_trait]
impl ContentDataProvider for DocsContentDataProvider {
    fn provider_id(&self) -> &'static str {
        "docs-content-enricher"
    }

    fn applies_to_sources(&self) -> Vec<String> {
        vec!["documentation".to_string()]
    }

    async fn enrich_content(
        &self,
        ctx: &ContentDataContext<'_>,
        item: &mut serde_json::Value,
    ) -> Result<()> {
        Ok(())
    }
}
```

## Step 3: Add Children for Index Pages

For documentation index pages, add child pages:

```rust
async fn enrich_content(
    &self,
    ctx: &ContentDataContext<'_>,
    item: &mut serde_json::Value,
) -> Result<()> {
    let kind = item.get("kind").and_then(|v| v.as_str()).unwrap_or("");

    if kind == "docs-index" {
        let children = self.fetch_children(ctx).await?;
        if let Some(obj) = item.as_object_mut() {
            obj.insert("children".to_string(), json!(children));
        }
    }

    Ok(())
}

async fn fetch_children(
    &self,
    ctx: &ContentDataContext<'_>,
) -> Result<Vec<serde_json::Value>> {
    let pool = ctx.db_pool();
    let source_name = ctx.source_name();
    let slug = ctx.slug();

    let prefix = if slug.is_empty() {
        String::new()
    } else {
        format!("{}/", slug)
    };

    let children = sqlx::query!(
        r#"
        SELECT slug, title, description, kind
        FROM markdown_content
        WHERE source_id = $1
          AND slug LIKE $2
          AND slug != $3
        ORDER BY title
        "#,
        source_name,
        format!("{}%", prefix),
        slug
    )
    .fetch_all(pool.as_ref())
    .await?;

    Ok(children
        .into_iter()
        .map(|row| json!({
            "slug": row.slug,
            "title": row.title,
            "description": row.description,
            "kind": row.kind,
        }))
        .collect())
}
```

## Step 4: Add Related Content

Fetch and add related posts:

```rust
async fn enrich_content(
    &self,
    ctx: &ContentDataContext<'_>,
    item: &mut serde_json::Value,
) -> Result<()> {
    let related = self.fetch_related(ctx, item).await?;

    if let Some(obj) = item.as_object_mut() {
        obj.insert("related_posts".to_string(), json!(related));
    }

    Ok(())
}

async fn fetch_related(
    &self,
    ctx: &ContentDataContext<'_>,
    item: &serde_json::Value,
) -> Result<Vec<serde_json::Value>> {
    let pool = ctx.db_pool();
    let current_slug = item.get("slug").and_then(|v| v.as_str()).unwrap_or("");

    let related = sqlx::query!(
        r#"
        SELECT slug, title, description
        FROM markdown_content
        WHERE source_id = $1
          AND slug != $2
        ORDER BY published_at DESC
        LIMIT 3
        "#,
        ctx.source_name(),
        current_slug
    )
    .fetch_all(pool.as_ref())
    .await?;

    Ok(related
        .into_iter()
        .map(|row| json!({
            "slug": row.slug,
            "title": row.title,
            "description": row.description,
        }))
        .collect())
}
```

## Step 5: Target Specific Sources

Use `applies_to_sources()` to only run for specific content sources:

```rust
fn applies_to_sources(&self) -> Vec<String> {
    vec!["documentation".to_string(), "guides".to_string()]
}
```

Return an empty vector to run for ALL sources:

```rust
fn applies_to_sources(&self) -> Vec<String> {
    vec![]
}
```

## Step 6: Register in Extension

Add to your extension's `content_data_providers()`:

```rust
impl Extension for WebExtension {
    fn content_data_providers(&self) -> Vec<Arc<dyn ContentDataProvider>> {
        vec![
            Arc::new(DocsContentDataProvider::new()),
        ]
    }
}
```

## Step 7: Export from Module

Update `enrichers/mod.rs`:

```rust
mod docs;

pub use docs::DocsContentDataProvider;
```

## Step 8: Use Enriched Data in Templates

The enriched data is available to PageDataProviders and ComponentRenderers:

```rust
async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
    let item = ctx.content_item()?;

    let children = item.get("children").cloned().unwrap_or(json!([]));

    Ok(json!({
        "CHILDREN": children,
    }))
}
```

Or in templates via Handlebars:

```handlebars
{{#if children}}
<nav class="children">
    {{#each children}}
    <a href="/{{this.slug}}">{{this.title}}</a>
    {{/each}}
</nav>
{{/if}}
```

## Step 9: Test

Run the publish pipeline:

```bash
systemprompt infra jobs run publish_pipeline
```

Check generated pages for enriched data.

## Complete Example

```rust
use systemprompt::extension::prelude::{ContentDataContext, ContentDataProvider};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;

pub struct DocsContentDataProvider;

impl DocsContentDataProvider {
    pub fn new() -> Self {
        Self
    }

    async fn fetch_children(
        &self,
        ctx: &ContentDataContext<'_>,
    ) -> Result<Vec<serde_json::Value>> {
        let pool = ctx.db_pool();
        let source_name = ctx.source_name();
        let slug = ctx.slug();

        let prefix = if slug.is_empty() {
            String::new()
        } else {
            format!("{}/", slug)
        };

        let children = sqlx::query!(
            r#"
            SELECT slug, title, description, kind
            FROM markdown_content
            WHERE source_id = $1
              AND slug LIKE $2
              AND slug != $3
            ORDER BY title
            "#,
            source_name,
            format!("{}%", prefix),
            slug
        )
        .fetch_all(pool.as_ref())
        .await?;

        Ok(children
            .into_iter()
            .map(|row| json!({
                "slug": row.slug,
                "title": row.title,
                "description": row.description,
                "kind": row.kind,
            }))
            .collect())
    }
}

#[async_trait]
impl ContentDataProvider for DocsContentDataProvider {
    fn provider_id(&self) -> &'static str {
        "docs-content-enricher"
    }

    fn applies_to_sources(&self) -> Vec<String> {
        vec!["documentation".to_string()]
    }

    async fn enrich_content(
        &self,
        ctx: &ContentDataContext<'_>,
        item: &mut serde_json::Value,
    ) -> Result<()> {
        let kind = item.get("kind").and_then(|v| v.as_str()).unwrap_or("");

        if kind == "docs-index" {
            let children = self.fetch_children(ctx).await?;
            if let Some(obj) = item.as_object_mut() {
                obj.insert("children".to_string(), json!(children));
            }
        }

        Ok(())
    }
}
```

## Checklist

- [ ] Created provider struct
- [ ] Implemented ContentDataProvider trait
- [ ] Set correct `provider_id()`
- [ ] Configured `applies_to_sources()` targeting
- [ ] Implemented `enrich_content()` logic
- [ ] Added database queries for enrichment
- [ ] Registered in extension
- [ ] Exported from module
- [ ] Verified enriched data available in providers
- [ ] Tested with publish pipeline
