---
title: "Web Prerendering"
description: "Generate static HTML pages at build time using PagePrerenderer, including list pages with custom card rendering."
author: "SystemPrompt"
slug: "build-web-prerender"
keywords: "prerender, static generation, PagePrerenderer, list pages, cards"
kind: "playbook"
published_at: "2026-01-31"
tags:
  - build
  - prerender
  - templates
after_reading_this:
  - "Understand when to use PagePrerenderer vs PageDataProvider"
  - "Create static page prerenderers for list pages"
  - "Render content cards with custom data attributes"
  - "Register prerenderers in extension.rs"
related_playbooks:
  - title: "Web Ingestion"
    url: "/playbooks/build-web-ingestion"
  - title: "Web Templates"
    url: "/playbooks/build-web-templates"
related_code:
  - title: "DocsIndexPrerenderer"
    url: "https://github.com/systempromptio/systemprompt-web/blob/main/extensions/web/src/docs/prerenderer.rs"
  - title: "DocsPageDataProvider"
    url: "https://github.com/systempromptio/systemprompt-web/blob/main/extensions/web/src/docs/provider.rs"
  - title: "Card Templates"
    url: "https://github.com/systempromptio/systemprompt-web/blob/main/services/web/templates/partials/content-card-image.html"
---

# Web Prerendering

Generate static HTML pages at build time using PagePrerenderer, including list pages with custom card rendering.

---

## PagePrerenderer vs PageDataProvider

| Aspect | PageDataProvider | PagePrerenderer |
|--------|------------------|-----------------|
| **When** | Runtime (per request) | Build time (once) |
| **Purpose** | Inject data into existing pages | Generate entire pages |
| **Use for** | Navigation, metadata | List pages, sitemaps |
| **Output** | JSON data for templates | Complete HTML files |

**Use PagePrerenderer when:**
- Generating list pages (blog index, docs index)
- Creating static pages from database content
- Pre-rendering pages that don't change per-request

---

## PagePrerenderer Trait

```rust
#[async_trait]
pub trait PagePrerenderer: Send + Sync {
    /// Unique identifier for this prerenderer
    fn page_type(&self) -> &'static str;

    /// Execution priority (lower = earlier)
    fn priority(&self) -> u32;

    /// Prepare page for rendering
    /// Returns None to skip, Some(spec) to render
    async fn prepare(&self, ctx: &PagePrepareContext<'_>) -> Result<Option<PageRenderSpec>>;
}
```

### PageRenderSpec

```rust
pub struct PageRenderSpec {
    /// Template name (from templates.yaml)
    pub template: String,

    /// Template variables (JSON object)
    pub data: serde_json::Value,

    /// Output path relative to web/dist/
    pub output_path: PathBuf,
}

impl PageRenderSpec {
    pub fn new(template: &str, data: Value, output_path: PathBuf) -> Self {
        Self {
            template: template.to_string(),
            data,
            output_path,
        }
    }
}
```

---

## Example: DocsIndexPrerenderer

**File**: `extensions/web/src/docs/prerenderer.rs`

This prerenderer generates the `/documentation/` index page with child documentation cards.

```rust
pub struct DocsIndexPrerenderer;

#[async_trait]
impl PagePrerenderer for DocsIndexPrerenderer {
    fn page_type(&self) -> &'static str {
        "docs-index-prerenderer"
    }

    fn priority(&self) -> u32 {
        100  // High priority to override defaults
    }

    async fn prepare(&self, ctx: &PagePrepareContext<'_>) -> Result<Option<PageRenderSpec>> {
        // 1. Get database connection
        let db = ctx.db_pool::<Arc<Database>>()?;
        let pool = db.pool()?;

        // 2. Fetch index content
        let row = sqlx::query!(
            r#"
            SELECT id, title, description, body, author, updated_at,
                   COALESCE(after_reading_this, '[]'::jsonb) as "after_reading_this!",
                   COALESCE(related_playbooks, '[]'::jsonb) as "related_playbooks!"
            FROM markdown_content
            WHERE source_id = 'documentation' AND slug = ''
            "#,
        )
        .fetch_optional(&*pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);  // No index content, skip
        };

        // 3. Fetch children
        let children = DocsContentDataProvider::new()
            .get_children_static(&pool, "documentation", "")
            .await;

        // 4. Build template data
        let mut template_data = json!({
            "TITLE": row.title,
            "DESCRIPTION": row.description,
            "CONTENT": markdown_to_html(&row.body),
        });

        // 5. Render children as HTML cards
        if !children.is_empty() {
            let children_html = render_children_cards(&children);
            template_data["CHILDREN"] = json!(children_html);
        }

        // 6. Return render spec
        let output_path = PathBuf::from("documentation/index.html");

        Ok(Some(PageRenderSpec::new(
            "docs-list",      // Template name
            template_data,    // Variables
            output_path,      // Output file
        )))
    }
}
```

---

## Rendering Cards

### Card Template

**File**: `services/web/templates/partials/content-card-image.html`

```handlebars
<a href="{{URL}}" class="blog-card-link" data-category="{{CATEGORY}}">
  <article class="blog-card content-card content-card--{{KIND}}" data-category="{{CATEGORY}}">
    {{#if IMAGE}}
    <div class="card-image">
      <img src="{{IMAGE}}" alt="{{TITLE}}" loading="lazy" />
    </div>
    {{/if}}
    <div class="card-content">
      {{#if CATEGORY}}
      <span class="card-category card-category--{{CATEGORY}}">{{CATEGORY}}</span>
      {{/if}}
      <h2 class="card-title">{{TITLE}}</h2>
      <p class="card-description">{{DESCRIPTION}}</p>
      <div class="meta">
        {{#if DATE}}
        <time datetime="{{DATE_ISO}}" class="meta-date">{{DATE}}</time>
        {{/if}}
      </div>
    </div>
  </article>
</a>
```

### Rendering Cards in Rust

```rust
fn render_blog_cards(posts: &[BlogPost]) -> String {
    posts
        .iter()
        .map(|post| {
            format!(
                r#"<a href="/blog/{}" class="blog-card-link" data-category="{}">
  <article class="blog-card content-card" data-category="{}">
    <div class="card-image">
      <img src="{}" alt="{}" loading="lazy" />
    </div>
    <div class="card-content">
      <span class="card-category card-category--{}">{}</span>
      <h2 class="card-title">{}</h2>
      <p class="card-description">{}</p>
      <div class="meta">
        <time class="card-date">{}</time>
      </div>
    </div>
  </article>
</a>"#,
                post.slug,
                post.category.as_deref().unwrap_or(""),
                post.category.as_deref().unwrap_or(""),
                post.image.as_deref().unwrap_or("/files/images/blog/placeholder.svg"),
                html_escape(&post.title),
                post.category.as_deref().unwrap_or(""),
                post.category.as_deref().unwrap_or(""),
                html_escape(&post.title),
                html_escape(&post.description),
                post.published_at.format("%B %d, %Y"),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
```

---

## Creating a Blog List Prerenderer

### Step 1: Create the Prerenderer

**File**: `extensions/web/src/blog/prerenderer.rs`

```rust
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use systemprompt::database::Database;
use systemprompt::template_provider::{PagePrepareContext, PagePrerenderer, PageRenderSpec};

pub struct BlogListPrerenderer;

impl BlogListPrerenderer {
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PagePrerenderer for BlogListPrerenderer {
    fn page_type(&self) -> &'static str {
        "blog-list-prerenderer"
    }

    fn priority(&self) -> u32 {
        100
    }

    async fn prepare(&self, ctx: &PagePrepareContext<'_>) -> Result<Option<PageRenderSpec>> {
        let db = ctx.db_pool::<Arc<Database>>().ok_or_else(|| {
            anyhow::anyhow!("No database in context")
        })?;
        let pool = db.pool().ok_or_else(|| {
            anyhow::anyhow!("Pool not initialized")
        })?;

        // Fetch all blog posts
        let posts = sqlx::query!(
            r#"
            SELECT slug, title, description, image, category, published_at
            FROM markdown_content
            WHERE source_id = 'blog' AND public = true
            ORDER BY published_at DESC
            "#
        )
        .fetch_all(&*pool)
        .await?;

        // Render cards with category
        let cards_html = render_blog_cards(&posts);

        // Build template data
        let template_data = json!({
            "POSTS": cards_html,
            "site": ctx.web_config,
        });

        let output_path = PathBuf::from("blog/index.html");

        Ok(Some(PageRenderSpec::new(
            "blog-list",
            template_data,
            output_path,
        )))
    }
}
```

### Step 2: Create Module

**File**: `extensions/web/src/blog/mod.rs`

```rust
mod prerenderer;

pub use prerenderer::BlogListPrerenderer;
```

### Step 3: Export Module

**File**: `extensions/web/src/lib.rs`

```rust
pub mod blog;
pub use blog::BlogListPrerenderer;
```

### Step 4: Register Prerenderer

**File**: `extensions/web/src/extension.rs`

```rust
impl Extension for WebExtension {
    fn page_prerenderers(&self) -> Vec<Arc<dyn PagePrerenderer>> {
        vec![
            Arc::new(DocsIndexPrerenderer::new()),
            Arc::new(BlogListPrerenderer::new()),  // Add this
        ]
    }
}
```

---

## Template Variables

Common variables passed to list templates:

| Variable | Type | Description |
|----------|------|-------------|
| `POSTS` | String | Pre-rendered HTML cards |
| `CHILDREN` | String | Pre-rendered child cards |
| `TITLE` | String | Page title |
| `DESCRIPTION` | String | Page description |
| `CONTENT` | String | Rendered markdown body |
| `site` | Object | Site configuration |

### Using in Templates

```handlebars
<main class="blog-list">
  <header class="page-header">
    <h1>{{TITLE}}</h1>
    <p>{{DESCRIPTION}}</p>
  </header>

  <div class="blog-grid" id="blog-grid">
    {{{POSTS}}}
  </div>
</main>
```

Note: Use triple braces `{{{POSTS}}}` for pre-rendered HTML (no escaping).

---

## Filtering with data-category

The `data-category` attribute enables client-side filtering:

### HTML Structure

```html
<a href="/blog/my-post" class="blog-card-link" data-category="announcement">
  <article class="blog-card" data-category="announcement">
    ...
  </article>
</a>
```

### CSS Filtering

```css
/* Hide cards that don't match filter */
.blog-grid[data-filter="announcement"] a:not([data-category="announcement"]),
.blog-grid[data-filter="guide"] a:not([data-category="guide"]),
.blog-grid[data-filter="article"] a:not([data-category="article"]) {
  display: none;
}
```

### JavaScript Filter

```javascript
function applyFilter(filter) {
  const grid = document.getElementById('blog-grid');
  grid.dataset.filter = filter || '';
}
```

---

## CLI Commands

```bash
# Run prerendering
systemprompt infra jobs run content_prerender

# Full publish pipeline
systemprompt infra jobs run publish_pipeline

# Check generated output
ls -la web/dist/blog/
cat web/dist/blog/index.html | grep "data-category"
```

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Page not generated | Check `prepare()` returns `Some(spec)` |
| Wrong template | Verify template name in `PageRenderSpec` |
| Missing variables | Add to template_data JSON |
| Cards missing data | Check database query includes all fields |
| Filtering not working | Verify `data-category` in rendered HTML |

---

## Quick Reference

| Task | Location |
|------|----------|
| Create prerenderer | `extensions/web/src/*/prerenderer.rs` |
| Register prerenderer | `extension.rs` → `page_prerenderers()` |
| Card template | `services/web/templates/partials/content-card-image.html` |
| List template | `services/web/templates/*-list.html` |
| Run prerender | `systemprompt infra jobs run content_prerender` |
