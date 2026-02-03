---
title: "Template Data Extender"
description: "Make final modifications to template data after all providers and renderers have run."
author: "SystemPrompt Team"
slug: "extensions/web-traits/template-data-extender"
keywords: "template, data, extender, modifications, extensions"
image: ""
kind: "reference"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Template Data Extender

TemplateDataExtender runs after PageDataProviders and ComponentRenderers, allowing you to make final modifications to the assembled template data.

## When It Runs

```
ContentDataProvider::enrich_content()
     |
PageDataProvider::provide_page_data()
     |
ComponentRenderer::render()
     |
=======================================
TemplateDataExtender::extend()  <- You are here
=======================================
     |
Handlebars template rendering
```

## The Trait

```rust
#[async_trait]
pub trait TemplateDataExtender: Send + Sync {
    fn extender_id(&self) -> &str;

    fn applies_to(&self) -> Vec<String>;

    fn priority(&self) -> u32 {
        100
    }

    async fn extend(
        &self,
        ctx: &ExtenderContext<'_>,
        data: &mut Value,
    ) -> Result<()>;
}
```

## ExtenderContext

```rust
pub struct ExtenderContext<'a> {
    pub item: &'a Value,
    pub all_items: Option<&'a [Value]>,
    pub config: &'a ContentConfigRaw,
    pub web_config: &'a FullWebConfig,
    pub content_html: Option<&'a str>,
    pub source_name: &'a str,
    pub url_pattern: &'a str,
}

impl<'a> ExtenderContext<'a> {
    pub fn db_pool<T>(&self) -> Option<&T>;
}
```

## Basic Implementation

```rust
use systemprompt::template_provider::{ExtenderContext, TemplateDataExtender};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct CanonicalUrlExtender;

#[async_trait]
impl TemplateDataExtender for CanonicalUrlExtender {
    fn extender_id(&self) -> &str {
        "canonical-url"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]  // All content types
    }

    fn priority(&self) -> u32 {
        100
    }

    async fn extend(
        &self,
        ctx: &ExtenderContext<'_>,
        data: &mut Value,
    ) -> Result<()> {
        let slug = ctx.item.get("slug").and_then(|v| v.as_str()).unwrap_or("");
        let canonical = ctx.url_pattern.replace("{slug}", slug);

        if let Some(obj) = data.as_object_mut() {
            obj.insert("CANONICAL_PATH".to_string(), json!(canonical));
            obj.insert("CANONICAL_URL".to_string(), json!(format!(
                "{}{}",
                ctx.web_config.base_url,
                canonical
            )));
        }

        Ok(())
    }
}
```

## Targeting Content Types

```rust
fn applies_to(&self) -> Vec<String> {
    vec!["blog".to_string(), "docs".to_string()]
}
```

Empty vector = all content types.

## Registration

```rust
impl Extension for WebExtension {
    fn template_data_extenders(&self) -> Vec<Arc<dyn TemplateDataExtender>> {
        vec![
            Arc::new(CanonicalUrlExtender),
            Arc::new(OpenGraphExtender),
            Arc::new(JsonLdExtender),
        ]
    }
}
```

## Common Patterns

### OpenGraph Metadata

```rust
async fn extend(&self, ctx: &ExtenderContext<'_>, data: &mut Value) -> Result<()> {
    let title = data.get("TITLE").and_then(|v| v.as_str()).unwrap_or("");
    let description = data.get("DESCRIPTION").and_then(|v| v.as_str()).unwrap_or("");
    let image = ctx.item.get("image").and_then(|v| v.as_str());

    if let Some(obj) = data.as_object_mut() {
        obj.insert("OG_TITLE".to_string(), json!(title));
        obj.insert("OG_DESCRIPTION".to_string(), json!(description));
        if let Some(img) = image {
            obj.insert("OG_IMAGE".to_string(), json!(format!("{}{}", ctx.web_config.base_url, img)));
        }
    }

    Ok(())
}
```

### JSON-LD Structured Data

```rust
async fn extend(&self, ctx: &ExtenderContext<'_>, data: &mut Value) -> Result<()> {
    let json_ld = json!({
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": data.get("TITLE"),
        "description": data.get("DESCRIPTION"),
        "author": {
            "@type": "Person",
            "name": data.get("AUTHOR")
        }
    });

    if let Some(obj) = data.as_object_mut() {
        obj.insert("JSON_LD".to_string(), json!(serde_json::to_string(&json_ld)?));
    }

    Ok(())
}
```

### Conditional Fields

```rust
async fn extend(&self, ctx: &ExtenderContext<'_>, data: &mut Value) -> Result<()> {
    let has_toc = ctx.content_html
        .map(|html| html.contains("<h2"))
        .unwrap_or(false);

    if let Some(obj) = data.as_object_mut() {
        obj.insert("SHOW_TOC".to_string(), json!(has_toc));
    }

    Ok(())
}
```