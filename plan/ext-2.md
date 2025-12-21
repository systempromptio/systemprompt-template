# Phase 2: Blog Extension - REVISED (Static Content Generator)

**Status**: REQUIRES CORRECTION - Original implementation was incorrect

**Problem**: The blog extension was built as a JSON REST API (`/api/v1/content`), but it should be a **Static Content Generator (SCG)** that produces HTML files served at `/blog/{slug}`.

**Evidence**: Accessing `http://localhost:8080/blog` returns 404 because no such route exists - the API was mounted at `/api/v1/content`.

---

## CORRECTED ARCHITECTURE

### How It Should Work (Reference: `../systemprompt-blog`)

The blog should integrate with **`systemprompt-core/crates/app/generator`** which provides:

1. **PublishContentJob** - Orchestrates the full pipeline:
   - `optimize_images()` → Convert to WebP
   - `prerender_content()` → Generate HTML from Handlebars templates
   - `generate_sitemap()` → Create sitemap.xml

2. **Template Engine** - Handlebars-based with templates:
   - `blog-post.html` - Single blog post
   - `blog-list.html` - Blog index page

3. **Output Structure**:
```
dist/
├── blog/
│   ├── index.html           # List page
│   ├── my-post/
│   │   └── index.html       # Post page
│   └── another-post/
│       └── index.html
└── sitemap.xml
```

### What Was Built (Wrong)

| Aspect | Built (Wrong) | Should Be |
|--------|---------------|-----------|
| Output | JSON at `/api/v1/content` | HTML at `/blog/{slug}` |
| Runtime | Always-on API server | Build-time generation |
| Templates | None | Handlebars HTML |
| SEO | None | Meta tags, structured data, sitemap |

---

## CORRECTION PLAN

### Step 1: Remove API Layer

Delete or disable the JSON API handlers:
- `src/api/handlers/content.rs` - Returns JSON, not needed
- `src/api/mod.rs` - Router mounts API endpoints

**Keep**:
- `src/models/` - Content models are reusable
- `src/services/ingestion.rs` - Markdown parsing is good
- `src/repository/content.rs` - DB operations needed for SCG
- `schema/*.sql` - Database tables used by core SCG

### Step 2: Add Templates

Create `services/web/templates/`:

**blog-post.html**:
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <title>{{TITLE}} | {{ORG_NAME}}</title>
    <meta name="description" content="{{DESCRIPTION}}">
    <meta property="og:title" content="{{TITLE}}">
    <meta property="og:image" content="{{IMAGE}}">
</head>
<body>
    <article>
        <h1>{{TITLE}}</h1>
        <p class="meta">By {{AUTHOR}} on {{DATE}}</p>
        {{{CONTENT}}}
    </article>
</body>
</html>
```

**blog-list.html**:
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <title>{{BLOG_NAME}} | {{ORG_NAME}}</title>
</head>
<body>
    <h1>{{BLOG_NAME}}</h1>
    {{{POSTS}}}
</body>
</html>
```

### Step 3: Update Content Config

Update `services/content/config.yml`:

```yaml
content_sources:
  blog:
    path: "./services/content/blog"
    source_id: "blog"
    category_id: "blog"
    enabled: true
    allowed_content_types:
      - article
      - tutorial
    sitemap:
      enabled: true
      url_pattern: "/blog/{slug}"
      priority: 0.8
      changefreq: "weekly"
      parent_route:
        enabled: true
        url: "/blog"
        priority: 0.9
        changefreq: "daily"

metadata:
  default_author: "SystemPrompt"
  language: "en"
  structured_data:
    organization:
      name: "SystemPrompt"
      url: "https://example.com"
```

### Step 4: Update Extension

Simplify `extension.rs` - remove API router, keep schemas and ingestion job:

```rust
impl BlogExtension {
    pub fn schemas() -> Vec<(&'static str, &'static str)> {
        // Keep database schemas
    }

    pub fn ingestion_job() -> ContentIngestionJob {
        // Keep ingestion
    }

    // REMOVE: router() - no API needed
    // REMOVE: base_path() - not an API
}
```

### Step 5: Update Main Application

Modify `src/main.rs`:

```rust
// REMOVE:
// app = app.nest("/api/v1/content", blog_router);

// ADD static file serving:
use tower_http::services::ServeDir;
app = app.nest_service("/", ServeDir::new("dist"));

// ADD: Run PublishContentJob on startup or via scheduler
```

### Step 6: Integrate with Core SCG

The core's `PublishContentJob` already handles:
- Reading content from database
- Rendering markdown to HTML
- Applying Handlebars templates
- Writing to `dist/` directory
- Generating sitemap

The extension just needs to:
1. Provide content via ingestion (already works)
2. Provide templates in correct location
3. Configure sitemap settings

---

## FILES TO MODIFY

| File | Action |
|------|--------|
| `extensions/blog/src/api/` | Remove or keep only redirect router |
| `extensions/blog/src/extension.rs` | Remove router methods |
| `src/main.rs` | Remove API mounting, add static serving |
| `services/web/templates/blog-post.html` | Create |
| `services/web/templates/blog-list.html` | Create |
| `services/content/config.yml` | Update with sitemap config |

---

## VERIFICATION

After changes:
1. Run content ingestion
2. Run `PublishContentJob`
3. Check `dist/blog/` for HTML files
4. Start server with static file serving
5. Access `http://localhost:8080/blog` → HTML list page
6. Access `http://localhost:8080/blog/{slug}` → HTML post page

---

## ORIGINAL PLAN (FOR REFERENCE)

The original plan below was executed but resulted in incorrect architecture.
It built a JSON API when a static site generator was needed.

---

## 1. Current State Analysis (Original)

### 1.1 Location in Core

```
/var/www/html/systemprompt-core/crates/domain/content/
├── Cargo.toml                          # Package: systemprompt-core-blog
├── schema/
│   ├── markdown_content.sql            # Main content table
│   ├── markdown_categories.sql         # Categories
│   ├── campaign_links.sql              # Link tracking
│   ├── link_clicks.sql                 # Click analytics
│   ├── link_analytics_views.sql        # Analytics views
│   ├── content_performance_metrics.sql # Performance metrics
│   └── markdown_fts.sql                # Full-text search
└── src/
    ├── lib.rs                          # Public API exports
    ├── error.rs                        # BlogError type
    ├── analytics/                      # Link analytics module
    │   ├── mod.rs
    │   ├── repository.rs
    │   └── service.rs
    ├── api/
    │   ├── mod.rs
    │   └── routes/
    │       ├── mod.rs                  # Router + route exports
    │       ├── blog.rs                 # Content CRUD handlers
    │       ├── query.rs                # Search/query handlers
    │       └── links/
    │           ├── mod.rs
    │           ├── handlers.rs         # Link handlers
    │           └── types.rs            # Request/response types
    ├── jobs/
    │   ├── mod.rs
    │   └── content_ingestion.rs        # ContentIngestionJob
    ├── models/
    │   ├── mod.rs
    │   ├── content.rs                  # Content, ContentMetadata
    │   ├── content_error.rs
    │   ├── link.rs                     # CampaignLink types
    │   ├── paper.rs                    # Paper/document types
    │   ├── search.rs                   # SearchRequest/Response
    │   └── builders/
    │       ├── mod.rs
    │       ├── content.rs
    │       └── link.rs
    ├── repository/
    │   ├── mod.rs
    │   ├── content/
    │   │   └── mod.rs                  # ContentRepository
    │   ├── images/
    │   │   └── mod.rs
    │   ├── link/
    │   │   ├── mod.rs                  # LinkRepository
    │   │   └── analytics.rs
    │   └── search/
    │       └── mod.rs                  # SearchRepository
    └── services/
        ├── mod.rs
        ├── content.rs                  # Content business logic
        ├── content_provider.rs         # BlogContentProvider trait impl
        ├── ingestion/
        │   └── mod.rs                  # IngestionService
        ├── link/
        │   ├── mod.rs
        │   ├── analytics.rs
        │   └── generation.rs
        ├── search/
        │   └── mod.rs                  # SearchService
        └── validation/
            └── mod.rs                  # Content validation
```

### 1.2 Current Dependencies

From `crates/domain/content/Cargo.toml`:

```toml
[dependencies]
# Internal dependencies
systemprompt-core-database = { path = "../../infra/database" }
systemprompt-core-logging = { path = "../../infra/logging" }
systemprompt-core-users = { path = "../users" }          # USER DEPENDENCY
systemprompt-runtime = { path = "../../app/runtime" }
systemprompt-models = { path = "../../shared/models" }
systemprompt-identifiers = { path = "../../shared/identifiers" }
systemprompt-traits = { path = "../../shared/traits" }
systemprompt-core-files = { path = "../files" }          # FILE DEPENDENCY
systemprompt-core-config = { path = "../../infra/config" }
```

**Cross-Domain Dependencies** (violations of architecture rules):
- `systemprompt-core-users` - for author information
- `systemprompt-core-files` - for image handling

### 1.3 Public API

From `src/lib.rs`:

```rust
// Models
pub use models::{
    Content, ContentMetadata, IngestionOptions, IngestionReport,
    SearchFilters, SearchRequest, SearchResponse, SearchResult,
};

// Repositories
pub use repository::{ContentRepository, SearchRepository};

// Services
pub use services::{BlogContentProvider, IngestionService, SearchService};

// API
pub use api::{get_content_handler, list_content_by_source_handler, query_handler, router};

// Analytics
pub use analytics::{LinkAnalyticsRepository, LinkAnalyticsService};

// Jobs
pub use jobs::ContentIngestionJob;
```

### 1.4 API Routes

From `src/api/routes/mod.rs`:

| Route | Method | Handler |
|-------|--------|---------|
| `/query` | POST | `query_handler` |
| `/{source_id}` | GET | `list_content_by_source_handler` |
| `/{source_id}/{slug}` | GET | `get_content_handler` |
| `/links/generate` | POST | `generate_link_handler` |
| `/links` | GET | `list_links_handler` |
| `/links/{link_id}/performance` | GET | `get_link_performance_handler` |
| `/links/{link_id}/clicks` | GET | `get_link_clicks_handler` |
| `/links/campaigns/{campaign_id}/performance` | GET | `get_campaign_performance_handler` |
| `/links/journey` | GET | `get_content_journey_handler` |
| `/r/{short_code}` | GET | `redirect_handler` (separate router) |

### 1.5 Database Tables

| Table | Schema File | Purpose |
|-------|-------------|---------|
| `markdown_content` | `markdown_content.sql` | Main content storage |
| `markdown_categories` | `markdown_categories.sql` | Content categories |
| `campaign_links` | `campaign_links.sql` | Trackable links |
| `link_clicks` | `link_clicks.sql` | Click events |
| `link_analytics_daily` | `link_analytics_views.sql` | Aggregated analytics |
| `content_performance_metrics` | `content_performance_metrics.sql` | Performance data |
| (FTS index) | `markdown_fts.sql` | Full-text search |

### 1.6 Scheduled Job

```rust
// src/jobs/content_ingestion.rs
impl Job for ContentIngestionJob {
    fn name(&self) -> &'static str { "content_ingestion" }
    fn schedule(&self) -> &'static str { "0 0 * * * *" }  // Every hour
}

systemprompt_traits::submit_job!(&ContentIngestionJob);
```

---

## 2. Extraction Strategy

### 2.1 Goals

1. **Zero Core Changes**: Blog functionality removed from core, no breaking changes
2. **Reference Implementation**: Serve as THE example of how to build extensions
3. **Full Feature Parity**: All existing functionality preserved
4. **Clear Dependencies**: Explicit, type-safe dependency declarations

### 2.2 Approach

Create a new extension crate that:
- Implements `ExtensionType`, `SchemaExtension`, `ApiExtension`, `JobExtension`
- Uses the new type-safe dependency system
- Embeds SQL schemas via `include_str!`
- Provides explicit capability requirements

---

## 3. New Extension Structure

### 3.1 Location

```
/var/www/html/systemprompt-template/extensions/blog/
├── Cargo.toml
├── README.md                           # Extension documentation
├── schema/
│   ├── 001_markdown_content.sql
│   ├── 002_markdown_categories.sql
│   ├── 003_campaign_links.sql
│   ├── 004_link_clicks.sql
│   ├── 005_link_analytics_views.sql
│   ├── 006_content_performance_metrics.sql
│   └── 007_markdown_fts.sql
└── src/
    ├── lib.rs                          # Extension entry point
    ├── extension.rs                    # BlogExtension impl
    ├── config.rs                       # BlogConfig
    ├── error.rs                        # BlogError
    ├── models/
    │   ├── mod.rs
    │   ├── content.rs
    │   ├── link.rs
    │   ├── search.rs
    │   └── builders/
    │       └── mod.rs
    ├── repository/
    │   ├── mod.rs
    │   ├── content.rs
    │   ├── link.rs
    │   ├── analytics.rs
    │   └── search.rs
    ├── services/
    │   ├── mod.rs
    │   ├── content.rs
    │   ├── ingestion.rs
    │   ├── link.rs
    │   ├── analytics.rs
    │   ├── search.rs
    │   └── validation.rs
    ├── api/
    │   ├── mod.rs
    │   ├── handlers/
    │   │   ├── mod.rs
    │   │   ├── content.rs
    │   │   ├── query.rs
    │   │   └── links.rs
    │   └── types.rs
    └── jobs/
        ├── mod.rs
        └── ingestion.rs
```

### 3.2 Cargo.toml

```toml
[package]
name = "systemprompt-blog-extension"
version = "1.0.0"
edition = "2021"
description = "Blog/CMS extension for SystemPrompt - reference implementation"
license = "MIT"
repository = "https://github.com/systempromptio/systemprompt-template"

[dependencies]
# Extension framework
systemprompt-extension = { git = "https://github.com/systempromptio/systemprompt-core" }

# Core shared types
systemprompt-models = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-identifiers = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-traits = { git = "https://github.com/systempromptio/systemprompt-core" }

# Database access
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls"] }

# Web framework
axum = "0.8"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Markdown processing
pulldown-cmark = "0.12"
comrak = "0.17"

# Utilities
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.18", features = ["v4", "serde"] }
thiserror = "2.0"
tracing = "0.1"
sha2 = "0.10"
walkdir = "2.0"
async-trait = "0.1"

[dev-dependencies]
tokio = { version = "1", features = ["test-util", "macros"] }
tempfile = "3.0"
```

---

## 4. Extension Implementation

### 4.1 Extension Entry Point

**File**: `extensions/blog/src/lib.rs`

```rust
//! Blog Extension for SystemPrompt
//!
//! This is the reference implementation demonstrating how to build
//! a full-featured extension with:
//! - Database schemas
//! - API routes
//! - Background jobs
//! - Type-safe dependencies
//!
//! # Usage
//!
//! ```rust,ignore
//! use systemprompt::prelude::*;
//! use systemprompt_blog_extension::BlogExtension;
//!
//! let registry = ExtensionBuilder::new()
//!     .extension(BlogExtension::default())
//!     .build()?;
//! ```

#![allow(clippy::module_name_repetitions)]

mod api;
mod config;
mod error;
mod extension;
mod jobs;
mod models;
mod repository;
mod services;

pub use config::BlogConfig;
pub use error::BlogError;
pub use extension::BlogExtension;

pub use models::{
    Content, ContentMetadata, IngestionOptions, IngestionReport,
    SearchFilters, SearchRequest, SearchResponse, SearchResult,
    CampaignLink, LinkClick, LinkPerformance,
};

pub use repository::{ContentRepository, LinkRepository, SearchRepository};
pub use services::{ContentService, IngestionService, LinkService, SearchService};
pub use jobs::ContentIngestionJob;
```

### 4.2 Extension Type Implementation

**File**: `extensions/blog/src/extension.rs`

```rust
//! BlogExtension - reference implementation of a full extension.

use std::sync::Arc;
use axum::Router;
use sqlx::PgPool;
use systemprompt_extension::{
    ApiExtension, Dependencies, ExtensionType, JobExtension,
    NoDependencies, SchemaDefinition, SchemaExtension,
};
use systemprompt_traits::Job;

use crate::{api, jobs::ContentIngestionJob, BlogConfig};

/// Blog extension providing content management, search, and analytics.
///
/// # Capabilities
///
/// - **Schema**: 7 database tables for content, links, and analytics
/// - **API**: REST endpoints for content CRUD, search, and link tracking
/// - **Jobs**: Hourly content ingestion from filesystem
///
/// # Dependencies
///
/// This extension has no dependencies on other extensions.
/// It only requires database and config capabilities from the context.
#[derive(Debug, Default, Clone)]
pub struct BlogExtension;

impl ExtensionType for BlogExtension {
    const ID: &'static str = "blog";
    const NAME: &'static str = "Blog & Content Management";
    const VERSION: &'static str = "1.0.0";
    const PRIORITY: u32 = 100;
}

// Blog has no dependencies on other extensions
impl NoDependencies for BlogExtension {}

impl SchemaExtension for BlogExtension {
    fn schemas(&self) -> Vec<SchemaDefinition> {
        vec![
            SchemaDefinition::embedded(
                "markdown_content",
                include_str!("../schema/001_markdown_content.sql"),
            ).with_required_columns(vec![
                "id".into(),
                "slug".into(),
                "title".into(),
                "body".into(),
            ]),
            SchemaDefinition::embedded(
                "markdown_categories",
                include_str!("../schema/002_markdown_categories.sql"),
            ),
            SchemaDefinition::embedded(
                "campaign_links",
                include_str!("../schema/003_campaign_links.sql"),
            ),
            SchemaDefinition::embedded(
                "link_clicks",
                include_str!("../schema/004_link_clicks.sql"),
            ),
            SchemaDefinition::embedded(
                "link_analytics_daily",
                include_str!("../schema/005_link_analytics_views.sql"),
            ),
            SchemaDefinition::embedded(
                "content_performance_metrics",
                include_str!("../schema/006_content_performance_metrics.sql"),
            ),
            // FTS index creation
            SchemaDefinition::embedded(
                "markdown_fts_index",
                include_str!("../schema/007_markdown_fts.sql"),
            ),
        ]
    }

    fn migration_weight(&self) -> u32 {
        100 // User extension, runs after core tables
    }
}

impl ApiExtension for BlogExtension {
    type Db = PgPool;
    type Config = BlogConfig;

    fn router(&self, db: &PgPool, config: &BlogConfig) -> Router {
        api::router(db.clone(), config.clone())
    }

    fn base_path(&self) -> &'static str {
        "/api/v1/content"
    }

    fn requires_auth(&self) -> bool {
        false // Public content endpoints
    }
}

impl JobExtension for BlogExtension {
    fn jobs(&self) -> Vec<Arc<dyn Job>> {
        vec![Arc::new(ContentIngestionJob)]
    }
}
```

### 4.3 Config

**File**: `extensions/blog/src/config.rs`

```rust
//! Blog extension configuration.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the blog extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogConfig {
    /// Content sources to ingest
    pub content_sources: Vec<ContentSource>,

    /// Base URL for generated links
    #[serde(default = "default_base_url")]
    pub base_url: String,

    /// Enable link tracking
    #[serde(default = "default_true")]
    pub enable_link_tracking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSource {
    /// Unique identifier for this source
    pub source_id: String,

    /// Category ID for content from this source
    pub category_id: String,

    /// Filesystem path to content directory
    pub path: PathBuf,

    /// Allowed content types (e.g., ["article", "tutorial"])
    pub allowed_content_types: Vec<String>,

    /// Whether this source is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Override existing content on re-ingestion
    #[serde(default)]
    pub override_existing: bool,
}

impl Default for BlogConfig {
    fn default() -> Self {
        Self {
            content_sources: vec![],
            base_url: default_base_url(),
            enable_link_tracking: true,
        }
    }
}

fn default_base_url() -> String {
    "https://example.com".to_string()
}

fn default_true() -> bool {
    true
}
```

---

## 5. API Routes Migration

### 5.1 Router Setup

**File**: `extensions/blog/src/api/mod.rs`

```rust
//! Blog API routes.

pub mod handlers;
mod types;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;

use crate::BlogConfig;

pub use types::*;

/// Build the blog API router.
pub fn router(db: PgPool, config: BlogConfig) -> Router {
    let state = BlogState { db, config };

    Router::new()
        // Content endpoints
        .route("/query", post(handlers::query_handler))
        .route("/:source_id", get(handlers::list_content_handler))
        .route("/:source_id/:slug", get(handlers::get_content_handler))

        // Link tracking endpoints
        .route("/links/generate", post(handlers::generate_link_handler))
        .route("/links", get(handlers::list_links_handler))
        .route("/links/:link_id/performance", get(handlers::link_performance_handler))
        .route("/links/:link_id/clicks", get(handlers::link_clicks_handler))
        .route("/links/campaigns/:campaign_id/performance", get(handlers::campaign_performance_handler))
        .route("/links/journey", get(handlers::content_journey_handler))

        .with_state(state)
}

/// Redirect router (mounted separately at /r/)
pub fn redirect_router(db: PgPool) -> Router {
    Router::new()
        .route("/:short_code", get(handlers::redirect_handler))
        .with_state(db)
}

#[derive(Clone)]
pub struct BlogState {
    pub db: PgPool,
    pub config: BlogConfig,
}
```

### 5.2 Handler Migration

**File**: `extensions/blog/src/api/handlers/content.rs`

```rust
//! Content API handlers.

use axum::{
    extract::{Path, State},
    Json,
};

use crate::{
    api::BlogState,
    models::{Content, SearchRequest, SearchResponse},
    repository::ContentRepository,
    services::SearchService,
    BlogError,
};

/// Query content with filters and pagination.
pub async fn query_handler(
    State(state): State<BlogState>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, BlogError> {
    let search_service = SearchService::new(&state.db);
    let response = search_service.search(request).await?;
    Ok(Json(response))
}

/// List content for a source.
pub async fn list_content_handler(
    State(state): State<BlogState>,
    Path(source_id): Path<String>,
) -> Result<Json<Vec<Content>>, BlogError> {
    let repo = ContentRepository::new(&state.db);
    let content = repo.list_by_source(&source_id).await?;
    Ok(Json(content))
}

/// Get single content item by slug.
pub async fn get_content_handler(
    State(state): State<BlogState>,
    Path((source_id, slug)): Path<(String, String)>,
) -> Result<Json<Content>, BlogError> {
    let repo = ContentRepository::new(&state.db);
    let content = repo
        .get_by_slug(&source_id, &slug)
        .await?
        .ok_or(BlogError::NotFound)?;
    Ok(Json(content))
}
```

---

## 6. Repository Migration

### 6.1 ContentRepository

**File**: `extensions/blog/src/repository/content.rs`

```rust
//! Content repository - database access layer.

use sqlx::PgPool;
use crate::{models::Content, BlogError};

pub struct ContentRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> ContentRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_slug(
        &self,
        source_id: &str,
        slug: &str,
    ) -> Result<Option<Content>, BlogError> {
        let content = sqlx::query_as!(
            Content,
            r#"
            SELECT id, slug, title, description, body, author,
                   published_at, keywords, kind, image, category_id,
                   source_id, version_hash, public, links, updated_at
            FROM markdown_content
            WHERE source_id = $1 AND slug = $2 AND public = true
            "#,
            source_id,
            slug
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(content)
    }

    pub async fn list_by_source(&self, source_id: &str) -> Result<Vec<Content>, BlogError> {
        let content = sqlx::query_as!(
            Content,
            r#"
            SELECT id, slug, title, description, body, author,
                   published_at, keywords, kind, image, category_id,
                   source_id, version_hash, public, links, updated_at
            FROM markdown_content
            WHERE source_id = $1 AND public = true
            ORDER BY published_at DESC
            "#,
            source_id
        )
        .fetch_all(self.pool)
        .await?;

        Ok(content)
    }

    pub async fn upsert(&self, content: &Content) -> Result<(), BlogError> {
        sqlx::query!(
            r#"
            INSERT INTO markdown_content (
                id, slug, title, description, body, author,
                published_at, keywords, kind, image, category_id,
                source_id, version_hash, public, links, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT (slug) DO UPDATE SET
                title = EXCLUDED.title,
                description = EXCLUDED.description,
                body = EXCLUDED.body,
                author = EXCLUDED.author,
                published_at = EXCLUDED.published_at,
                keywords = EXCLUDED.keywords,
                kind = EXCLUDED.kind,
                image = EXCLUDED.image,
                category_id = EXCLUDED.category_id,
                version_hash = EXCLUDED.version_hash,
                public = EXCLUDED.public,
                links = EXCLUDED.links,
                updated_at = CURRENT_TIMESTAMP
            "#,
            content.id,
            content.slug,
            content.title,
            content.description,
            content.body,
            content.author,
            content.published_at,
            content.keywords,
            content.kind,
            content.image,
            content.category_id,
            content.source_id,
            content.version_hash,
            content.public,
            content.links,
            content.updated_at,
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }
}
```

---

## 7. Job Migration

**File**: `extensions/blog/src/jobs/ingestion.rs`

```rust
//! Content ingestion background job.

use anyhow::Result;
use std::sync::Arc;
use systemprompt_traits::{Job, JobContext, JobResult};

use crate::services::IngestionService;

/// Scheduled job that ingests markdown content from configured directories.
#[derive(Debug, Clone, Copy)]
pub struct ContentIngestionJob;

#[async_trait::async_trait]
impl Job for ContentIngestionJob {
    fn name(&self) -> &'static str {
        "blog_content_ingestion"
    }

    fn description(&self) -> &'static str {
        "Ingests markdown content from configured directories into the database"
    }

    fn schedule(&self) -> &'static str {
        "0 0 * * * *" // Every hour
    }

    async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
        let start = std::time::Instant::now();

        // Get database pool from job context
        let pool = ctx.db_pool::<sqlx::PgPool>()
            .ok_or_else(|| anyhow::anyhow!("Database pool not available"))?;

        // Get blog config from job context
        let config = ctx.extension_config::<crate::BlogConfig>()
            .ok_or_else(|| anyhow::anyhow!("Blog config not available"))?;

        let ingestion_service = IngestionService::new(pool);

        let mut total_processed = 0u64;
        let mut total_errors = 0u64;

        for source in &config.content_sources {
            if !source.enabled {
                continue;
            }

            match ingestion_service.ingest_source(source).await {
                Ok(report) => {
                    total_processed += report.files_processed as u64;
                    total_errors += report.errors.len() as u64;
                }
                Err(e) => {
                    tracing::error!(source = %source.source_id, error = %e, "Ingestion failed");
                    total_errors += 1;
                }
            }
        }

        Ok(JobResult::success()
            .with_stats(total_processed, total_errors)
            .with_duration(start.elapsed().as_millis() as u64))
    }
}
```

---

## 8. Removing from Core

### 8.1 Files to Delete from Core

```bash
# After extension is working, remove from core:
rm -rf /var/www/html/systemprompt-core/crates/domain/content/
```

### 8.2 Update Core Cargo.toml

Remove from workspace members:
```diff
# /var/www/html/systemprompt-core/Cargo.toml
[workspace]
members = [
    # ...
-   "crates/domain/content",
    # ...
]
```

### 8.3 Update Core Dependencies

Any crate that depends on `systemprompt-core-blog` needs updating:
- Check `crates/entry/api/` for blog route mounting
- Check `crates/app/scheduler/` for job registration
- Check main binary for blog imports

---

## 9. Execution Checklist

### Phase 2a: Create Extension Structure
- [ ] Create `extensions/blog/` directory in template
- [ ] Create `Cargo.toml` with dependencies
- [ ] Copy and adapt schema files with `include_str!` embedding
- [ ] Create `src/lib.rs` with public API

### Phase 2b: Implement Extension Traits
- [ ] Implement `ExtensionType` for `BlogExtension`
- [ ] Implement `SchemaExtension` with all 7 tables
- [ ] Implement `ApiExtension` with router
- [ ] Implement `JobExtension` with `ContentIngestionJob`

### Phase 2c: Migrate Models
- [ ] Copy `models/content.rs`
- [ ] Copy `models/link.rs`
- [ ] Copy `models/search.rs`
- [ ] Copy `models/builders/`

### Phase 2d: Migrate Repositories
- [ ] Create `repository/content.rs`
- [ ] Create `repository/link.rs`
- [ ] Create `repository/analytics.rs`
- [ ] Create `repository/search.rs`

### Phase 2e: Migrate Services
- [ ] Create `services/content.rs`
- [ ] Create `services/ingestion.rs`
- [ ] Create `services/link.rs`
- [ ] Create `services/search.rs`
- [ ] Create `services/validation.rs`

### Phase 2f: Migrate API
- [ ] Create `api/mod.rs` with router
- [ ] Create `api/handlers/content.rs`
- [ ] Create `api/handlers/links.rs`
- [ ] Create `api/handlers/query.rs`
- [ ] Create `api/types.rs`

### Phase 2g: Migrate Jobs
- [ ] Create `jobs/ingestion.rs`

### Phase 2h: Documentation
- [ ] Create `README.md` with usage examples
- [ ] Add rustdoc comments to all public items

---

## 10. Output Artifacts

After executing this phase:

1. **New crate**: `extensions/blog/` in systemprompt-template
2. **Working extension**: Implements all extension traits
3. **Full feature parity**: All existing functionality preserved
4. **Reference docs**: Comprehensive documentation for users
5. **Ready for removal**: Blog can be removed from core

---

## 11. File Count Summary

| Category | Files | Lines (approx) |
|----------|-------|----------------|
| Schema | 7 | 150 |
| Models | 6 | 400 |
| Repository | 4 | 300 |
| Services | 6 | 600 |
| API | 5 | 400 |
| Jobs | 2 | 150 |
| Extension | 3 | 200 |
| **Total** | **33** | **~2200** |
