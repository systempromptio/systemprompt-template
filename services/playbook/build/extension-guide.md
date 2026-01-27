---
title: "Extension Guide Playbook"
description: "Step-by-step guide for building extensions on systemprompt-core."
keywords:
  - extension
  - guide
  - tutorial
  - build
---

# Extension Guide

This guide walks through building an extension step-by-step.

> **Help**: `{ "command": "playbook build" }` via `systemprompt_help`

---

## Prerequisites

- Rust toolchain installed
- Access to systemprompt-core repository
- Database (PostgreSQL) available

---

## Step 1: Create the Crate

```bash
# Create extension directory
mkdir -p extensions/my-extension/src
mkdir -p extensions/my-extension/schema

# Create Cargo.toml
cat > extensions/my-extension/Cargo.toml << 'EOF'
[package]
name = "systemprompt-my-extension"
version = "1.0.0"
edition = "2021"

[dependencies]
systemprompt-models = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-identifiers = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-traits = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-core-database = { git = "https://github.com/systempromptio/systemprompt-core" }
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio", "macros"] }
thiserror = "2.0"
async-trait = "0.1"
axum = "0.8"
tracing = "0.1"
anyhow = "1.0"

[lints]
workspace = true
EOF
```

---

## Step 2: Implement Extension Trait

Create `src/extension.rs`:

```rust
use systemprompt_traits::{Extension, ExtensionContext, ExtensionMetadata, SchemaDefinition};
use std::sync::Arc;

use crate::api;
use crate::jobs::MyJob;

#[derive(Debug, Default, Clone)]
pub struct MyExtension;

impl Extension for MyExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "my_extension",
            name: "My Extension",
            version: env!("CARGO_PKG_VERSION"),
            priority: 100,
            dependencies: vec![],
        }
    }

    fn schemas(&self) -> Vec<SchemaDefinition> {
        vec![
            SchemaDefinition::inline("items", include_str!("../schema/001_items.sql")),
        ]
    }

    fn router(&self, ctx: &ExtensionContext) -> Option<axum::Router> {
        let pool = ctx.database().postgres_pool()?;
        Some(api::router(pool))
    }

    fn jobs(&self) -> Vec<Arc<dyn systemprompt_traits::Job>> {
        vec![Arc::new(MyJob)]
    }
}

systemprompt_traits::register_extension!(MyExtension);
```

---

## Step 3: Implement ExtensionConfig (If Needed)

Create `src/config.rs`:

```rust
use serde::Deserialize;
use std::path::{Path, PathBuf};
use systemprompt::extension::typed::{ExtensionConfig, ExtensionConfigErrors};
use url::Url;

use crate::MyExtension;

#[derive(Debug, Deserialize)]
pub struct MyConfigRaw {
    pub data_path: String,
    pub api_url: String,
    #[serde(default)]
    pub feature_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct MyConfigValidated {
    data_path: PathBuf,
    api_url: Url,
    feature_enabled: bool,
}

impl MyConfigValidated {
    pub fn data_path(&self) -> &Path { &self.data_path }
    pub fn api_url(&self) -> &Url { &self.api_url }
    pub fn feature_enabled(&self) -> bool { self.feature_enabled }
}

impl ExtensionConfig for MyExtension {
    type Raw = MyConfigRaw;
    type Validated = MyConfigValidated;
    const PREFIX: &'static str = "my_extension";

    fn validate(raw: Self::Raw, base_path: &Path) -> Result<Self::Validated, ExtensionConfigErrors> {
        let mut errors = ExtensionConfigErrors::new(Self::PREFIX);

        let path = base_path.join(&raw.data_path);
        if !path.exists() {
            errors.push_with_path("data_path", "Path does not exist", &path);
        }

        let api_url = Url::parse(&raw.api_url)
            .map_err(|e| errors.push("api_url", e.to_string()))
            .unwrap_or_else(|_| Url::parse("https://invalid").unwrap());

        errors.into_result(MyConfigValidated {
            data_path: path.canonicalize().unwrap_or(path),
            api_url,
            feature_enabled: raw.feature_enabled,
        })
    }
}

systemprompt::extension::register_config_extension!(MyExtension);
```

---

## Step 4: Implement ExtensionError

Create `src/error.rs`:

```rust
use systemprompt_traits::ExtensionError;
use thiserror::Error;
use axum::http::StatusCode;

#[derive(Error, Debug)]
pub enum MyExtensionError {
    #[error("Item not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl ExtensionError for MyExtensionError {
    fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NOT_FOUND",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Io(_) => "IO_ERROR",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Database(_) | Self::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(self, Self::Database(_) | Self::Io(_))
    }
}
```

---

## Step 5: Create Models

Create `src/models/mod.rs`:

```rust
mod item;

pub use item::*;
```

Create `src/models/item.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use systemprompt_identifiers::ContentId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct Item {
    pub id: ContentId,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Item {
    pub const COLUMNS: &'static str = r#"
        id as "id: ContentId",
        slug,
        title,
        description,
        created_at,
        updated_at
    "#;
}

pub struct CreateItemParams {
    pub slug: String,
    pub title: String,
    pub description: String,
}

pub struct CreateItemParamsBuilder {
    slug: String,
    title: String,
    description: String,
}

impl CreateItemParamsBuilder {
    pub fn new(slug: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            slug: slug.into(),
            title: title.into(),
            description: String::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn build(self) -> CreateItemParams {
        CreateItemParams {
            slug: self.slug,
            title: self.title,
            description: self.description,
        }
    }
}

impl CreateItemParams {
    pub fn builder(slug: impl Into<String>, title: impl Into<String>) -> CreateItemParamsBuilder {
        CreateItemParamsBuilder::new(slug, title)
    }
}
```

---

## Step 6: Create Schema

Create `schema/001_items.sql`:

```sql
CREATE TABLE IF NOT EXISTS items (
    id TEXT PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_items_slug ON items(slug);
CREATE INDEX IF NOT EXISTS idx_items_created_at ON items(created_at DESC);
```

---

## Step 7: Create Repository

Create `src/repository/mod.rs`:

```rust
mod item;

pub use item::ItemRepository;
```

Create `src/repository/item.rs`:

```rust
use std::sync::Arc;
use sqlx::PgPool;
use systemprompt_identifiers::ContentId;

use crate::error::MyExtensionError;
use crate::models::{Item, CreateItemParams};

pub struct ItemRepository {
    pool: Arc<PgPool>,
}

impl ItemRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, params: &CreateItemParams) -> Result<Item, MyExtensionError> {
        let id = ContentId::new(uuid::Uuid::new_v4().to_string());

        let query = format!(
            "INSERT INTO items (id, slug, title, description)
             VALUES ($1, $2, $3, $4)
             RETURNING {}",
            Item::COLUMNS
        );

        sqlx::query_as::<_, Item>(&query)
            .bind(id.as_str())
            .bind(&params.slug)
            .bind(&params.title)
            .bind(&params.description)
            .fetch_one(&*self.pool)
            .await
            .map_err(MyExtensionError::from)
    }

    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<Item>, MyExtensionError> {
        let query = format!(
            "SELECT {} FROM items WHERE slug = $1",
            Item::COLUMNS
        );

        sqlx::query_as::<_, Item>(&query)
            .bind(slug)
            .fetch_optional(&*self.pool)
            .await
            .map_err(MyExtensionError::from)
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Item>, MyExtensionError> {
        let query = format!(
            "SELECT {} FROM items ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            Item::COLUMNS
        );

        sqlx::query_as::<_, Item>(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await
            .map_err(MyExtensionError::from)
    }

    pub async fn delete(&self, id: &ContentId) -> Result<(), MyExtensionError> {
        sqlx::query!("DELETE FROM items WHERE id = $1", id.as_str())
            .execute(&*self.pool)
            .await?;

        Ok(())
    }
}
```

---

## Step 8: Create Service

Create `src/services/mod.rs`:

```rust
mod item;

pub use item::ItemService;
```

Create `src/services/item.rs`:

```rust
use std::sync::Arc;
use sqlx::PgPool;
use systemprompt_identifiers::ContentId;

use crate::error::MyExtensionError;
use crate::models::{Item, CreateItemParams};
use crate::repository::ItemRepository;

pub struct ItemService {
    repo: ItemRepository,
}

impl ItemService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            repo: ItemRepository::new(pool),
        }
    }

    pub async fn create(&self, params: CreateItemParams) -> Result<Item, MyExtensionError> {
        if params.slug.is_empty() {
            return Err(MyExtensionError::Validation("Slug cannot be empty".into()));
        }

        if params.title.is_empty() {
            return Err(MyExtensionError::Validation("Title cannot be empty".into()));
        }

        self.repo.create(&params).await
    }

    pub async fn get_by_slug(&self, slug: &str) -> Result<Item, MyExtensionError> {
        self.repo
            .find_by_slug(slug)
            .await?
            .ok_or_else(|| MyExtensionError::NotFound(slug.to_string()))
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Item>, MyExtensionError> {
        self.repo.list(limit, offset).await
    }

    pub async fn delete(&self, id: &ContentId) -> Result<(), MyExtensionError> {
        self.repo.delete(id).await
    }
}
```

---

## Step 9: Create API Routes

Create `src/api/mod.rs`:

```rust
mod handlers;
mod types;

use std::sync::Arc;
use axum::{routing::get, Router};
use sqlx::PgPool;

pub use handlers::*;
pub use types::*;

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<PgPool>,
}

pub fn router(pool: Arc<PgPool>) -> Router {
    let state = AppState { pool };

    Router::new()
        .route("/items", get(list_items).post(create_item))
        .route("/items/:slug", get(get_item).delete(delete_item))
        .with_state(state)
}
```

Create `src/api/handlers.rs` and `src/api/types.rs` with appropriate handlers.

---

## Step 10: Create Job

Create `src/jobs/mod.rs`:

```rust
mod my_job;

pub use my_job::MyJob;
```

Create `src/jobs/my_job.rs`:

```rust
use std::sync::Arc;
use sqlx::PgPool;
use systemprompt_traits::{Job, JobContext, JobResult};

use crate::services::ItemService;

#[derive(Debug, Clone, Copy, Default)]
pub struct MyJob;

#[async_trait::async_trait]
impl Job for MyJob {
    fn name(&self) -> &'static str { "my_extension_cleanup" }
    fn description(&self) -> &'static str { "Cleans up old items" }
    fn schedule(&self) -> &'static str { "0 0 * * * *" }

    async fn execute(&self, ctx: &JobContext) -> anyhow::Result<JobResult> {
        let pool = ctx.db_pool::<PgPool>()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let service = ItemService::new(Arc::new(pool.clone()));

        tracing::info!("Running cleanup job");

        Ok(JobResult::success())
    }
}
```

---

## Step 11: Create lib.rs

Create `src/lib.rs`:

```rust
mod api;
mod config;
mod error;
mod extension;
mod jobs;
mod models;
mod repository;
mod services;

pub use config::*;
pub use error::*;
pub use extension::*;
pub use models::*;
pub use services::*;
```

---

## Step 12: Build and Test

```bash
# Build extension
cargo build -p systemprompt-my-extension

# Lint
cargo clippy -p systemprompt-my-extension -- -D warnings

# Format
cargo fmt -p systemprompt-my-extension

# Test
cargo test -p systemprompt-my-extension
```

---

## Quick Reference

| Step | File | Purpose |
|------|------|---------|
| 1 | `Cargo.toml` | Crate manifest |
| 2 | `src/extension.rs` | Extension trait |
| 3 | `src/config.rs` | Type-state config |
| 4 | `src/error.rs` | Error types |
| 5 | `src/models/` | Domain types |
| 6 | `schema/` | SQL migrations |
| 7 | `src/repository/` | Data access |
| 8 | `src/services/` | Business logic |
| 9 | `src/api/` | HTTP routes |
| 10 | `src/jobs/` | Background jobs |
| 11 | `src/lib.rs` | Public exports |
| 12 | Build/test | Verify |
