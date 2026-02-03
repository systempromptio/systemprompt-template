---
title: "Frontmatter Processor"
description: "Parse custom frontmatter fields during content ingestion."
author: "SystemPrompt Team"
slug: "extensions/web-traits/frontmatter-processor"
keywords: "frontmatter, processor, yaml, parsing, extensions"
image: ""
kind: "reference"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Frontmatter Processor

FrontmatterProcessor parses and transforms custom frontmatter fields during content ingestion.

## When It Runs

```
Markdown file read
     |
YAML frontmatter parsed
     |
=======================================
FrontmatterProcessor::process()  <- You are here
=======================================
     |
ContentMetadata struct created
     |
Database insert
```

## The Trait

```rust
#[async_trait]
pub trait FrontmatterProcessor: Send + Sync {
    fn processor_id(&self) -> &'static str;

    fn applies_to_sources(&self) -> Vec<String>;

    fn priority(&self) -> u32 {
        100
    }

    async fn process(
        &self,
        ctx: &FrontmatterContext<'_>,
        frontmatter: &mut Value,
    ) -> Result<()>;
}
```

## FrontmatterContext

```rust
pub struct FrontmatterContext<'a> {
    pub source_name: &'a str,
    pub file_path: &'a Path,
    pub raw_content: &'a str,
}
```

## Basic Implementation

```rust
use systemprompt::extension::prelude::{FrontmatterContext, FrontmatterProcessor};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct CustomFieldProcessor;

#[async_trait]
impl FrontmatterProcessor for CustomFieldProcessor {
    fn processor_id(&self) -> &'static str {
        "custom-fields"
    }

    fn applies_to_sources(&self) -> Vec<String> {
        vec![]  // All sources
    }

    async fn process(
        &self,
        ctx: &FrontmatterContext<'_>,
        frontmatter: &mut Value,
    ) -> Result<()> {
        // Parse custom date format
        if let Some(date) = frontmatter.get("date").and_then(|v| v.as_str()) {
            if let Ok(parsed) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
                let iso = parsed.format("%Y-%m-%dT00:00:00Z").to_string();
                if let Some(obj) = frontmatter.as_object_mut() {
                    obj.insert("published_at".to_string(), json!(iso));
                }
            }
        }

        // Add word count
        let word_count = ctx.raw_content.split_whitespace().count();
        if let Some(obj) = frontmatter.as_object_mut() {
            obj.insert("word_count".to_string(), json!(word_count));
        }

        Ok(())
    }
}
```

## Registration

```rust
impl Extension for WebExtension {
    fn frontmatter_processors(&self) -> Vec<Arc<dyn FrontmatterProcessor>> {
        vec![
            Arc::new(CustomFieldProcessor),
            Arc::new(TagNormalizer),
        ]
    }
}
```

## Common Patterns

### Tag Normalization

```rust
async fn process(&self, _ctx: &FrontmatterContext<'_>, frontmatter: &mut Value) -> Result<()> {
    if let Some(tags) = frontmatter.get_mut("tags") {
        if let Some(arr) = tags.as_array_mut() {
            for tag in arr.iter_mut() {
                if let Some(s) = tag.as_str() {
                    *tag = json!(s.to_lowercase().replace(" ", "-"));
                }
            }
        }
    }
    Ok(())
}
```

### Slug Generation

```rust
async fn process(&self, ctx: &FrontmatterContext<'_>, frontmatter: &mut Value) -> Result<()> {
    if frontmatter.get("slug").is_none() {
        let title = frontmatter.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let slug = title.to_lowercase()
            .replace(" ", "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>();

        if let Some(obj) = frontmatter.as_object_mut() {
            obj.insert("slug".to_string(), json!(slug));
        }
    }
    Ok(())
}
```

### Default Values

```rust
async fn process(&self, _ctx: &FrontmatterContext<'_>, frontmatter: &mut Value) -> Result<()> {
    let defaults = [
        ("public", json!(true)),
        ("author", json!("SystemPrompt Team")),
    ];

    if let Some(obj) = frontmatter.as_object_mut() {
        for (key, default) in defaults {
            if !obj.contains_key(key) {
                obj.insert(key.to_string(), default);
            }
        }
    }

    Ok(())
}
```