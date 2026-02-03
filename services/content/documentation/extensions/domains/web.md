---
title: "Web Extensions"
description: "Build web extensions for page data, static generation, templates, and asset management."
author: "SystemPrompt Team"
slug: "extensions/domains/web"
keywords: "web, extensions, templates, static generation, assets, providers"
image: "/files/images/docs/extensions-web.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Web Extensions

Web extensions provide **all** page data, **all** HTML generation, component rendering, and asset management for the frontend. The generator core is coordination only - it loads content, calls providers, and writes files. Extensions control everything else.

## Architecture Principles

The generator follows an extension-first architecture:

| Component | Responsibility |
|-----------|---------------|
| **Core** | Coordination: load context, call providers, write files |
| **Extensions** | ALL content transformation, HTML generation, field mapping |

This means:
- Core provides only `CONTENT`, `TOC_HTML`, and `SLUG` to templates
- Extensions provide ALL other template variables via `PageDataProvider`
- Extensions generate ALL HTML fragments via `ComponentRenderer`
- No hardcoded field mappings or HTML generation in core

## Complete Data Flow

Understanding how data flows from content to templates:

```
Markdown File (frontmatter + body)
         |
    parse_markdown() [ingestion.rs]
         |
    ContentMetadata struct
         |
    markdown_content table (PostgreSQL)
         |
    ContentDataProvider::enrich_content()
         |
    PageDataProvider::provide_page_data()
         |
    ComponentRenderer::render()
         |
    TemplateDataExtender::extend()
         |
    Template variables (JSON)
         |
    Handlebars template rendering
         |
    HTML output
```

## Web Extension Traits

| Trait | Purpose | When It Runs |
|-------|---------|--------------|
| `ContentDataProvider` | Enrich content JSON after loading from DB | During `contents_to_json()` |
| `PageDataProvider` | Provide ALL template variables | Before template rendering |
| `ComponentRenderer` | Generate ALL HTML fragments | After PageDataProviders |
| `TemplateDataExtender` | Final modifications to template data | After ComponentRenderers |
| `PagePrerenderer` | Generate static pages at build time | During publish pipeline |
| `FrontmatterProcessor` | Parse custom frontmatter fields | During content ingestion |
| `RssFeedProvider` | Generate RSS feeds | During publish pipeline |
| `SitemapProvider` | Generate sitemap entries | During publish pipeline |

See [Web Traits](/documentation/extensions/web-traits/) for detailed trait documentation.

---

## PageDataProvider

PageDataProvider is the **primary** mechanism for providing template variables. Core only provides minimal data (`CONTENT`, `TOC_HTML`, `SLUG`). Your extension must provide everything else: `TITLE`, `DESCRIPTION`, `DATE`, `AUTHOR`, etc.

```rust
use systemprompt::extension::prelude::{PageContext, PageDataProvider};
use anyhow::Result;
use serde_json::{json, Value};

pub struct ContentPageDataProvider;

#[async_trait]
impl PageDataProvider for ContentPageDataProvider {
    fn provider_id(&self) -> &'static str {
        "content-page-data"
    }

    fn applies_to_pages(&self) -> Vec<String> {
        vec![]  // Empty = applies to all pages
    }

    fn priority(&self) -> u32 {
        100  // Default priority (lower = runs earlier)
    }

    async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
        let item = ctx.content_item().ok_or_else(|| anyhow!("No content item"))?;

        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let description = item.get("description")
            .or_else(|| item.get("excerpt"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        Ok(json!({
            "TITLE": title,
            "DESCRIPTION": description,
            "AUTHOR": item.get("author").and_then(|v| v.as_str()).unwrap_or(""),
        }))
    }
}
```

See [PageDataProvider](/documentation/extensions/web-traits/page-data-provider) for full reference.

---

## ComponentRenderer

ComponentRenderer generates HTML fragments that are inserted into template variables.

```rust
use systemprompt::template_provider::{ComponentContext, ComponentRenderer, RenderedComponent};

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
        vec![]  // Empty = applies to all content types
    }

    async fn render(&self, ctx: &ComponentContext<'_>) -> Result<RenderedComponent> {
        let items = ctx.all_items.unwrap_or(&[]);
        let html = items
            .iter()
            .map(|item| self.render_card(item))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(RenderedComponent::new("POSTS", html))
    }
}
```

See [ComponentRenderer](/documentation/extensions/web-traits/component-renderer) for full reference.

---

## ContentDataProvider

Enriches content items during `contents_to_json()`, before PageDataProviders run.

```rust
use systemprompt::extension::prelude::{ContentDataContext, ContentDataProvider};

pub struct DocsContentDataProvider;

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
        // Add computed fields, related content, etc.
        Ok(())
    }
}
```

See [ContentDataProvider](/documentation/extensions/web-traits/content-data-provider) for full reference.

---

## TemplateDataExtender

Runs after PageDataProviders and ComponentRenderers for final transformations.

```rust
use systemprompt::template_provider::{ExtenderContext, TemplateDataExtender};

pub struct CanonicalUrlExtender;

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
        data: &mut serde_json::Value,
    ) -> Result<()> {
        let slug = ctx.item.get("slug").and_then(|v| v.as_str()).unwrap_or("");
        if let Some(obj) = data.as_object_mut() {
            obj.insert("CANONICAL_PATH".to_string(), json!(format!("/{}", slug)));
        }
        Ok(())
    }
}
```

See [TemplateDataExtender](/documentation/extensions/web-traits/template-data-extender) for full reference.

---

## PagePrerenderer

Generate static HTML pages at build time for list pages, index pages, and other content that doesn't come from markdown files.

```rust
use systemprompt::template_provider::{PagePrepareContext, PagePrerenderer, PageRenderSpec};

pub struct BlogListPrerenderer;

#[async_trait]
impl PagePrerenderer for BlogListPrerenderer {
    fn page_type(&self) -> &str {
        "blog-list"
    }

    fn priority(&self) -> u32 {
        100
    }

    async fn prepare(&self, ctx: &PagePrepareContext<'_>) -> Result<Option<PageRenderSpec>> {
        let template_data = json!({
            "TITLE": "Blog",
            "DESCRIPTION": "Latest posts",
        });

        Ok(Some(PageRenderSpec::new(
            "blog-list",
            template_data,
            PathBuf::from("blog/index.html"),
        )))
    }
}
```

See [PagePrerenderer](/documentation/extensions/web-traits/page-prerenderer) for full reference.

---

## RequiredAssets

Register CSS and JS files:

```rust
fn declares_assets(&self) -> bool {
    true
}

fn required_assets(&self, paths: &dyn AssetPaths) -> Vec<AssetDefinition> {
    let storage_css = paths.storage_files().join("css");
    vec![
        AssetDefinition::css(storage_css.join("core/variables.css"), "css/core/variables.css"),
        AssetDefinition::css(storage_css.join("components/header.css"), "css/components/header.css"),
        AssetDefinition::js(paths.storage_files().join("js/main.js"), "js/main.js"),
    ]
}
```

Assets are copied from `storage/files/` to `web/dist/` during the publish pipeline.

See [Asset Declaration](/documentation/extensions/web-traits/asset-declaration) for full reference.

---

## Complete Extension Example

```rust
use systemprompt::extension::prelude::*;
use std::sync::Arc;

pub struct WebExtension {
    config: WebConfig,
}

impl Extension for WebExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "web",
            name: "Web Content & Navigation",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    fn page_data_providers(&self) -> Vec<Arc<dyn PageDataProvider>> {
        vec![
            Arc::new(ContentPageDataProvider),
            Arc::new(MetadataPageDataProvider::new(self.config.clone())),
        ]
    }

    fn component_renderers(&self) -> Vec<Arc<dyn ComponentRenderer>> {
        vec![
            Arc::new(ContentCardsRenderer),
            Arc::new(RelatedContentRenderer),
        ]
    }

    fn content_data_providers(&self) -> Vec<Arc<dyn ContentDataProvider>> {
        vec![Arc::new(DocsContentDataProvider)]
    }

    fn template_data_extenders(&self) -> Vec<Arc<dyn TemplateDataExtender>> {
        vec![Arc::new(CanonicalUrlExtender)]
    }

    fn page_prerenderers(&self) -> Vec<Arc<dyn PagePrerenderer>> {
        vec![Arc::new(BlogListPrerenderer)]
    }

    fn frontmatter_processors(&self) -> Vec<Arc<dyn FrontmatterProcessor>> {
        vec![Arc::new(CustomFieldProcessor)]
    }

    fn rss_feed_providers(&self) -> Vec<Arc<dyn RssFeedProvider>> {
        vec![Arc::new(BlogRssProvider)]
    }

    fn sitemap_providers(&self) -> Vec<Arc<dyn SitemapProvider>> {
        vec![Arc::new(ContentSitemapProvider)]
    }

    fn declares_assets(&self) -> bool {
        true
    }

    fn required_assets(&self, paths: &dyn AssetPaths) -> Vec<AssetDefinition> {
        vec![
            AssetDefinition::css(paths.storage_files().join("css/core/variables.css"), "css/core/variables.css"),
        ]
    }
}

register_extension!(WebExtension);
```

---

## Project Structure

```
extensions/web/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── extension.rs           # Trait implementations
│   ├── providers/
│   │   ├── mod.rs
│   │   ├── content_page.rs    # PageDataProvider
│   │   └── metadata.rs        # PageDataProvider
│   ├── components/
│   │   ├── mod.rs
│   │   ├── cards.rs           # ComponentRenderer
│   │   └── related.rs         # ComponentRenderer
│   ├── enrichers/
│   │   ├── mod.rs
│   │   └── docs.rs            # ContentDataProvider
│   ├── extenders/
│   │   ├── mod.rs
│   │   └── canonical_url.rs   # TemplateDataExtender
│   └── prerenderers/
│       ├── mod.rs
│       └── blog_list.rs       # PagePrerenderer
└── schema/

services/web/config/            # YAML configuration
├── navigation.yaml
├── homepage.yaml
└── features/

storage/files/css/              # CSS source files
└── ...
```