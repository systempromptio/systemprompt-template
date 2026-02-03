---
title: "Creating Template Data Extenders"
description: "Step-by-step guide to creating TemplateDataExtender implementations for final template modifications."
author: "SystemPrompt Team"
slug: "build-template-data-extender"
keywords: "template data, extenders, final modifications, extensions"
image: "/files/images/playbooks/build-extender.svg"
kind: "playbook"
public: true
tags: ["build", "extenders", "templates"]
published_at: "2026-01-31"
updated_at: "2026-01-31"
after_reading_this:
  - "Create a TemplateDataExtender for final template modifications"
  - "Add computed fields after providers and components run"
  - "Target specific content types with applies_to()"
related_docs:
  - title: "Web Extensions Overview"
    url: "/documentation/extensions/web"
related_code:
  - title: "TemplateDataExtender Trait"
    url: "https://github.com/systempromptio/systemprompt-core/blob/main/crates/shared/provider-contracts/src/extender.rs#L158-L171"
  - title: "ExtenderContext Struct"
    url: "https://github.com/systempromptio/systemprompt-core/blob/main/crates/shared/provider-contracts/src/extender.rs#L8-L17"
---

# Creating Template Data Extenders

This playbook walks you through creating a TemplateDataExtender that modifies template data after PageDataProviders and ComponentRenderers have run.

## When It Runs

TemplateDataExtender runs last in the rendering pipeline:

```
Database Query
     |
ContentDataProvider::enrich_content()
     |
PageDataProvider::provide_page_data()
     |
ComponentRenderer::render()
     |
=======================================
TemplateDataExtender::extend()  <-- You are here
=======================================
     |
Template rendering
```

This is the right place to:
- Add computed fields based on assembled data
- Generate canonical URLs from slugs
- Add schema.org structured data
- Final transformations before rendering

## Prerequisites

- Existing extension crate in `extensions/`
- Understanding of what final modifications are needed
- Content type(s) you want to target

## Step 1: Define Your Extender Struct

Create a new file in your extension's `extenders/` directory:

```rust
use systemprompt::template_provider::{ExtenderContext, TemplateDataExtender};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct CanonicalUrlExtender;

impl CanonicalUrlExtender {
    pub fn new() -> Self {
        Self
    }
}
```

## Step 2: Implement the Trait

Implement `TemplateDataExtender`:

```rust
#[async_trait]
impl TemplateDataExtender for CanonicalUrlExtender {
    fn extender_id(&self) -> &str {
        "canonical-url"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]
    }

    async fn extend(
        &self,
        ctx: &ExtenderContext<'_>,
        data: &mut Value,
    ) -> Result<()> {
        let slug = ctx.item.get("slug")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let canonical = ctx.url_pattern.replace("{slug}", slug);

        if let Some(obj) = data.as_object_mut() {
            obj.insert("CANONICAL_PATH".to_string(), json!(canonical));
        }

        Ok(())
    }
}
```

## Step 3: Access ExtenderContext Fields

The `ExtenderContext` provides access to all rendering context:

```rust
async fn extend(
    &self,
    ctx: &ExtenderContext<'_>,
    data: &mut Value,
) -> Result<()> {
    let item = &ctx.item;
    let all_items = ctx.all_items;
    let config = &ctx.config;
    let web_config = ctx.web_config;
    let content_html = ctx.content_html;
    let url_pattern = ctx.url_pattern;
    let source_name = ctx.source_name;

    let pool = ctx.db_pool::<Arc<DbPool>>();

    Ok(())
}
```

| Field | Type | Description |
|-------|------|-------------|
| `item` | `&Value` | Current content item JSON |
| `all_items` | `&[Value]` | All content items for this source |
| `config` | `&serde_yaml::Value` | Content source configuration |
| `web_config` | `&WebConfig` | Web configuration |
| `content_html` | `&str` | Rendered HTML content |
| `url_pattern` | `&str` | URL pattern for this content type |
| `source_name` | `&str` | Content source name |

## Step 4: Add Structured Data

Generate schema.org JSON-LD:

```rust
pub struct SchemaOrgExtender;

#[async_trait]
impl TemplateDataExtender for SchemaOrgExtender {
    fn extender_id(&self) -> &str {
        "schema-org"
    }

    fn applies_to(&self) -> Vec<String> {
        vec!["blog".to_string()]
    }

    async fn extend(
        &self,
        ctx: &ExtenderContext<'_>,
        data: &mut Value,
    ) -> Result<()> {
        let title = ctx.item.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let description = ctx.item.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let published = ctx.item.get("published_at")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let schema = json!({
            "@context": "https://schema.org",
            "@type": "Article",
            "headline": title,
            "description": description,
            "datePublished": published,
        });

        if let Some(obj) = data.as_object_mut() {
            obj.insert("SCHEMA_ORG".to_string(), json!(schema.to_string()));
        }

        Ok(())
    }
}
```

## Step 5: Add Reading Time

Calculate reading time from content:

```rust
pub struct ReadingTimeExtender;

#[async_trait]
impl TemplateDataExtender for ReadingTimeExtender {
    fn extender_id(&self) -> &str {
        "reading-time"
    }

    fn applies_to(&self) -> Vec<String> {
        vec!["blog".to_string(), "documentation".to_string()]
    }

    async fn extend(
        &self,
        ctx: &ExtenderContext<'_>,
        data: &mut Value,
    ) -> Result<()> {
        let word_count = ctx.content_html
            .split_whitespace()
            .count();

        let reading_time = (word_count / 200).max(1);

        if let Some(obj) = data.as_object_mut() {
            obj.insert("READING_TIME".to_string(), json!(reading_time));
            obj.insert("READING_TIME_LABEL".to_string(),
                json!(format!("{} min read", reading_time)));
        }

        Ok(())
    }
}
```

## Step 6: Target Specific Content Types

Use `applies_to()` to run for specific content types:

```rust
fn applies_to(&self) -> Vec<String> {
    vec!["blog".to_string(), "documentation".to_string()]
}
```

Return empty vector to run for ALL content types:

```rust
fn applies_to(&self) -> Vec<String> {
    vec![]
}
```

## Step 7: Set Priority

Control execution order with priority (lower runs first):

```rust
fn priority(&self) -> u32 {
    50
}
```

## Step 8: Register in Extension

Add to your extension's `template_data_extenders()`:

```rust
impl Extension for WebExtension {
    fn template_data_extenders(&self) -> Vec<Arc<dyn TemplateDataExtender>> {
        vec![
            Arc::new(CanonicalUrlExtender::new()),
            Arc::new(SchemaOrgExtender::new()),
            Arc::new(ReadingTimeExtender::new()),
        ]
    }
}
```

## Step 9: Export from Module

Update `extenders/mod.rs`:

```rust
mod canonical_url;
mod schema_org;
mod reading_time;

pub use canonical_url::CanonicalUrlExtender;
pub use schema_org::SchemaOrgExtender;
pub use reading_time::ReadingTimeExtender;
```

## Step 10: Use in Templates

The extended data is available in templates:

```handlebars
<head>
    <link rel="canonical" href="{{CANONICAL_PATH}}">
    <script type="application/ld+json">{{{SCHEMA_ORG}}}</script>
</head>

<article>
    <span class="reading-time">{{READING_TIME_LABEL}}</span>
</article>
```

## Step 11: Test

Run the publish pipeline:

```bash
systemprompt infra jobs run publish_pipeline
```

Check generated HTML for your extended data.

## Complete Example

```rust
use systemprompt::template_provider::{ExtenderContext, TemplateDataExtender};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct CanonicalUrlExtender {
    base_url: String,
}

impl CanonicalUrlExtender {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }
}

#[async_trait]
impl TemplateDataExtender for CanonicalUrlExtender {
    fn extender_id(&self) -> &str {
        "canonical-url"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]
    }

    fn priority(&self) -> u32 {
        100
    }

    async fn extend(
        &self,
        ctx: &ExtenderContext<'_>,
        data: &mut Value,
    ) -> Result<()> {
        let slug = ctx.item.get("slug")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let path = ctx.url_pattern.replace("{slug}", slug);
        let canonical = format!("{}{}", self.base_url, path);

        if let Some(obj) = data.as_object_mut() {
            obj.insert("CANONICAL_URL".to_string(), json!(canonical));
            obj.insert("CANONICAL_PATH".to_string(), json!(path));
        }

        Ok(())
    }
}
```

## Checklist

- [ ] Created extender struct
- [ ] Implemented TemplateDataExtender trait
- [ ] Set correct `extender_id()`
- [ ] Configured `applies_to()` targeting
- [ ] Implemented `extend()` logic
- [ ] Set priority if needed
- [ ] Registered in extension
- [ ] Exported from module
- [ ] Verified data available in templates
- [ ] Tested with publish pipeline
