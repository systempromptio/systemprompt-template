---
title: "Create Library Extension"
description: "Create a complete library extension with all capabilities."
author: "SystemPrompt"
slug: "build-02-library-extensions-create-extension"
keywords: "library, extension, create, rust"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Create Library Extension

Create a complete library extension. Reference: `extensions/web/` for working example.

> **Help**: `{ "command": "core playbooks show build_create-library-extension" }`

---

## Structure

```
extensions/my-extension/
├── Cargo.toml
├── schema/
│   └── 001_tables.sql
└── src/
    ├── lib.rs
    ├── extension.rs
    ├── error.rs
    ├── models/
    ├── repository/
    ├── services/
    ├── api/
    └── jobs/
```

---

## Cargo.toml

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
thiserror = { workspace = true }
tracing = { workspace = true }
```

---

## src/lib.rs

See `extensions/web/src/lib.rs:1-15` for reference.

```rust
mod error;
mod extension;
mod models;
mod repository;
mod services;
mod api;
mod jobs;

pub use error::MyExtensionError;
pub use extension::MyExtension;

pub const PREFIX: &str = "my-extension";
```

---

## src/extension.rs

See `extensions/web/src/extension.rs:15-80` for reference.

```rust
use std::sync::Arc;
use systemprompt::extension::prelude::*;

use crate::api;
use crate::jobs::CleanupJob;

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

    fn priority(&self) -> u32 {
        50
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["users"]
    }

    fn schemas(&self) -> Vec<SchemaDefinition> {
        vec![
            SchemaDefinition::inline("my_tables", include_str!("../schema/001_tables.sql")),
        ]
    }

    fn migration_weight(&self) -> u32 {
        50
    }

    fn router(&self, ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
        let db = ctx.database();
        let pool = db.as_any().downcast_ref::<Database>()?.pool()?;
        Some(ExtensionRouter::new(api::router(pool), "/api/v1/my-extension"))
    }

    fn jobs(&self) -> Vec<Arc<dyn Job>> {
        vec![Arc::new(CleanupJob)]
    }
}

register_extension!(MyExtension);
```

---

## src/error.rs

See `extensions/web/src/error.rs:1-40` for reference.

```rust
use axum::http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyExtensionError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(String),
}

impl MyExtensionError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NOT_FOUND",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Validation(_) => "VALIDATION_ERROR",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
        }
    }
}
```

---

## schema/001_tables.sql

```sql
CREATE TABLE IF NOT EXISTS my_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_my_items_name ON my_items(name);
CREATE INDEX IF NOT EXISTS idx_my_items_created ON my_items(created_at DESC);
```

---

## Checklist

- [ ] Package name follows `{name}-extension` pattern
- [ ] `Cargo.toml` uses workspace dependencies
- [ ] `src/extension.rs` implements `Extension` trait
- [ ] `metadata()` returns unique ID
- [ ] `register_extension!` macro called
- [ ] `PREFIX` constant exported in `src/lib.rs`
- [ ] Linked in template `src/lib.rs`
- [ ] `src/error.rs` implements status codes
- [ ] Schema files use idempotent patterns

---

## Code Quality

| Metric | Limit |
|--------|-------|
| File length | 300 lines |
| Function length | 75 lines |
| No `unwrap()` | Use `?` or `ok_or_else()` |
| No inline comments | Code documents itself |
| Typed identifiers | From `systemprompt_identifiers` |

---

## Quick Reference

| Task | Command/Action |
|------|----------------|
| Create crate | `mkdir -p extensions/my-extension/src` |
| Build | `cargo build --workspace` |
| Verify | `cargo run -- extensions list` |
| Lint | `cargo clippy --workspace -- -D warnings` |
| Format | `cargo fmt --all` |

---

## Related

-> See [Add Database Schema](add-schema.md) for database tables
-> See [Add API Routes](add-api-routes.md) for HTTP endpoints
-> See [Add Background Job](add-background-job.md) for scheduled tasks
-> See [Rust Standards](../06-standards/rust-standards.md) for code style