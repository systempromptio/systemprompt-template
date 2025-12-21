# Phase 3: Template Integration

**Objective**: Integrate the Blog extension into systemprompt-template, demonstrating the complete extension workflow for downstream users.

**Prerequisites**:
- Phase 1 (Extension Framework Core) must be complete
- Phase 2 (Blog Extension Extraction) must be complete

---

## 1. Overview

This phase:
1. Updates template's Cargo.toml to use new extension system
2. Configures the blog extension in template's main.rs
3. Sets up blog-specific configuration
4. Updates API server to mount extension routes
5. Verifies the complete integration works

---

## 2. Template Project Structure

### 2.1 Current Structure

```
/var/www/html/systemprompt-template/
├── Cargo.toml                          # Workspace root
├── core/                               # Git submodule (to be replaced)
├── extensions/                         # NEW: Local extensions
│   └── blog/                          # Blog extension (from Phase 2)
├── services/
│   ├── config/
│   │   ├── agents.yaml
│   │   ├── content.yaml               # Blog content config
│   │   └── ...
│   └── content/                        # Markdown content files
│       ├── blog/
│       └── docs/
└── src/
    └── main.rs                         # Application entry point
```

### 2.2 Target Structure

```
/var/www/html/systemprompt-template/
├── Cargo.toml                          # Updated workspace
├── extensions/
│   └── blog/                          # Blog extension crate
│       ├── Cargo.toml
│       └── src/
├── services/
│   ├── config/
│   │   ├── extensions.yaml            # NEW: Extension config
│   │   ├── blog.yaml                  # Blog-specific config
│   │   └── ...
│   └── content/
└── src/
    ├── main.rs                         # Updated entry point
    └── lib.rs                          # Library exports
```

---

## 3. Workspace Configuration

### 3.1 Update Cargo.toml

**File**: `/var/www/html/systemprompt-template/Cargo.toml`

```toml
[workspace]
members = [
    # Local extensions
    "extensions/blog",

    # Local MCP servers
    "services/mcp/system-tools",
]
resolver = "2"

[workspace.package]
edition = "2021"
version = "1.0.0"
authors = ["Your Name <your.email@example.com>"]
license = "MIT"

[workspace.dependencies]
# ============================================
# SYSTEMPROMPT CORE (from crates.io or git)
# ============================================

# Option 1: From crates.io (recommended for production)
# systemprompt = { version = "1.0", features = ["api", "database"] }

# Option 2: From git (for development)
systemprompt = { git = "https://github.com/systempromptio/systemprompt-core", features = ["api", "database"] }

# Extension framework
systemprompt-extension = { git = "https://github.com/systempromptio/systemprompt-core" }

# Shared types
systemprompt-models = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-identifiers = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-traits = { git = "https://github.com/systempromptio/systemprompt-core" }

# Infrastructure
systemprompt-core-database = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-core-config = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-core-logging = { git = "https://github.com/systempromptio/systemprompt-core" }

# ============================================
# LOCAL EXTENSIONS
# ============================================
systemprompt-blog-extension = { path = "extensions/blog" }

# ============================================
# THIRD-PARTY DEPENDENCIES
# ============================================
tokio = { version = "1.47", features = ["full"] }
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres"] }
axum = "0.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.18", features = ["v4", "serde"] }
thiserror = "2.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
anyhow = "1.0"
async-trait = "0.1"

[workspace.lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "deny", priority = -1 }
```

### 3.2 Root Package

**File**: `/var/www/html/systemprompt-template/Cargo.toml` (continued)

```toml
[package]
name = "systemprompt-template"
version.workspace = true
edition.workspace = true
description = "SystemPrompt Template - Example project with blog extension"

[[bin]]
name = "systemprompt-server"
path = "src/main.rs"

[dependencies]
# Core framework
systemprompt = { workspace = true }
systemprompt-extension = { workspace = true }
systemprompt-core-database = { workspace = true }
systemprompt-core-config = { workspace = true }
systemprompt-core-logging = { workspace = true }

# Local extensions
systemprompt-blog-extension = { workspace = true }

# Runtime
tokio = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
anyhow = { workspace = true }
```

---

## 4. Application Entry Point

### 4.1 Main Entry Point

**File**: `/var/www/html/systemprompt-template/src/main.rs`

```rust
//! SystemPrompt Template Server
//!
//! This demonstrates how to build a SystemPrompt application with extensions.

use anyhow::Result;
use systemprompt::prelude::*;
use systemprompt_blog_extension::BlogExtension;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    tracing::info!("Starting SystemPrompt Template Server");

    // Load configuration
    let config = load_config()?;

    // Build extension registry with type-safe dependencies
    let extensions = build_extensions()?;

    // Create application context
    let ctx = AppContextBuilder::new()
        .with_config(config)
        .with_extensions(extensions)
        .build()
        .await?;

    // Install extension database schemas
    install_extension_schemas(&ctx).await?;

    // Start the API server
    start_server(&ctx).await?;

    Ok(())
}

/// Initialize tracing/logging.
fn init_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info,sqlx=warn".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Load application configuration.
fn load_config() -> Result<Config> {
    // Load from environment and config files
    let config = Config::load_from_env()?;
    Ok(config)
}

/// Build the extension registry.
///
/// This is where all extensions are registered with type-safe
/// dependency checking at compile time.
fn build_extensions() -> Result<ExtensionRegistry> {
    tracing::info!("Building extension registry");

    // Blog extension has no dependencies, so it can be first
    let registry = ExtensionBuilder::new()
        .extension(BlogExtension::default())
        // Add more extensions here:
        // .extension(AnalyticsExtension::default())
        // .extension(SeoExtension::default())
        .build()?;

    tracing::info!(
        extensions = registry.len(),
        "Extension registry built"
    );

    Ok(registry)
}

/// Install database schemas for all extensions.
async fn install_extension_schemas(ctx: &AppContext) -> Result<()> {
    tracing::info!("Installing extension schemas");

    let registry = ctx.extension_registry();
    let db = ctx.database();

    for schema_ext in registry.schema_extensions() {
        tracing::debug!(
            extension = schema_ext.id(),
            tables = schema_ext.schemas().len(),
            "Installing schemas"
        );

        for schema in schema_ext.schemas() {
            install_schema(db, &schema).await?;
        }
    }

    tracing::info!("Extension schemas installed");
    Ok(())
}

/// Install a single schema definition.
async fn install_schema(db: &PgPool, schema: &SchemaDefinition) -> Result<()> {
    // Check if table exists
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = $1)"
    )
    .bind(&schema.table)
    .fetch_one(db)
    .await?;

    if !exists {
        tracing::info!(table = %schema.table, "Creating table");
        let sql = schema.sql.resolve()?;
        sqlx::raw_sql(&sql).execute(db).await?;
    } else {
        tracing::debug!(table = %schema.table, "Table exists, skipping");
    }

    Ok(())
}

/// Start the API server with extension routes mounted.
async fn start_server(ctx: &AppContext) -> Result<()> {
    use axum::Router;

    // Build core routes
    let mut app = Router::new()
        .route("/health", axum::routing::get(|| async { "OK" }));

    // Mount extension routes
    let registry = ctx.extension_registry();
    for api_ext in registry.api_extensions() {
        let ext_router = api_ext.router(ctx.database(), ctx.config());
        let base_path = api_ext.base_path();

        tracing::info!(
            extension = api_ext.id(),
            path = base_path,
            "Mounting extension routes"
        );

        app = app.nest(base_path, ext_router);
    }

    // Add middleware
    app = app
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::cors::CorsLayer::permissive());

    // Bind and serve
    let addr = ctx.server_address();
    tracing::info!(%addr, "Starting server");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

---

## 5. Extension Configuration

### 5.1 Blog Extension Config

**File**: `/var/www/html/systemprompt-template/services/config/blog.yaml`

```yaml
# Blog Extension Configuration
#
# This configures the blog extension for this template project.

# Content sources to ingest
content_sources:
  - source_id: "blog"
    category_id: "blog"
    path: "./services/content/blog"
    allowed_content_types:
      - article
      - tutorial
      - announcement
    enabled: true
    override_existing: false

  - source_id: "docs"
    category_id: "documentation"
    path: "./services/content/docs"
    allowed_content_types:
      - documentation
      - guide
      - reference
    enabled: true
    override_existing: true

# Base URL for generated links
base_url: "${BASE_URL:-https://example.com}"

# Enable link tracking for analytics
enable_link_tracking: true
```

### 5.2 Extensions Config

**File**: `/var/www/html/systemprompt-template/services/config/extensions.yaml`

```yaml
# Extension Configuration
#
# This file configures all extensions for the template project.

extensions:
  # Blog extension settings
  blog:
    enabled: true
    config_file: "./services/config/blog.yaml"

  # Future extensions can be added here:
  # analytics:
  #   enabled: true
  #   config_file: "./services/config/analytics.yaml"
```

---

## 6. Content Directory Structure

### 6.1 Blog Content

```
/var/www/html/systemprompt-template/services/content/
├── blog/
│   ├── 2024-01-introduction.md
│   ├── 2024-02-getting-started.md
│   └── ...
└── docs/
    ├── installation.md
    ├── configuration.md
    ├── extensions/
    │   ├── creating-extensions.md
    │   └── blog-extension.md
    └── ...
```

### 6.2 Example Content File

**File**: `/var/www/html/systemprompt-template/services/content/blog/2024-01-introduction.md`

```markdown
---
title: "Introduction to SystemPrompt"
description: "Learn how to build AI-powered applications with SystemPrompt"
author: "SystemPrompt Team"
published_at: "2024-01-15T00:00:00Z"
keywords: "systemprompt, ai, agents, introduction"
kind: "article"
image: "/images/blog/introduction.png"
public: true
---

# Introduction to SystemPrompt

SystemPrompt is an extensible framework for building AI-powered applications.

## Getting Started

To get started, install SystemPrompt:

```bash
cargo add systemprompt
```

## Creating Your First Extension

Extensions allow you to add custom functionality...
```

---

## 7. Environment Configuration

### 7.1 Environment Variables

**File**: `/var/www/html/systemprompt-template/.env.local`

```bash
# Database
DATABASE_URL=postgres://postgres:password@localhost:5432/systemprompt

# Server
HOST=0.0.0.0
PORT=3000
BASE_URL=http://localhost:3000

# Logging
RUST_LOG=info,sqlx=warn,systemprompt=debug

# Extensions
EXTENSIONS_CONFIG=./services/config/extensions.yaml
BLOG_CONFIG=./services/config/blog.yaml
```

---

## 8. Integration with Core Services

### 8.1 API Route Registration

The blog extension routes are automatically mounted at `/api/v1/content/`:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/content/query` | POST | Search content |
| `/api/v1/content/{source}` | GET | List content by source |
| `/api/v1/content/{source}/{slug}` | GET | Get single content |
| `/api/v1/content/links/generate` | POST | Generate tracking link |
| `/api/v1/content/links` | GET | List all links |
| `/api/v1/content/links/{id}/performance` | GET | Link performance |

### 8.2 Job Scheduler Integration

The blog extension's `ContentIngestionJob` is automatically registered with the scheduler:

```rust
// In main.rs or job registration
for job_ext in registry.job_extensions() {
    for job in job_ext.jobs() {
        scheduler.register(job);
    }
}
```

---

## 9. Testing the Integration

### 9.1 Start the Server

```bash
cd /var/www/html/systemprompt-template
cargo run
```

### 9.2 Verify Extension Routes

```bash
# List blog content
curl http://localhost:3000/api/v1/content/blog

# Search content
curl -X POST http://localhost:3000/api/v1/content/query \
  -H "Content-Type: application/json" \
  -d '{"query": "introduction", "limit": 10}'

# Get specific content
curl http://localhost:3000/api/v1/content/blog/2024-01-introduction
```

### 9.3 Verify Schema Installation

```bash
# Connect to database
psql $DATABASE_URL

# Check tables exist
\dt markdown_*
\dt campaign_*
\dt link_*
```

---

## 10. Execution Checklist

### Phase 3a: Workspace Setup
- [ ] Update `/var/www/html/systemprompt-template/Cargo.toml`
- [ ] Add `extensions/blog` to workspace members
- [ ] Configure workspace dependencies

### Phase 3b: Main Entry Point
- [ ] Create/update `src/main.rs` with extension registration
- [ ] Implement `build_extensions()` function
- [ ] Implement `install_extension_schemas()` function
- [ ] Implement extension route mounting

### Phase 3c: Configuration
- [ ] Create `services/config/blog.yaml`
- [ ] Create `services/config/extensions.yaml`
- [ ] Update `.env.local` with extension config paths

### Phase 3d: Content Setup
- [ ] Ensure `services/content/blog/` exists with sample content
- [ ] Ensure `services/content/docs/` exists with documentation

### Phase 3e: Verification
- [ ] Run `cargo build` - should compile
- [ ] Run `cargo run` - server should start
- [ ] Test API endpoints - should return data
- [ ] Check database - tables should exist
- [ ] Run ingestion job - content should be indexed

---

## 11. Output Artifacts

After executing this phase:

1. **Updated template workspace** with blog extension as dependency
2. **Working main.rs** that demonstrates extension usage
3. **Configuration files** for blog extension
4. **Sample content** ready for ingestion
5. **Verified integration** with all tests passing

---

## 12. User Experience

After this phase, a user cloning the template can:

1. Clone the repository
2. Run `cargo build`
3. Run `cargo run`
4. Access blog content at `/api/v1/content/`

No understanding of the extension system is required for basic usage.
For customization, they can:
- Add their own content to `services/content/`
- Modify `services/config/blog.yaml`
- Create new extensions following the blog extension pattern
