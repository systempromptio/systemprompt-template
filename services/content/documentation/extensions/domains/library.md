---
title: "Library Extensions"
description: "Build library extensions that compile into the main binary: API routes, database schemas, background jobs, and providers."
author: "SystemPrompt Team"
slug: "extensions/domains/library"
keywords: "rust, extensions, library, api, schema, jobs, traits"
image: "/files/images/docs/extensions-rust.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Library Extensions

Library extensions compile directly into your main binary. They implement the Extension trait to add capabilities: HTTP routes, database schemas, background jobs, and providers. At runtime, the framework discovers these extensions and calls their lifecycle hooks to wire everything together.

## When to Use Library Extensions

Choose a library extension when you need:

- **Shared database connections** - Access the same connection pool as the runtime
- **Coordinated lifecycle** - Schemas migrate together, routes mount together
- **Single binary deployment** - Everything in one executable
- **Internal APIs** - Endpoints that serve the web frontend

Choose an [MCP server](/documentation/extensions/domains/mcp) (standalone binary) when you need:

- **Independent scaling** - Run multiple instances of the tool server
- **Isolation** - Separate process with its own resource limits
- **External tool access** - AI agents calling your tools via MCP protocol

Choose a [CLI extension](/documentation/extensions/domains/cli) (standalone binary) when you need:

- **Shell scripting** - Commands that agents invoke via subprocess
- **External integrations** - Tools that connect to third-party services
- **Utility commands** - One-off operations like data migration

## The Extension Trait

Library extensions implement a single `Extension` trait with 30+ methods. Each method returns data describing a capability. The runtime calls these methods at startup to discover what your extension provides.

```rust
use systemprompt::extension::prelude::*;
use systemprompt::traits::Job;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct MyExtension;

impl Extension for MyExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "my-extension",
            name: "My Extension",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    fn schemas(&self) -> Vec<SchemaDefinition> {
        vec![
            SchemaDefinition::inline("my_tables", include_str!("../schema/001_tables.sql")),
        ]
    }

    fn router(&self, ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
        let db = ctx.database();
        let pool = db.as_any().downcast_ref::<Database>()?.pool()?;
        let router = crate::api::router(pool);
        Some(ExtensionRouter::new(router, "/api/v1/my-extension"))
    }

    fn jobs(&self) -> Vec<Arc<dyn Job>> {
        vec![Arc::new(crate::jobs::MyJob)]
    }

    fn priority(&self) -> u32 {
        50
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["users"]
    }
}

register_extension!(MyExtension);
```

All trait methods have sensible defaults. You implement only the hooks your extension needs. For the complete reference, see [Extension Trait Reference](/documentation/extensions/traits/extension-trait).

## Extension Points

Library extensions can provide:

| Hook | Purpose |
|------|---------|
| `schemas()` | Database table definitions |
| `migrations()` | Schema migration SQL |
| `router()` | HTTP API routes |
| `jobs()` | Background tasks |
| `config_prefix()` | Configuration namespace |
| `llm_providers()` | LLM implementations |
| `tool_providers()` | MCP tool implementations |
| `page_data_providers()` | Template data for pages |
| `component_renderers()` | HTML fragment generators |
| `content_data_providers()` | Content enrichment |
| `template_data_extenders()` | Final template modifications |
| `page_prerenderers()` | Static page generators |
| `frontmatter_processors()` | Frontmatter parsing |
| `rss_feed_providers()` | RSS feed generation |
| `sitemap_providers()` | Sitemap generation |
| `roles()` | RBAC role definitions |
| `required_assets()` | CSS/JS asset declarations |

## Project Structure

```
extensions/my-extension/
├── Cargo.toml
├── schema/                 # SQL migrations
│   ├── 001_tables.sql
│   └── 002_indexes.sql
└── src/
    ├── lib.rs              # Public exports
    ├── extension.rs        # Extension trait implementation
    ├── api/
    │   ├── mod.rs          # Router definition
    │   └── handlers/       # HTTP handlers
    ├── models/             # Data types
    ├── repository/         # Data access layer
    ├── services/           # Business logic
    └── jobs/               # Background tasks
```

## Database Schemas

Return SQL definitions from the `schemas()` method. The runtime executes these during database initialization, ordered by `migration_weight()`.

```rust
pub const SCHEMA_TABLES: &str = include_str!("../schema/001_tables.sql");
pub const SCHEMA_INDEXES: &str = include_str!("../schema/002_indexes.sql");

fn schemas(&self) -> Vec<SchemaDefinition> {
    vec![
        SchemaDefinition::inline("my_tables", SCHEMA_TABLES),
        SchemaDefinition::inline("my_indexes", SCHEMA_INDEXES),
    ]
}

fn migration_weight(&self) -> u32 {
    50  // Lower = runs earlier
}
```

Schema SQL example:

```sql
-- schema/001_tables.sql
CREATE TABLE IF NOT EXISTS my_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_my_items_name ON my_items(name);
```

Use `CREATE TABLE IF NOT EXISTS` and similar idempotent patterns. Schemas may execute multiple times across restarts.

## HTTP Routes

Return an Axum router from the `router()` method. The runtime mounts it at the path you specify.

```rust
fn router(&self, ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
    let db_handle = ctx.database();
    let db = db_handle.as_any().downcast_ref::<Database>()?;
    let pool = db.pool()?;

    let router = Router::new()
        .route("/items", get(list_items).post(create_item))
        .route("/items/:id", get(get_item).put(update_item).delete(delete_item))
        .with_state(AppState { pool });

    Some(ExtensionRouter::new(router, "/api/v1/my-extension"))
}
```

Handler example:

```rust
use axum::{extract::{Path, State}, Json};

async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Item>, AppError> {
    let item = sqlx::query_as!(Item, "SELECT * FROM my_items WHERE id = $1", id)
        .fetch_one(state.pool.as_ref())
        .await?;
    Ok(Json(item))
}
```

## Background Jobs

Return job implementations from the `jobs()` method. Jobs implement the `Job` trait and register for scheduling.

```rust
fn jobs(&self) -> Vec<Arc<dyn Job>> {
    vec![
        Arc::new(CleanupJob),
        Arc::new(SyncJob),
    ]
}
```

See [Job Extension](/documentation/extensions/traits/job-extension) for implementation details.

## Dependencies

Declare dependencies on other extensions:

```rust
fn dependencies(&self) -> Vec<&'static str> {
    vec!["users", "oauth"]
}
```

The runtime validates dependencies exist and detects circular references. Extensions load in priority order (lower priority values load first).

## Registration

After implementing the Extension trait, register with the `register_extension!` macro:

```rust
register_extension!(MyExtension);
```

Then reference your extension in the template's `src/lib.rs` to prevent linker stripping:

```rust
pub use my_extension_crate as my_extension;

pub fn __force_extension_link() {
    let _ = core::hint::black_box(&web::WebExtension::PREFIX);
    let _ = core::hint::black_box(&my_extension::MyExtension::PREFIX);
}
```

See [Registration](/documentation/extensions/lifecycle/registration) for details.

## Cargo Configuration

```toml
[package]
name = "my-extension"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["rlib"]

[dependencies]
systemprompt = { workspace = true }
axum = { workspace = true }
tokio = { workspace = true }
sqlx = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
async-trait = { workspace = true }
anyhow = { workspace = true }
```

Use workspace dependencies from the root `Cargo.toml` to ensure version consistency across all extensions.

## Building

```bash
# Build all workspace members
cargo build

# Build release
cargo build --release

# Build specific extension
cargo build -p my-extension
```