---
title: "Building List Pages"
description: "Complete guide to building extension-controlled list pages with providers and renderers."
author: "SystemPrompt Team"
slug: "build-list-pages"
keywords: "list pages, index, providers, renderers, extensions"
image: "/files/images/playbooks/build-list.svg"
kind: "playbook"
public: true
tags: ["build", "lists", "prerender"]
published_at: "2026-01-31"
updated_at: "2026-01-31"
after_reading_this:
  - "Create extension-controlled list pages"
  - "Combine PageDataProvider + ComponentRenderer"
  - "Render index content with enriched data"
related_docs:
  - title: "Page Data Providers Reference"
    url: "/documentation/extensions/page-data-providers"
  - title: "Component Renderers Reference"
    url: "/documentation/extensions/component-renderers"
  - title: "Web Extensions Overview"
    url: "/documentation/extensions/web"
---

# Building List Pages

This playbook walks you through building list pages (like `/blog/` or `/documentation/`) using the extension-first architecture. List pages require coordination between PageDataProviders and ComponentRenderers.

## Architecture Overview

List pages use content type `{source}-list` (e.g., `blog-list`, `documentation-list`).

The core generator provides:
- `HAS_INDEX_CONTENT` - Boolean indicating if index content exists

Your extensions provide everything else:
- `TITLE`, `DESCRIPTION` via PageDataProvider
- `POSTS`, `ITEMS` via ComponentRenderer
- Any other template variables you need

## Prerequisites

- Existing extension crate in `extensions/`
- Template registered for `{source}-list` content type
- Understanding of what data your list page needs

## Step 1: Create List PageDataProvider

Create a provider for list-specific data:

```rust
use systemprompt::extension::prelude::{PageContext, PageDataProvider};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct DocsListPageDataProvider;

impl DocsListPageDataProvider {
    pub fn new() -> Self {
        Self
    }
}

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
                item.get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Documentation"),
                item.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Browse all documentation"),
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

## Step 2: Handle Index Content

When index content exists (empty slug item), extract its data:

```rust
async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
    let index_item = ctx.content_item();

    match index_item {
        Some(item) => {
            let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("Docs");
            let description = item.get("description").and_then(|v| v.as_str()).unwrap_or("");
            let content = item.get("content").and_then(|v| v.as_str()).unwrap_or("");

            let rendered_content = if content.is_empty() {
                String::new()
            } else {
                render_markdown(content)
            };

            Ok(json!({
                "TITLE": title,
                "DESCRIPTION": description,
                "INDEX_CONTENT": rendered_content,
            }))
        }
        None => Ok(json!({
            "TITLE": "Documentation",
            "DESCRIPTION": "Browse all documentation",
            "INDEX_CONTENT": "",
        })),
    }
}
```

## Step 3: Create List ComponentRenderer

Create a renderer for the content cards:

```rust
use systemprompt::template_provider::{ComponentContext, ComponentRenderer, RenderedComponent};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

pub struct DocsCardsRenderer;

impl DocsCardsRenderer {
    pub fn new() -> Self {
        Self
    }

    fn render_card(&self, item: &Value) -> String {
        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let slug = item.get("slug").and_then(|v| v.as_str()).unwrap_or("");
        let description = item.get("description").and_then(|v| v.as_str()).unwrap_or("");
        let kind = item.get("kind").and_then(|v| v.as_str()).unwrap_or("docs");

        format!(
            r#"<article class="docs-card" data-kind="{kind}">
  <a href="/documentation/{slug}">
    <h3>{title}</h3>
    <p>{description}</p>
  </a>
</article>"#
        )
    }
}

#[async_trait]
impl ComponentRenderer for DocsCardsRenderer {
    fn component_id(&self) -> &str {
        "docs-cards"
    }

    fn variable_name(&self) -> &str {
        "POSTS"
    }

    fn applies_to(&self) -> Vec<String> {
        vec!["documentation-list".to_string()]
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

## Step 4: Add Children Renderer (Optional)

For hierarchical content, add a children renderer:

```rust
pub struct DocsChildrenRenderer;

#[async_trait]
impl ComponentRenderer for DocsChildrenRenderer {
    fn component_id(&self) -> &str {
        "docs-children"
    }

    fn variable_name(&self) -> &str {
        "CHILDREN"
    }

    fn applies_to(&self) -> Vec<String> {
        vec!["documentation-list".to_string()]
    }

    async fn render(&self, ctx: &ComponentContext<'_>) -> Result<RenderedComponent> {
        let index_item = ctx.content_item;

        let children = match index_item {
            Some(item) => item.get("children").and_then(|v| v.as_array()),
            None => None,
        };

        let html = match children {
            Some(items) => items
                .iter()
                .map(|child| {
                    let title = child.get("title").and_then(|v| v.as_str()).unwrap_or("");
                    let slug = child.get("slug").and_then(|v| v.as_str()).unwrap_or("");
                    format!(r#"<a href="/documentation/{slug}" class="child-link">{title}</a>"#)
                })
                .collect::<Vec<_>>()
                .join("\n"),
            None => String::new(),
        };

        Ok(RenderedComponent::new("CHILDREN", html))
    }
}
```

## Step 5: Register Providers and Renderers

Register in your extension:

```rust
impl Extension for WebExtension {
    fn page_data_providers(&self) -> Vec<Arc<dyn PageDataProvider>> {
        vec![
            Arc::new(DocsListPageDataProvider::new()),
        ]
    }

    fn component_renderers(&self) -> Vec<Arc<dyn ComponentRenderer>> {
        vec![
            Arc::new(DocsCardsRenderer::new()),
            Arc::new(DocsChildrenRenderer::new()),
        ]
    }
}
```

## Step 6: Create the Template

Create a template for the list page:

```handlebars
<!DOCTYPE html>
<html>
<head>
    <title>{{TITLE}}</title>
    <meta name="description" content="{{DESCRIPTION}}">
</head>
<body>
    <main>
        <h1>{{TITLE}}</h1>

        {{#if HAS_INDEX_CONTENT}}
        <div class="index-content">
            {{{INDEX_CONTENT}}}
        </div>
        {{/if}}

        {{#if CHILDREN}}
        <nav class="children-nav">
            {{{CHILDREN}}}
        </nav>
        {{/if}}

        <div class="content-grid">
            {{{POSTS}}}
        </div>
    </main>
</body>
</html>
```

## Step 7: Register the Template

Register the template for the list content type:

```rust
impl Extension for WebExtension {
    fn templates(&self) -> Vec<TemplateDefinition> {
        vec![
            TemplateDefinition::new("documentation-list", "templates/docs-list.html"),
        ]
    }
}
```

## Step 8: Test

Run the publish pipeline:

```bash
systemprompt infra jobs run publish_pipeline
```

Check the generated list page at `/documentation/index.html`.

## Troubleshooting

### Cards Not Rendering

1. Check `applies_to()` matches the content type (`documentation-list`)
2. Verify `all_items` is populated in ComponentContext
3. Check items have non-empty slugs

### Index Content Not Showing

1. Verify index content exists (empty slug item)
2. Check `HAS_INDEX_CONTENT` is true in template data
3. Verify PageDataProvider extracts content

### Children Not Appearing

1. Check ContentDataProvider adds `children` field
2. Verify ComponentRenderer reads from correct location
3. Check template uses correct variable name

## Complete Example

**PageDataProvider:**

```rust
pub struct DocsListPageDataProvider;

#[async_trait]
impl PageDataProvider for DocsListPageDataProvider {
    fn provider_id(&self) -> &'static str { "docs-list-page-data" }

    fn applies_to_pages(&self) -> Vec<String> {
        vec!["documentation-list".to_string()]
    }

    async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
        let index_item = ctx.content_item();

        let (title, description, content) = match index_item {
            Some(item) => (
                item.get("title").and_then(|v| v.as_str()).unwrap_or("Docs"),
                item.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                item.get("content").and_then(|v| v.as_str()).unwrap_or(""),
            ),
            None => ("Documentation", "Browse all documentation", ""),
        };

        Ok(json!({
            "TITLE": title,
            "DESCRIPTION": description,
            "INDEX_CONTENT": render_markdown(content),
        }))
    }
}
```

**ComponentRenderer:**

```rust
pub struct DocsCardsRenderer;

#[async_trait]
impl ComponentRenderer for DocsCardsRenderer {
    fn component_id(&self) -> &str { "docs-cards" }
    fn variable_name(&self) -> &str { "POSTS" }

    fn applies_to(&self) -> Vec<String> {
        vec!["documentation-list".to_string()]
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
            .map(|item| {
                let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
                let slug = item.get("slug").and_then(|v| v.as_str()).unwrap_or("");
                let desc = item.get("description").and_then(|v| v.as_str()).unwrap_or("");
                format!(r#"<article class="card"><a href="/documentation/{slug}"><h3>{title}</h3><p>{desc}</p></a></article>"#)
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(RenderedComponent::new("POSTS", html))
    }
}
```

## Checklist

- [ ] Created PageDataProvider for list page
- [ ] Handled index content (empty slug item)
- [ ] Created ComponentRenderer for cards
- [ ] Configured `applies_to()` with `{source}-list`
- [ ] Registered providers and renderers
- [ ] Created template with correct variables
- [ ] Registered template for content type
- [ ] Tested with publish pipeline
- [ ] Verified index content renders
- [ ] Verified cards render
- [ ] Verified children render (if applicable)
