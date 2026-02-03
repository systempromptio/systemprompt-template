---
title: "Creating Page Data Providers"
description: "Step-by-step guide to creating PageDataProvider implementations for template variables."
author: "SystemPrompt Team"
slug: "build-page-data-provider"
keywords: "page data, providers, templates, variables, extensions"
image: "/files/images/playbooks/build-provider.svg"
kind: "playbook"
public: true
tags: ["build", "providers", "templates"]
published_at: "2026-01-31"
updated_at: "2026-01-31"
after_reading_this:
  - "Create a PageDataProvider for any content type"
  - "Extract fields from content items to template variables"
  - "Target specific content types with applies_to_pages()"
related_docs:
  - title: "Page Data Providers Reference"
    url: "/documentation/extensions/page-data-providers"
  - title: "Web Extensions Overview"
    url: "/documentation/extensions/web"
---

# Creating Page Data Providers

This playbook walks you through creating a PageDataProvider that supplies template variables for your pages.

## Prerequisites

- Existing extension crate in `extensions/`
- Understanding of what template variables your templates need
- Content type(s) you want to target

## Step 1: Define Your Provider Struct

Create a new file in your extension's `providers/` directory:

```rust
use systemprompt::extension::prelude::{PageContext, PageDataProvider};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct ContentPageDataProvider;

impl ContentPageDataProvider {
    pub fn new() -> Self {
        Self
    }
}
```

## Step 2: Implement the Trait

Implement `PageDataProvider`:

```rust
#[async_trait]
impl PageDataProvider for ContentPageDataProvider {
    fn provider_id(&self) -> &'static str {
        "content-page-data"
    }

    fn applies_to_pages(&self) -> Vec<String> {
        vec![]
    }

    async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
        let item = ctx.content_item()
            .ok_or_else(|| anyhow::anyhow!("No content item available"))?;

        let title = item.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let description = item.get("description")
            .or_else(|| item.get("excerpt"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        Ok(json!({
            "TITLE": title,
            "DESCRIPTION": description,
        }))
    }
}
```

## Step 3: Add Date Formatting

Most content needs formatted dates:

```rust
async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
    let item = ctx.content_item()
        .ok_or_else(|| anyhow::anyhow!("No content item"))?;

    let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let description = item.get("description").and_then(|v| v.as_str()).unwrap_or("");

    let published = item.get("published_at")
        .or_else(|| item.get("date"))
        .and_then(|v| v.as_str());

    let (formatted_date, iso_date) = match published {
        Some(date_str) => {
            match chrono::DateTime::parse_from_rfc3339(date_str) {
                Ok(dt) => (
                    dt.format("%B %d, %Y").to_string(),
                    dt.format("%Y-%m-%d").to_string(),
                ),
                Err(_) => (date_str.to_string(), date_str.to_string()),
            }
        }
        None => (String::new(), String::new()),
    };

    Ok(json!({
        "TITLE": title,
        "DESCRIPTION": description,
        "DATE": formatted_date,
        "DATE_ISO": iso_date,
    }))
}
```

## Step 4: Add Author and Keywords

Extract additional metadata:

```rust
async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
    let item = ctx.content_item()
        .ok_or_else(|| anyhow::anyhow!("No content item"))?;

    let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let description = item.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let author = item.get("author").and_then(|v| v.as_str()).unwrap_or("");

    let keywords = item.get("keywords")
        .or_else(|| item.get("tags"))
        .cloned()
        .unwrap_or(json!([]));

    let published = item.get("published_at")
        .or_else(|| item.get("date"))
        .and_then(|v| v.as_str());

    let (formatted_date, iso_date) = format_date(published);

    Ok(json!({
        "TITLE": title,
        "DESCRIPTION": description,
        "AUTHOR": author,
        "KEYWORDS": keywords,
        "DATE": formatted_date,
        "DATE_ISO": iso_date,
    }))
}

fn format_date(date_str: Option<&str>) -> (String, String) {
    match date_str {
        Some(s) => chrono::DateTime::parse_from_rfc3339(s)
            .map(|dt| (
                dt.format("%B %d, %Y").to_string(),
                dt.format("%Y-%m-%d").to_string(),
            ))
            .unwrap_or_else(|_| (s.to_string(), s.to_string())),
        None => (String::new(), String::new()),
    }
}
```

## Step 5: Target Specific Content Types

To only run for specific pages:

```rust
fn applies_to_pages(&self) -> Vec<String> {
    vec!["blog".to_string(), "docs".to_string()]
}
```

For list pages, the content type is `{source}-list`:

```rust
fn applies_to_pages(&self) -> Vec<String> {
    vec!["blog-list".to_string(), "documentation-list".to_string()]
}
```

## Step 6: Register in Extension

Add to your extension's `page_data_providers()`:

```rust
impl Extension for WebExtension {
    fn page_data_providers(&self) -> Vec<Arc<dyn PageDataProvider>> {
        vec![
            Arc::new(ContentPageDataProvider::new()),
        ]
    }
}
```

## Step 7: Export from Module

Update `providers/mod.rs`:

```rust
mod content_page;

pub use content_page::ContentPageDataProvider;
```

## Step 8: Verify Template Usage

Check your templates use the variables:

```handlebars
<head>
    <title>{{TITLE}}</title>
    <meta name="description" content="{{DESCRIPTION}}">
    <meta name="author" content="{{AUTHOR}}">
</head>

<article>
    <h1>{{TITLE}}</h1>
    <time datetime="{{DATE_ISO}}">{{DATE}}</time>
    <p class="author">By {{AUTHOR}}</p>
</article>
```

## Step 9: Test

Run the publish pipeline to verify:

```bash
systemprompt infra jobs run publish_pipeline
```

Check generated HTML for your variables.

## Complete Example

```rust
use systemprompt::extension::prelude::{PageContext, PageDataProvider};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct ContentPageDataProvider;

impl ContentPageDataProvider {
    pub fn new() -> Self {
        Self
    }

    fn format_date(date_str: Option<&str>) -> (String, String) {
        match date_str {
            Some(s) => chrono::DateTime::parse_from_rfc3339(s)
                .map(|dt| (
                    dt.format("%B %d, %Y").to_string(),
                    dt.format("%Y-%m-%d").to_string(),
                ))
                .unwrap_or_else(|_| (s.to_string(), s.to_string())),
            None => (String::new(), String::new()),
        }
    }
}

#[async_trait]
impl PageDataProvider for ContentPageDataProvider {
    fn provider_id(&self) -> &'static str {
        "content-page-data"
    }

    fn applies_to_pages(&self) -> Vec<String> {
        vec![]
    }

    async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
        let item = ctx.content_item()
            .ok_or_else(|| anyhow::anyhow!("No content item"))?;

        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let description = item.get("description")
            .or_else(|| item.get("excerpt"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let author = item.get("author").and_then(|v| v.as_str()).unwrap_or("");

        let keywords = item.get("keywords")
            .or_else(|| item.get("tags"))
            .cloned()
            .unwrap_or(json!([]));

        let published = item.get("published_at")
            .or_else(|| item.get("date"))
            .and_then(|v| v.as_str());

        let (formatted_date, iso_date) = Self::format_date(published);

        Ok(json!({
            "TITLE": title,
            "DESCRIPTION": description,
            "AUTHOR": author,
            "KEYWORDS": keywords,
            "DATE": formatted_date,
            "DATE_ISO": iso_date,
        }))
    }
}
```

## Checklist

- [ ] Created provider struct
- [ ] Implemented PageDataProvider trait
- [ ] Set correct `provider_id()`
- [ ] Configured `applies_to_pages()` targeting
- [ ] Extracted all needed fields from content item
- [ ] Added date formatting
- [ ] Registered in extension
- [ ] Exported from module
- [ ] Verified template variables
- [ ] Tested with publish pipeline
