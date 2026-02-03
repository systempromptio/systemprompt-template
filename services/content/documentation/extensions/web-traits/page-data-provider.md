---
title: "Page Data Providers"
description: "Create PageDataProvider implementations to provide ALL template variables for your pages."
author: "SystemPrompt Team"
slug: "extensions/web-traits/page-data-provider"
keywords: "page data, providers, templates, variables, extensions"
image: "/files/images/docs/extensions-providers.svg"
kind: "reference"
public: true
tags: []
published_at: "2026-01-31"
updated_at: "2026-02-02"
---

# Page Data Providers

PageDataProvider is the **primary** mechanism for providing template variables to Handlebars templates. The generator core only provides minimal data (`CONTENT`, `TOC_HTML`, `SLUG`). Your extensions must provide everything else through PageDataProvider implementations.

## When It Runs

PageDataProvider runs during content rendering, after content is loaded but before template rendering:

```
Database Query
     ↓
ContentDataProvider::enrich_content()
     ↓
═══════════════════════════════════════
PageDataProvider::provide_page_data()  ← You are here
═══════════════════════════════════════
     ↓
ComponentRenderer::render()
     ↓
TemplateDataExtender::extend()
     ↓
Handlebars template rendering
```

## The PageDataProvider Trait

```rust
#[async_trait]
pub trait PageDataProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;

    fn applies_to_pages(&self) -> Vec<String>;

    async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value>;
}
```

### Methods

| Method | Purpose |
|--------|---------|
| `provider_id()` | Unique identifier for logging and debugging |
| `applies_to_pages()` | Content types this provider runs for (empty = all) |
| `provide_page_data()` | Returns JSON data to merge into template variables |

## PageContext

The `PageContext` provides access to content and configuration:

```rust
impl<'a> PageContext<'a> {
    pub fn content_type(&self) -> &str;
    pub fn content_item(&self) -> Option<&Value>;
    pub fn all_items(&self) -> Option<&[Value]>;
    pub fn web_config(&self) -> &FullWebConfig;
    pub fn config(&self) -> &ContentConfigRaw;
    pub fn db_pool(&self) -> &DbPool;
}
```

### Content Access

```rust
async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
    let item = ctx.content_item().ok_or_else(|| anyhow!("No content item"))?;

    let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let description = item.get("description").and_then(|v| v.as_str()).unwrap_or("");

    Ok(json!({
        "TITLE": title,
        "DESCRIPTION": description,
    }))
}
```

### Database Access

```rust
async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
    let pool = ctx.db_pool();

    let related = sqlx::query!(
        "SELECT slug, title FROM markdown_content WHERE source_id = $1 LIMIT 5",
        "blog"
    )
    .fetch_all(pool.as_ref())
    .await?;

    Ok(json!({ "related_posts": related }))
}
```

## Basic Implementation

```rust
use systemprompt::extension::prelude::{PageContext, PageDataProvider};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct ContentPageDataProvider;

#[async_trait]
impl PageDataProvider for ContentPageDataProvider {
    fn provider_id(&self) -> &'static str {
        "content-page-data"
    }

    fn applies_to_pages(&self) -> Vec<String> {
        vec![]
    }

    async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
        let item = ctx.content_item().ok_or_else(|| anyhow::anyhow!("No content item"))?;

        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let description = item.get("description")
            .or_else(|| item.get("excerpt"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let published = item.get("published_at")
            .or_else(|| item.get("date"))
            .and_then(|v| v.as_str());

        let formatted_date = published
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
            .map(|dt| dt.format("%B %d, %Y").to_string())
            .unwrap_or_default();

        let iso_date = published
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_default();

        Ok(json!({
            "TITLE": title,
            "DESCRIPTION": description,
            "DATE": formatted_date,
            "DATE_ISO": iso_date,
            "AUTHOR": item.get("author").and_then(|v| v.as_str()).unwrap_or(""),
            "KEYWORDS": item.get("keywords").or_else(|| item.get("tags")),
        }))
    }
}
```

## Targeting Specific Content Types

Use `applies_to_pages()` to run only for specific content types:

```rust
fn applies_to_pages(&self) -> Vec<String> {
    vec!["blog".to_string(), "docs".to_string()]
}
```

Return an empty vector to run for ALL content types:

```rust
fn applies_to_pages(&self) -> Vec<String> {
    vec![]
}
```

## List Page Providers

For list pages (like blog index), the content type is `{source}-list`:

```rust
pub struct DocsListPageDataProvider;

#[async_trait]
impl PageDataProvider for DocsListPageDataProvider {
    fn provider_id(&self) -> &'static str {
        "docs-list-page-data"
    }

    fn applies_to_pages(&self) -> Vec<String> {
        vec!["documentation-list".to_string()]
    }

    async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
        let index_item = ctx.content_item();

        let (title, description) = match index_item {
            Some(item) => (
                item.get("title").and_then(|v| v.as_str()).unwrap_or("Documentation"),
                item.get("description").and_then(|v| v.as_str()).unwrap_or(""),
            ),
            None => ("Documentation", "Browse all documentation"),
        };

        Ok(json!({
            "TITLE": title,
            "DESCRIPTION": description,
        }))
    }
}
```

## Multiple Providers

Multiple providers can contribute to the same page. Data is merged in registration order:

```rust
impl Extension for WebExtension {
    fn page_data_providers(&self) -> Vec<Arc<dyn PageDataProvider>> {
        vec![
            Arc::new(ContentPageDataProvider),
            Arc::new(MetadataPageDataProvider::new(self.config.clone())),
            Arc::new(BrandingPageDataProvider::new(self.web_config.clone())),
            Arc::new(DocsListPageDataProvider),
        ]
    }
}
```

## Data Merging

When multiple providers return data, values are merged recursively:

```rust
{
    "TITLE": "From Provider 1",
    "meta": { "author": "Alice" }
}

{
    "DESCRIPTION": "From Provider 2",
    "meta": { "editor": "Bob" }
}

{
    "TITLE": "From Provider 1",
    "DESCRIPTION": "From Provider 2",
    "meta": { "author": "Alice", "editor": "Bob" }
}
```

Later providers can override earlier values, so order matters.

## Error Handling

When a provider fails, the generator logs a warning and continues:

```rust
async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
    let item = ctx.content_item()
        .ok_or_else(|| anyhow::anyhow!("Content item required"))?;

    if item.get("title").is_none() {
        return Err(anyhow::anyhow!("Missing required field: title"));
    }

    Ok(json!({ "TITLE": item["title"] }))
}
```

## Configuration-Based Providers

Providers can read configuration to provide site-wide data:

```rust
pub struct BrandingPageDataProvider {
    web_config: FullWebConfig,
}

impl BrandingPageDataProvider {
    pub fn new(web_config: FullWebConfig) -> Self {
        Self { web_config }
    }
}

#[async_trait]
impl PageDataProvider for BrandingPageDataProvider {
    fn provider_id(&self) -> &'static str {
        "branding-page-data"
    }

    fn applies_to_pages(&self) -> Vec<String> {
        vec![]
    }

    async fn provide_page_data(&self, _ctx: &PageContext<'_>) -> Result<Value> {
        Ok(json!({
            "SITE_NAME": self.web_config.branding.site_name,
            "TWITTER_HANDLE": self.web_config.branding.twitter_handle,
            "LOGO_PATH": self.web_config.branding.logo.primary.svg,
            "FAVICON_PATH": self.web_config.branding.favicon,
        }))
    }
}
```

## Template Usage

Template variables from providers are available in Handlebars:

```handlebars
<head>
    <title>{{TITLE}}</title>
    <meta name="description" content="{{DESCRIPTION}}">
    <meta name="author" content="{{AUTHOR}}">
    <link rel="icon" href="{{FAVICON_PATH}}">
</head>

<article>
    <h1>{{TITLE}}</h1>
    <time datetime="{{DATE_ISO}}">{{DATE}}</time>
    {{{CONTENT}}}
</article>
```

## Testing

Test providers by constructing PageContext with test data:

```rust
#[tokio::test]
async fn test_content_provider() {
    let item = json!({
        "title": "Test Post",
        "description": "A test post",
        "published_at": "2026-01-31T00:00:00Z",
    });

    let ctx = PageContext::test_context("blog", &item);
    let provider = ContentPageDataProvider;

    let result = provider.provide_page_data(&ctx).await.unwrap();

    assert_eq!(result["TITLE"], "Test Post");
    assert_eq!(result["DESCRIPTION"], "A test post");
    assert_eq!(result["DATE"], "January 31, 2026");
}
```