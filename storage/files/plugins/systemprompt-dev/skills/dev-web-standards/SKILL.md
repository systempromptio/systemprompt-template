---
name: "Web Standards"
description: "Extension system, content templates, page prerendering, and CSS management standards for systemprompt.io web development"
---

# Extension System Guide

Technical guide for extending systemprompt.io with custom templates, components, and data providers.

---

## Crate Dependencies

Extension implementations depend on `systemprompt-provider-contracts` for provider trait definitions:

```
systemprompt-provider-contracts
+-- LlmProvider, ToolProvider      <- AI/MCP providers
+-- Job, JobContext, JobResult     <- Background jobs
+-- ComponentRenderer              <- Template components
+-- TemplateDataExtender           <- Data extenders
+-- PageDataProvider               <- Page data providers
+-- TemplateProvider               <- Template definitions
```

Import via the extension prelude:

```rust
use systemprompt::extension::prelude::*;
```

The prelude re-exports all provider contracts from `systemprompt-provider-contracts`.

---

## Architecture Overview

```
+-----------------------------------------------------------------+
|                        Extension Project                         |
|                                                                  |
|  +-----------------+  +-----------------+  +-------------------+ |
|  | Templates       |  | Components      |  | Data Providers    | |
|  | services/web/   |  | ComponentRender |  | PageDataProvider  | |
|  | templates/      |  | trait impl      |  | TemplateDataExt   | |
|  +--------+--------+  +--------+--------+  +---------+---------+ |
|           |                    |                      |          |
|           +--------------------+----------------------+          |
|                                |                                 |
|                    +-----------v------------+                    |
|                    | Extension trait impl   |                    |
|                    | register_extension!()  |                    |
|                    +-----------+------------+                    |
+-------------------------------|----------------------------------+
                                | inventory collects at compile time
                    +-----------|------------+
                    | TemplateRegistry       |
                    | - providers            |
                    | - loaders              |
                    | - components           |
                    | - page_providers       |
                    | - extenders            |
                    +------------------------+
```

---

## Template System

### Directory Structure

Extensions define templates in `services/web/templates/`:

```
services/web/
  templates/
    homepage.html          <- Homepage template
    blog-post.html         <- Content type template
    blog-list.html         <- Parent route template
    partials/
      header.html          <- Shared partials
      footer.html
  web.yaml                 <- Template configuration
```

### Template Definition

Templates are discovered via `template.yaml` files:

```yaml
name: blog
priority: 500
content_types:
  - blog
  - article
source:
  type: file
  path: blog-post.html
```

| Field | Purpose |
|-------|---------|
| `name` | Template identifier |
| `priority` | Lower wins (500 = extension, 1000 = core default) |
| `content_types` | Content types this template handles |
| `source.path` | Path relative to templates directory |

### Template Variables

Templates receive data via Handlebars context:

```handlebars
<html>
<head>
    <title>{{title}}</title>
    <meta name="description" content="{{description}}">
    <link rel="stylesheet" href="{{CSS_BASE_PATH}}/main.css">
</head>
<body>
    <header>
        <img src="{{LOGO_PATH}}" alt="{{ORG_NAME}}">
    </header>

    <main>
        {{{CONTENT_HTML}}}
    </main>

    <aside>
        {{{POPULAR_ITEMS_HTML}}}
    </aside>

    <footer>
        {{{FOOTER_NAV}}}
    </footer>

    <script src="{{JS_BASE_PATH}}/main.js"></script>
</body>
</html>
```

### Standard Variables

| Variable | Source | Description |
|----------|--------|-------------|
| `site` | web.yaml | Full site configuration |
| `title` | Content item | Page title |
| `description` | Content item | Meta description |
| `CONTENT_HTML` | Rendered markdown | Main content body |
| `ORG_NAME` | content.yaml | Organization name |
| `ORG_URL` | content.yaml | Organization URL |
| `LOGO_PATH` | web.yaml | Logo file path |
| `FAVICON_PATH` | web.yaml | Favicon path |
| `JS_BASE_PATH` | Generated | JavaScript directory |
| `CSS_BASE_PATH` | Generated | CSS directory |
| `FOOTER_NAV` | Generated | Footer navigation HTML |

---

## Component Renderers

Components inject dynamic HTML into templates.

### Implementing ComponentRenderer

```rust
use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use systemprompt::extension::prelude::*;

pub struct PopularItemsComponent;

#[async_trait]
impl ComponentRenderer for PopularItemsComponent {
    fn component_id(&self) -> &str {
        "popular-items"
    }

    fn variable_name(&self) -> &str {
        "POPULAR_ITEMS_HTML"
    }

    fn applies_to(&self) -> Vec<String> {
        vec!["blog".to_string(), "homepage".to_string()]
    }

    async fn render(&self, ctx: &ComponentContext<'_>) -> Result<RenderedComponent> {
        let html = match (ctx.all_items, ctx.popular_ids) {
            (Some(items), Some(popular_ids)) => {
                let popular: Vec<_> = items
                    .iter()
                    .filter(|item| {
                        item.get("id")
                            .and_then(|id| id.as_str())
                            .map(|id| popular_ids.contains(&id.to_string()))
                            .unwrap_or(false)
                    })
                    .take(5)
                    .collect();

                render_popular_list(&popular)
            }
            _ => String::new(),
        };

        Ok(RenderedComponent::new(self.variable_name(), html))
    }

    fn priority(&self) -> u32 {
        100
    }
}
```

### ComponentContext

```rust
pub struct ComponentContext<'a> {
    pub web_config: &'a serde_yaml::Value,
    pub item: Option<&'a Value>,
    pub all_items: Option<&'a [Value]>,
    pub popular_ids: Option<&'a [String]>,
}
```

| Field | Available For | Description |
|-------|---------------|-------------|
| `web_config` | All pages | Site configuration |
| `item` | Content pages | Current content item |
| `all_items` | Content pages | All items in source |
| `popular_ids` | Content pages | IDs of popular content |

Use `ComponentContext::for_page()` for static pages (homepage, about).
Use `ComponentContext::for_content()` for content pages (blog posts).

---

## Page Data Providers

Providers inject dynamic data into static pages.

### Implementing PageDataProvider

```rust
use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use systemprompt::extension::prelude::*;
use systemprompt_database::DbPool;

pub struct FeaturedPostsProvider;

#[async_trait]
impl PageDataProvider for FeaturedPostsProvider {
    fn provider_id(&self) -> &str {
        "featured-posts"
    }

    fn applies_to_pages(&self) -> Vec<String> {
        vec!["homepage".to_string()]
    }

    async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<serde_json::Value> {
        let db_pool = ctx.db_pool::<DbPool>()
            .ok_or_else(|| anyhow::anyhow!("Database pool not available"))?;

        let repo = ContentRepository::new(db_pool)?;
        let featured = repo.get_featured(3).await?;

        Ok(serde_json::json!({
            "featured_posts": featured
                .iter()
                .map(|p| serde_json::json!({
                    "title": p.title,
                    "slug": p.slug,
                    "description": p.description
                }))
                .collect::<Vec<_>>()
        }))
    }

    fn priority(&self) -> u32 {
        100
    }
}
```

### PageContext

```rust
pub struct PageContext<'a> {
    pub page_type: &'a str,
    pub web_config: &'a serde_yaml::Value,
}

impl PageContext {
    pub fn db_pool<T: 'static>(&self) -> Option<&T>;
}
```

The `db_pool()` method uses type erasure to access the database pool.

---

## Template Data Extenders

Extenders modify template data before rendering.

### Implementing TemplateDataExtender

```rust
use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use systemprompt::extension::prelude::*;

pub struct RelatedPostsExtender;

#[async_trait]
impl TemplateDataExtender for RelatedPostsExtender {
    fn extender_id(&self) -> &str {
        "related-posts"
    }

    fn applies_to(&self) -> Vec<String> {
        vec!["blog".to_string()]
    }

    async fn extend(&self, ctx: &ExtenderContext<'_>) -> Result<ExtendedData> {
        let category = ctx.item
            .get("category_id")
            .and_then(|v| v.as_str());

        let related = match category {
            Some(cat) => find_related_posts(ctx.all_items, cat, ctx.item),
            None => vec![],
        };

        Ok(ExtendedData::new()
            .with_value("related_posts", serde_json::to_value(&related)?))
    }

    fn priority(&self) -> u32 {
        100
    }
}
```

---

## Registering Extensions

### Extension Implementation

```rust
use std::sync::Arc;
use systemprompt::extension::prelude::*;

pub struct BlogExtension;

impl Extension for BlogExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "blog",
            name: "Blog Extension",
            version: "0.1.0",
        }
    }

    fn component_renderers(&self) -> Vec<Arc<dyn ComponentRenderer>> {
        vec![
            Arc::new(PopularItemsComponent),
            Arc::new(TableOfContentsComponent),
        ]
    }

    fn template_data_extenders(&self) -> Vec<Arc<dyn TemplateDataExtender>> {
        vec![
            Arc::new(RelatedPostsExtender),
        ]
    }

    fn page_data_providers(&self) -> Vec<Arc<dyn PageDataProvider>> {
        vec![
            Arc::new(FeaturedPostsProvider),
        ]
    }
}

register_extension!(BlogExtension);
```

### Extension Discovery

Extensions register via the `inventory` crate. At runtime:

```rust
let extensions = ExtensionRegistry::discover();
for ext in extensions.extensions() {
    for component in ext.component_renderers() {
        registry_builder = registry_builder.with_component(component);
    }
    for extender in ext.template_data_extenders() {
        registry_builder = registry_builder.with_extender(extender);
    }
    for provider in ext.page_data_providers() {
        registry_builder = registry_builder.with_page_provider(provider);
    }
}
```

---

## Template Priority System

Templates resolve by priority (lower wins):

| Priority | Source |
|----------|--------|
| 500 | Extension templates |
| 1000 | Core default templates |

This allows extensions to override core templates.

---

## Static Assets

### Scripts and Styles

Place assets in `services/web/`:

```
services/web/
  js/
    main.js
    analytics.js
  css/
    main.css
    theme.css
  assets/
    logo.svg
    favicon.ico
```

Reference in templates using base path variables:

```handlebars
<link rel="stylesheet" href="{{CSS_BASE_PATH}}/main.css">
<script src="{{JS_BASE_PATH}}/main.js"></script>
<img src="/assets/logo.svg" alt="Logo">
```

---

## Best Practices

### Template Organization

| Pattern | Description |
|---------|-------------|
| One template per content type | `blog-post.html`, `product.html`, `event.html` |
| Shared partials in `partials/` | `header.html`, `footer.html`, `sidebar.html` |
| Index templates with `-list` suffix | `blog-list.html` for `/blog` route |

### Component Design

| Pattern | Description |
|---------|-------------|
| Single responsibility | One component per feature |
| Graceful degradation | Return empty string if data unavailable |
| Priority ordering | Use priority to control render order |

### Error Handling

| Pattern | Description |
|---------|-------------|
| Log and continue | Components log errors but don't fail page |
| Propagate critical errors | Missing required data should fail |
| Use `context()` for error messages | Provide context for debugging |

---

# Content Templates

## Key Concepts

### Content Kind (Frontmatter)

Each markdown content file has a `kind` field in its frontmatter that identifies what type of content it is:

```yaml
---
title: My Blog Post
kind: article
date: 2024-01-15
image: /images/blog/my-post.jpg
---
```

Common kind values:
- `article` -- Blog posts, news articles
- `tutorial` -- Step-by-step guides
- `legal` -- Privacy policies, terms of service
- `page` -- Generic static pages
- `homepage` -- Homepage content

### Content Type (Database)

When content is ingested into the database, the `kind` field becomes the `content_type` field. This is the value used during prerendering to find the appropriate template.

### Allowed Content Types (Source Config)

In `content.yaml`, each content source can specify which content types it accepts:

```yaml
content_sources:
  blog:
    path: services/content/blog
    source_id: blog
    category_id: blog
    enabled: true
    allowed_content_types:
      - article
      - tutorial
```

### Template Content Types

Templates declare which content types they can render via `templates.yaml`:

```yaml
templates:
  blog-post:
    content_types:
      - article
      - tutorial
  legal-page:
    content_types:
      - legal
      - page
  homepage:
    content_types:
      - homepage
```

---

## How Templates Are Matched

During prerendering, the system:

1. Reads content from the database with its `content_type`
2. Calls `template_registry.find_template_for_content_type(content_type)`
3. Returns the first template that lists that content_type

```
Content (content_type: "article")
        |
        v
TemplateRegistry.find_template_for_content_type("article")
        |
        v
Scans templates for one with content_types containing "article"
        |
        v
Finds "blog-post" template -> Renders content
```

---

## Template Priority

When multiple templates can handle the same content type, priority determines which is used:

1. **Extension templates** (priority ~100) -- Project-specific templates
2. **Embedded defaults** (priority 1000) -- Built-in fallback templates

Lower priority numbers win.

---

## Configuration Files Reference

### content.yaml

Defines content sources and their allowed types:

```yaml
content_sources:
  blog:
    path: services/content/blog
    source_id: blog
    category_id: blog
    enabled: true
    description: Blog posts
    allowed_content_types:
      - article
      - tutorial
    sitemap:
      enabled: true
      url_pattern: /blog/{slug}
      changefreq: weekly
      priority: 0.8
```

### templates.yaml

Defines available templates and which content types they handle:

```yaml
templates:
  blog-post:
    content_types:
      - article
      - tutorial
  homepage:
    content_types:
      - homepage
```

### web.yaml (Branding)

Required branding fields:

```yaml
branding:
  copyright: "2024 Your Company. All rights reserved."
  twitter_handle: "@yourhandle"
  display_sitename: true
  favicon: /favicon.ico
  logo:
    primary:
      svg: /images/logo.svg
```

---

## Creating a New Content Type

1. **Define the kind in frontmatter**:
   ```yaml
   ---
   title: New Case Study
   kind: case-study
   ---
   ```

2. **Add to allowed_content_types in content.yaml**

3. **Create a template in templates.yaml**

4. **Create the template HTML file**

---

## Common Kind/Type Mappings

| Frontmatter `kind` | Use Case | Typical Template |
|-------------------|----------|------------------|
| `article` | Blog posts, news | blog-post |
| `tutorial` | How-to guides | blog-post or tutorial |
| `legal` | Privacy, terms | legal-page |
| `page` | Static pages | legal-page or page |
| `homepage` | Site homepage | homepage |
| `landing` | Marketing pages | landing-page |
| `documentation` | Docs, references | docs-page |

---

# Page Prerenderer Implementation Guide

## Overview

Page prerendering is fully extension-driven. The core provides:
- `PagePrerenderer` trait for extensions to implement
- `PagePrepareContext` with access to configuration and data
- Generic engine that discovers and executes prerenderers

Extensions own:
- Page type definitions
- Base data construction
- Template selection
- Output path decisions

## Key Types

### PagePrerenderer Trait

```rust
#[async_trait]
pub trait PagePrerenderer: Send + Sync {
    fn page_type(&self) -> &str;

    fn priority(&self) -> u32 {
        100
    }

    async fn prepare(&self, ctx: &PagePrepareContext<'_>) -> Result<Option<PageRenderSpec>>;
}
```

### PagePrepareContext

```rust
pub struct PagePrepareContext<'a> {
    pub web_config: &'a WebConfig,
    content_config: &'a (dyn Any + Send + Sync),
    db_pool: &'a (dyn Any + Send + Sync),
    dist_dir: &'a Path,
}

impl<'a> PagePrepareContext<'a> {
    pub fn content_config<T: 'static>(&self) -> Option<&T>
    pub fn db_pool<T: 'static>(&self) -> Option<&T>
    pub fn dist_dir(&self) -> &Path
}
```

### PageRenderSpec

```rust
pub struct PageRenderSpec {
    pub template_name: String,
    pub base_data: serde_json::Value,
    pub output_path: PathBuf,
}
```

## Implementing a Page Prerenderer

### Step 1: Create the Prerenderer

```rust
use std::path::PathBuf;
use anyhow::Result;
use async_trait::async_trait;
use systemprompt_models::ContentConfigRaw;
use systemprompt_provider_contracts::{
    PagePrepareContext, PagePrerenderer, PageRenderSpec,
};

const PAGE_TYPE: &str = "docs-index";
const TEMPLATE_NAME: &str = "docs-index";
const OUTPUT_FILE: &str = "docs/index.html";

#[derive(Debug, Clone, Copy, Default)]
pub struct DocsIndexPrerenderer;

#[async_trait]
impl PagePrerenderer for DocsIndexPrerenderer {
    fn page_type(&self) -> &str {
        PAGE_TYPE
    }

    fn priority(&self) -> u32 {
        100
    }

    async fn prepare(&self, ctx: &PagePrepareContext<'_>) -> Result<Option<PageRenderSpec>> {
        let base_data = serde_json::json!({
            "site": ctx.web_config,
            "page_title": "Documentation"
        });

        Ok(Some(PageRenderSpec::new(
            TEMPLATE_NAME,
            base_data,
            PathBuf::from(OUTPUT_FILE),
        )))
    }
}
```

### Step 2: Register with Extension

```rust
use std::sync::Arc;
use systemprompt_extension::prelude::*;

impl Extension for MyExtension {
    fn page_prerenderers(&self) -> Vec<Arc<dyn PagePrerenderer>> {
        vec![Arc::new(DocsIndexPrerenderer)]
    }
}
```

## Data Flow

1. **Discovery**: `ExtensionRegistry::discover()` finds all extensions
2. **Collection**: Extensions return prerenderers via `page_prerenderers()`
3. **Registration**: Prerenderers added to `TemplateRegistry`
4. **Execution**: Engine iterates prerenderers in priority order
5. **Preparation**: Each prerenderer's `prepare()` builds the render spec
6. **Enhancement**: Engine collects `PageDataProvider`s and `ComponentRenderer`s for the page type
7. **Merge**: Provider data merged with base data from spec
8. **Render**: Template rendered with merged data
9. **Output**: HTML written to spec's output path

## Priority

Lower priority values indicate higher importance and execute first.

| Priority | Use Case |
|----------|----------|
| 0-49 | Critical -- overrides defaults |
| 50-99 | Core application pages |
| 100 | Default (fallback, easily overridden) |
| 101+ | Low priority/optional |

## Error Handling

- Return `Ok(None)` to skip rendering (template not found, feature disabled)
- Return `Err(...)` for actual failures
- Engine logs warnings for missing templates but continues with other pages

## CLI Usage

```bash
systemprompt core content publish --step pages
systemprompt core content publish
```

## Files Reference

| File | Purpose |
|------|---------|
| `crates/shared/provider-contracts/src/page_prerenderer.rs` | Trait definitions |
| `crates/shared/extension/src/lib.rs` | Extension trait with `page_prerenderers()` |
| `crates/domain/templates/src/registry.rs` | Registry for prerenderers |
| `crates/app/generator/src/prerender/engine.rs` | Execution engine |
| `crates/domain/content/src/homepage_prerenderer.rs` | Default homepage implementation |

---

# CSS Management Rules

## File Locations

All CSS files go in `storage/files/css/` and must be registered in `extensions/web/src/extension.rs`.

```
storage/files/css/               <- CSS SOURCE (put files here)
extensions/web/src/extension.rs  <- REGISTER here in required_assets()
web/dist/css/                    <- OUTPUT (generated, never edit)
```

**NEVER put CSS in `extensions/*/assets/css/`.**

## Adding a New CSS File

1. Create the file in `storage/files/css/`
2. Register it in `extensions/web/src/extension.rs` via the `required_assets()` method (which delegates to `web_assets()` in `assets.rs`)
3. Run: `just build && systemprompt infra jobs run copy_extension_assets`

## Registration Pattern

The `WebExtension` implements `Extension::required_assets()` which calls `web_assets(paths)` from `crate::assets`. New CSS files must be added to the asset definitions returned by this function.

## Build & Deploy

```bash
just build                                              # Build all extensions
systemprompt infra jobs run copy_extension_assets        # Copy assets to dist/
systemprompt web validate                               # Validate output
```

## Key Files

| File | Purpose |
|------|---------|
| `storage/files/css/` | CSS source directory |
| `extensions/web/src/extension.rs` | Extension registration with `required_assets()` |
| `extensions/web/src/assets.rs` | Asset definitions (`web_assets` function) |
| `web/dist/css/` | Generated output (never edit directly) |
