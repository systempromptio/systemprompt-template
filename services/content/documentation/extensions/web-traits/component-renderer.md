---
title: "Component Renderers"
description: "Create ComponentRenderer implementations to generate HTML fragments for your templates."
author: "SystemPrompt Team"
slug: "extensions/web-traits/component-renderer"
keywords: "component, renderer, html, partials, templates, extensions"
image: "/files/images/docs/extensions-components.svg"
kind: "reference"
public: true
tags: []
published_at: "2026-01-31"
updated_at: "2026-02-02"
---

# Component Renderers

ComponentRenderer generates HTML fragments that are inserted into template variables. Use this for content cards, navigation menus, related content sections, and any pre-rendered HTML that templates need.

## When It Runs

ComponentRenderer runs after PageDataProviders, allowing components to access all template data:

```
Database Query
     ↓
ContentDataProvider::enrich_content()
     ↓
PageDataProvider::provide_page_data()
     ↓
═══════════════════════════════════════
ComponentRenderer::render()            ← You are here
═══════════════════════════════════════
     ↓
TemplateDataExtender::extend()
     ↓
Handlebars template rendering
```

## The ComponentRenderer Trait

```rust
#[async_trait]
pub trait ComponentRenderer: Send + Sync {
    fn component_id(&self) -> &str;

    fn variable_name(&self) -> &str;

    fn applies_to(&self) -> Vec<String>;

    fn partial_template(&self) -> Option<PartialTemplate> {
        None
    }

    async fn render(&self, ctx: &ComponentContext<'_>) -> Result<RenderedComponent>;
}
```

### Methods

| Method | Purpose |
|--------|---------|
| `component_id()` | Unique identifier for logging and debugging |
| `variable_name()` | Template variable to populate with rendered HTML |
| `applies_to()` | Content types this renderer runs for (empty = all) |
| `partial_template()` | Optional Handlebars partial to use instead of `render()` |
| `render()` | Returns HTML to insert into the template variable |

## ComponentContext

The `ComponentContext` provides access to content data:

```rust
pub struct ComponentContext<'a> {
    pub web_config: &'a FullWebConfig,
    pub content_item: Option<&'a Value>,
    pub all_items: Option<&'a [Value]>,
    pub popular_ids: Option<&'a [String]>,
}

impl<'a> ComponentContext<'a> {
    pub fn for_content(
        web_config: &'a FullWebConfig,
        item: &'a Value,
        all_items: &'a [Value],
        popular_ids: &'a [String],
    ) -> Self;

    pub fn for_list(
        web_config: &'a FullWebConfig,
        items: &'a [Value],
    ) -> Self;
}
```

## Basic Implementation

```rust
use systemprompt::template_provider::{ComponentContext, ComponentRenderer, RenderedComponent};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

pub struct ContentCardsRenderer;

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
            .map(|item| self.render_single_card(item))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(RenderedComponent::new("POSTS", html))
    }
}

impl ContentCardsRenderer {
    fn render_single_card(&self, item: &Value) -> String {
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

## RenderedComponent

The return type contains the variable name and HTML content:

```rust
pub struct RenderedComponent {
    pub variable_name: String,
    pub html: String,
}

impl RenderedComponent {
    pub fn new(variable_name: impl Into<String>, html: impl Into<String>) -> Self {
        Self {
            variable_name: variable_name.into(),
            html: html.into(),
        }
    }
}
```

## Targeting Content Types

Use `applies_to()` to run only for specific content types:

```rust
fn applies_to(&self) -> Vec<String> {
    vec!["blog-list".to_string(), "documentation-list".to_string()]
}
```

Return an empty vector to run for ALL content types:

```rust
fn applies_to(&self) -> Vec<String> {
    vec![]
}
```

## Using Partial Templates

For complex rendering, delegate to a Handlebars partial:

```rust
pub struct NavigationRenderer;

#[async_trait]
impl ComponentRenderer for NavigationRenderer {
    fn component_id(&self) -> &str {
        "navigation"
    }

    fn variable_name(&self) -> &str {
        "NAVIGATION"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]
    }

    fn partial_template(&self) -> Option<PartialTemplate> {
        Some(PartialTemplate {
            name: "partials/navigation".to_string(),
        })
    }

    async fn render(&self, _ctx: &ComponentContext<'_>) -> Result<RenderedComponent> {
        Ok(RenderedComponent::new("NAVIGATION", String::new()))
    }
}
```

When `partial_template()` returns `Some`, the registry renders the partial using the current template data instead of calling `render()`.

## Related Content Renderer

```rust
pub struct RelatedContentRenderer;

#[async_trait]
impl ComponentRenderer for RelatedContentRenderer {
    fn component_id(&self) -> &str {
        "related-content"
    }

    fn variable_name(&self) -> &str {
        "RELATED_CONTENT"
    }

    fn applies_to(&self) -> Vec<String> {
        vec!["blog".to_string(), "docs".to_string()]
    }

    async fn render(&self, ctx: &ComponentContext<'_>) -> Result<RenderedComponent> {
        let current_item = ctx.content_item.ok_or_else(|| anyhow::anyhow!("No content item"))?;
        let all_items = ctx.all_items.unwrap_or(&[]);

        let current_slug = current_item
            .get("slug")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let related: Vec<String> = all_items
            .iter()
            .filter(|item| {
                item.get("slug")
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| s != current_slug && !s.is_empty())
            })
            .take(3)
            .map(|item| self.render_related_card(item))
            .collect();

        if related.is_empty() {
            return Ok(RenderedComponent::new("RELATED_CONTENT", String::new()));
        }

        let html = format!(
            r#"<section class="related-content">
  <h2>Related Posts</h2>
  <div class="related-grid">
    {}
  </div>
</section>"#,
            related.join("\n")
        );

        Ok(RenderedComponent::new("RELATED_CONTENT", html))
    }
}

impl RelatedContentRenderer {
    fn render_related_card(&self, item: &Value) -> String {
        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let slug = item.get("slug").and_then(|v| v.as_str()).unwrap_or("");

        format!(
            r#"<a href="/{slug}" class="related-card">
  <h4>{title}</h4>
</a>"#
        )
    }
}
```

## Popular Items Renderer

Access popular item IDs from the context:

```rust
pub struct PopularPostsRenderer;

#[async_trait]
impl ComponentRenderer for PopularPostsRenderer {
    fn component_id(&self) -> &str {
        "popular-posts"
    }

    fn variable_name(&self) -> &str {
        "POPULAR_POSTS"
    }

    fn applies_to(&self) -> Vec<String> {
        vec!["blog".to_string()]
    }

    async fn render(&self, ctx: &ComponentContext<'_>) -> Result<RenderedComponent> {
        let popular_ids = ctx.popular_ids.unwrap_or(&[]);
        let all_items = ctx.all_items.unwrap_or(&[]);

        let popular: Vec<&Value> = popular_ids
            .iter()
            .filter_map(|id| {
                all_items.iter().find(|item| {
                    item.get("id").and_then(|v| v.as_str()) == Some(id)
                })
            })
            .take(5)
            .collect();

        let html = popular
            .iter()
            .map(|item| self.render_item(item))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(RenderedComponent::new("POPULAR_POSTS", html))
    }
}
```

## Registration

Register ComponentRenderers in your extension:

```rust
impl Extension for WebExtension {
    fn component_renderers(&self) -> Vec<Arc<dyn ComponentRenderer>> {
        vec![
            Arc::new(ContentCardsRenderer),
            Arc::new(RelatedContentRenderer),
            Arc::new(PopularPostsRenderer),
            Arc::new(NavigationRenderer::new(self.nav_config.clone())),
        ]
    }
}
```

## Template Usage

Use the triple-brace syntax to insert unescaped HTML:

```handlebars
<main>
    {{{CONTENT}}}

    {{{RELATED_CONTENT}}}

    {{{POPULAR_POSTS}}}
</main>

<aside>
    {{{NAVIGATION}}}
</aside>
```

Double braces escape HTML, triple braces insert raw HTML.

## Order of Execution

Components execute in registration order. If one component depends on another's output, register them in the correct order:

```rust
fn component_renderers(&self) -> Vec<Arc<dyn ComponentRenderer>> {
    vec![
        Arc::new(CardDataProvider),
        Arc::new(CardListRenderer),
    ]
}
```

## Error Handling

When a component fails, the generator logs a warning and continues with an empty value:

```rust
async fn render(&self, ctx: &ComponentContext<'_>) -> Result<RenderedComponent> {
    let items = ctx.all_items
        .ok_or_else(|| anyhow::anyhow!("all_items required for card rendering"))?;

    if items.is_empty() {
        return Err(anyhow::anyhow!("No items to render"));
    }

    Ok(RenderedComponent::new("POSTS", self.render_cards(items)))
}
```

## Testing

Test renderers by constructing ComponentContext:

```rust
#[tokio::test]
async fn test_cards_renderer() {
    let items = vec![
        json!({ "slug": "post-1", "title": "First Post", "description": "Description 1" }),
        json!({ "slug": "post-2", "title": "Second Post", "description": "Description 2" }),
    ];

    let ctx = ComponentContext::for_list(&FullWebConfig::default(), &items);
    let renderer = ContentCardsRenderer;

    let result = renderer.render(&ctx).await.unwrap();

    assert_eq!(result.variable_name, "POSTS");
    assert!(result.html.contains("First Post"));
    assert!(result.html.contains("Second Post"));
}
```