---
title: "Creating Component Renderers"
description: "Step-by-step guide to creating ComponentRenderer implementations for HTML generation."
author: "SystemPrompt Team"
slug: "build-component-renderer"
keywords: "component, renderer, html, partials, extensions"
image: "/files/images/playbooks/build-component.svg"
kind: "playbook"
public: true
tags: ["build", "components", "html"]
published_at: "2026-01-31"
updated_at: "2026-01-31"
after_reading_this:
  - "Create a ComponentRenderer for HTML generation"
  - "Output pre-rendered HTML to template variables"
  - "Target specific content types with applies_to()"
related_docs:
  - title: "Component Renderers Reference"
    url: "/documentation/extensions/component-renderers"
  - title: "Web Extensions Overview"
    url: "/documentation/extensions/web"
---

# Creating Component Renderers

This playbook walks you through creating a ComponentRenderer that generates HTML fragments for your templates.

## Prerequisites

- Existing extension crate in `extensions/`
- Understanding of what HTML fragments your templates need
- Content type(s) you want to target

## Step 1: Define Your Renderer Struct

Create a new file in your extension's `components/` directory:

```rust
use systemprompt::template_provider::{ComponentContext, ComponentRenderer, RenderedComponent};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

pub struct ContentCardsRenderer;

impl ContentCardsRenderer {
    pub fn new() -> Self {
        Self
    }
}
```

## Step 2: Implement the Trait

Implement `ComponentRenderer`:

```rust
#[async_trait]
impl ComponentRenderer for ContentCardsRenderer {
    fn component_id(&self) -> &str {
        "content-cards"
    }

    fn variable_name(&self) -> &str {
        "POSTS"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]
    }

    async fn render(&self, ctx: &ComponentContext<'_>) -> Result<RenderedComponent> {
        let items = ctx.all_items.unwrap_or(&[]);

        let html = items
            .iter()
            .filter(|item| {
                item.get("slug")
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| !s.is_empty())
            })
            .map(|item| self.render_card(item))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(RenderedComponent::new("POSTS", html))
    }
}
```

## Step 3: Create the Card Rendering Method

Add a method to render individual cards:

```rust
impl ContentCardsRenderer {
    pub fn new() -> Self {
        Self
    }

    fn render_card(&self, item: &Value) -> String {
        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let slug = item.get("slug").and_then(|v| v.as_str()).unwrap_or("");
        let description = item.get("description").and_then(|v| v.as_str()).unwrap_or("");

        format!(
            r#"<article class="card">
  <a href="/{slug}">
    <h3>{title}</h3>
    <p>{description}</p>
  </a>
</article>"#
        )
    }
}
```

## Step 4: Add Image Support

Enhance cards with images:

```rust
fn render_card(&self, item: &Value) -> String {
    let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let slug = item.get("slug").and_then(|v| v.as_str()).unwrap_or("");
    let description = item.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let image = item.get("image").and_then(|v| v.as_str());

    let image_html = match image {
        Some(src) => format!(r#"<img src="{src}" alt="{title}" loading="lazy" />"#),
        None => r#"<div class="card-placeholder"></div>"#.to_string(),
    };

    format!(
        r#"<article class="card">
  <a href="/{slug}">
    {image_html}
    <div class="card-content">
      <h3>{title}</h3>
      <p>{description}</p>
    </div>
  </a>
</article>"#
    )
}
```

## Step 5: Add Date Formatting

Include formatted dates:

```rust
fn render_card(&self, item: &Value) -> String {
    let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let slug = item.get("slug").and_then(|v| v.as_str()).unwrap_or("");
    let description = item.get("description").and_then(|v| v.as_str()).unwrap_or("");

    let date = item.get("published_at")
        .and_then(|v| v.as_str())
        .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
        .map(|dt| dt.format("%B %d, %Y").to_string())
        .unwrap_or_default();

    format!(
        r#"<article class="card">
  <a href="/{slug}">
    <h3>{title}</h3>
    <p>{description}</p>
    <time>{date}</time>
  </a>
</article>"#
    )
}
```

## Step 6: Target Specific Content Types

For list pages only:

```rust
fn applies_to(&self) -> Vec<String> {
    vec!["blog-list".to_string(), "documentation-list".to_string()]
}
```

## Step 7: Register in Extension

Add to your extension's `component_renderers()`:

```rust
impl Extension for WebExtension {
    fn component_renderers(&self) -> Vec<Arc<dyn ComponentRenderer>> {
        vec![
            Arc::new(ContentCardsRenderer::new()),
        ]
    }
}
```

## Step 8: Export from Module

Update `components/mod.rs`:

```rust
mod cards;

pub use cards::ContentCardsRenderer;
```

## Step 9: Use in Templates

Use triple braces for unescaped HTML:

```handlebars
<main class="content-list">
    {{{POSTS}}}
</main>
```

## Step 10: Test

Run the publish pipeline:

```bash
systemprompt infra jobs run publish_pipeline
```

Check generated HTML for your cards.

## Complete Example

```rust
use systemprompt::template_provider::{ComponentContext, ComponentRenderer, RenderedComponent};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

pub struct ContentCardsRenderer;

impl ContentCardsRenderer {
    pub fn new() -> Self {
        Self
    }

    fn render_card(&self, item: &Value) -> String {
        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let slug = item.get("slug").and_then(|v| v.as_str()).unwrap_or("");
        let description = item.get("description").and_then(|v| v.as_str()).unwrap_or("");

        let image = item.get("image").and_then(|v| v.as_str());
        let image_html = match image {
            Some(src) => format!(r#"<img src="{src}" alt="{title}" loading="lazy" />"#),
            None => r#"<div class="card-placeholder"></div>"#.to_string(),
        };

        let date = item.get("published_at")
            .and_then(|v| v.as_str())
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
            .map(|dt| dt.format("%B %d, %Y").to_string())
            .unwrap_or_default();

        format!(
            r#"<article class="card">
  <a href="/{slug}">
    {image_html}
    <div class="card-content">
      <h3>{title}</h3>
      <p>{description}</p>
      <time>{date}</time>
    </div>
  </a>
</article>"#
        )
    }
}

#[async_trait]
impl ComponentRenderer for ContentCardsRenderer {
    fn component_id(&self) -> &str {
        "content-cards"
    }

    fn variable_name(&self) -> &str {
        "POSTS"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]
    }

    async fn render(&self, ctx: &ComponentContext<'_>) -> Result<RenderedComponent> {
        let items = ctx.all_items.unwrap_or(&[]);

        let html = items
            .iter()
            .filter(|item| {
                item.get("slug")
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| !s.is_empty())
            })
            .map(|item| self.render_card(item))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(RenderedComponent::new("POSTS", html))
    }
}
```

## Using Partial Templates

For complex rendering, delegate to a Handlebars partial:

```rust
fn partial_template(&self) -> Option<PartialTemplate> {
    Some(PartialTemplate {
        name: "partials/card".to_string(),
    })
}
```

Create the partial at `templates/partials/card.html`:

```handlebars
{{#each items}}
<article class="card">
  <a href="/{{this.slug}}">
    <h3>{{this.title}}</h3>
    <p>{{this.description}}</p>
  </a>
</article>
{{/each}}
```

## Checklist

- [ ] Created renderer struct
- [ ] Implemented ComponentRenderer trait
- [ ] Set correct `component_id()`
- [ ] Set correct `variable_name()`
- [ ] Configured `applies_to()` targeting
- [ ] Created card rendering method
- [ ] Added image support
- [ ] Added date formatting
- [ ] Registered in extension
- [ ] Exported from module
- [ ] Used triple braces in templates
- [ ] Tested with publish pipeline
