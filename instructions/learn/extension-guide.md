# Building Extensions on SystemPrompt Core

This guide teaches how to build anything on top of systemprompt-core using extensions. The blog extension serves as the reference implementation.

---

## Core Principle

**If it's Rust code, it's an extension. If it's YAML/Markdown, it's a service.**

| Category | Location | Purpose |
|----------|----------|---------|
| Extensions | `/extensions/` | Rust implementations (including MCP servers) |
| Services | `/services/` | Configuration only (YAML, Markdown) |

---

## Overview

Extensions add functionality to SystemPrompt without modifying core. They provide:

| Component | Purpose |
|-----------|---------|
| **Extension trait** | Unified interface for discovery |
| **ExtensionConfig trait** | Type-safe configuration with startup validation |
| **Schemas** | Database tables owned by the extension |
| **Models** | Domain types and DTOs |
| **Repositories** | Data access layer (SQL via sqlx) |
| **Services** | Business logic |
| **API Routes** | HTTP endpoints (Axum) |
| **Jobs** | Background tasks (scheduler integration) |
| **Error types** | Consistent error handling |

---

## Project Structure

```
systemprompt-template/
├── core/                          # READ-ONLY submodule
│
├── extensions/                    # ALL Rust implementations
│   ├── blog/                      # Reference implementation
│   │   ├── Cargo.toml
│   │   ├── schema/                # SQL migrations
│   │   │   └── 001_table.sql
│   │   └── src/
│   │       ├── lib.rs             # Public exports
│   │       ├── extension.rs       # Implements Extension trait
│   │       ├── config.rs          # Configuration types
│   │       ├── error.rs           # Implements ExtensionError trait
│   │       ├── models/            # Domain models
│   │       ├── repository/        # Data access
│   │       ├── services/          # Business logic
│   │       ├── api/               # HTTP routes
│   │       │   ├── mod.rs
│   │       │   ├── handlers/
│   │       │   └── types.rs
│   │       └── jobs/              # Background jobs
│   └── mcp/                       # MCP servers (Rust crates)
│       ├── admin/
│       ├── system-tools/
│       └── infrastructure/
│
├── services/                      # PURE CONFIG (no .rs files)
│   ├── agents/                    # Agent YAML definitions
│   ├── ai/                        # AI provider config
│   ├── config/                    # Root configuration
│   ├── content/                   # Markdown content
│   ├── scheduler/                 # Job schedules (refs extension jobs)
│   ├── skills/                    # Skill definitions
│   └── web/                       # Theme config
│
└── src/
    └── main.rs                    # Server entry point
```

---

## Core Dependencies

Extensions use these crates from systemprompt-core:

```toml
[dependencies]
# Shared types (required)
systemprompt-models = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-identifiers = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-traits = { git = "https://github.com/systempromptio/systemprompt-core" }

# Database (required)
systemprompt-core-database = { git = "https://github.com/systempromptio/systemprompt-core" }

# Optional - use as needed
systemprompt-runtime = { git = "..." }       # AppContext, lifecycle
systemprompt-core-logging = { git = "..." }  # Tracing setup
systemprompt-core-config = { git = "..." }   # Config loading
```

---

## Step 1: Create Extension Crate

### Cargo.toml

```toml
[package]
name = "systemprompt-my-extension"
version = "1.0.0"
edition = "2021"

[lints]
workspace = true

[dependencies]
# Core types
systemprompt-models = { git = "https://github.com/systempromptio/systemprompt-core", branch = "main" }
systemprompt-identifiers = { git = "https://github.com/systempromptio/systemprompt-core", branch = "main" }
systemprompt-traits = { git = "https://github.com/systempromptio/systemprompt-core", branch = "main" }
systemprompt-core-database = { git = "https://github.com/systempromptio/systemprompt-core", branch = "main" }

# Web framework
axum = "0.8"
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono"] }

# Async
tokio = { version = "1.47", features = ["full"] }
async-trait = "0.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Utilities
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.18", features = ["v4", "serde"] }
thiserror = "2.0"
tracing = "0.1"
anyhow = "1.0"
```

### Add to Workspace

In root `Cargo.toml`:

```toml
[workspace]
members = [
    "extensions/my-extension",
]
```

---

## Step 2: Implement Extension Trait

### src/extension.rs

```rust
use std::sync::Arc;
use axum::Router;
use systemprompt_traits::{Extension, ExtensionContext, ExtensionMetadata, SchemaDefinition, Job};

use crate::{api, jobs::MyJob};

pub const SCHEMA_ITEMS: &str = include_str!("../schema/001_items.sql");

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
            SchemaDefinition::inline("items", SCHEMA_ITEMS),
        ]
    }

    fn router(&self, ctx: &ExtensionContext) -> Option<Router> {
        let pool = ctx.database().postgres_pool()?;
        Some(api::router(pool))
    }

    fn jobs(&self) -> Vec<Arc<dyn Job>> {
        vec![Arc::new(MyJob)]
    }
}

register_extension!(MyExtension);
```

---

## Step 3: Implement ExtensionConfig Trait (Type-State Pattern)

Extensions that need configuration implement `ExtensionConfig` using the type-state pattern. This ensures:
- Config is validated **at startup**, not runtime
- Invalid configs cause startup failure
- Validated config uses rich types (`PathBuf`, `Url`, typed IDs)
- No re-parsing at runtime - validated config is stored and reused

### src/config.rs

```rust
use serde::Deserialize;
use std::path::{Path, PathBuf};
use systemprompt::extension::typed::{ExtensionConfig, ExtensionConfigErrors};
use url::Url;

use crate::MyExtension;

// ============================================================================
// RAW CONFIG - Deserialized from profile YAML, unvalidated
// ============================================================================

/// Raw config - just deserialized, paths/URLs are strings.
#[derive(Debug, Deserialize)]
pub struct MyExtensionConfigRaw {
    pub data_path: String,
    pub api_url: String,
    #[serde(default)]
    pub enabled: bool,
}

// ============================================================================
// VALIDATED CONFIG - Paths verified, URLs parsed
// ============================================================================

/// Validated config - paths are PathBuf, URLs are parsed.
/// Can ONLY be constructed via `ExtensionConfig::validate()`.
#[derive(Debug, Clone)]
pub struct MyExtensionConfigValidated {
    data_path: PathBuf,  // Canonicalized, verified to exist
    api_url: Url,        // Parsed and validated
    enabled: bool,
}

impl MyExtensionConfigValidated {
    pub fn data_path(&self) -> &Path {
        &self.data_path
    }

    pub fn api_url(&self) -> &Url {
        &self.api_url
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

// ============================================================================
// VALIDATION - Transform Raw → Validated (consumes Raw)
// ============================================================================

impl ExtensionConfig for MyExtension {
    type Raw = MyExtensionConfigRaw;
    type Validated = MyExtensionConfigValidated;

    const PREFIX: &'static str = "my_extension";

    fn validate(raw: Self::Raw, base_path: &Path) -> Result<Self::Validated, ExtensionConfigErrors> {
        let mut errors = ExtensionConfigErrors::new(Self::PREFIX);

        // Resolve and validate path
        let resolved_path = if Path::new(&raw.data_path).is_absolute() {
            PathBuf::from(&raw.data_path)
        } else {
            base_path.join(&raw.data_path)
        };

        if !resolved_path.exists() {
            errors.push_with_path("data_path", "Path does not exist", &resolved_path);
        }

        // Parse and validate URL
        let api_url = match Url::parse(&raw.api_url) {
            Ok(url) => url,
            Err(e) => {
                errors.push("api_url", format!("Invalid URL: {}", e));
                Url::parse("https://invalid.example.com").unwrap()
            }
        };

        // Return validated config or errors
        errors.into_result(MyExtensionConfigValidated {
            data_path: resolved_path.canonicalize().unwrap_or(resolved_path),
            api_url,
            enabled: raw.enabled,
        })
    }
}
```

### Register Config Extension

In `src/extension.rs`, add the config registration:

```rust
register_extension!(MyExtension);
register_config_extension!(MyExtension);  // Validates config at startup
```

### Profile Configuration

Extension config is stored in the profile under `extensions.{PREFIX}`:

```yaml
# profiles/local.secrets.profile.yml

name: local
display_name: "Local Development"

paths:
  # ... standard paths ...

extensions:
  my_extension:
    data_path: data/my-extension    # Relative to services path
    api_url: https://api.example.com
    enabled: true
```

### Using Validated Config in Jobs

Jobs receive the **already validated** config - no parsing, guaranteed valid:

```rust
#[async_trait::async_trait]
impl Job for MyJob {
    async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
        // Get validated config - paths guaranteed to exist
        let config: Arc<MyExtensionConfigValidated> = ctx
            .extension_config::<MyExtension>()
            .ok_or_else(|| anyhow::anyhow!("Config not found"))?;

        // Use validated paths directly - no existence check needed
        let data = std::fs::read_to_string(config.data_path())?;

        Ok(JobResult::success())
    }
}
```

### Type-State Summary

```
┌─────────────────────────────────────────────────────────────┐
│  MyExtensionConfigRaw                                       │
│  ├── data_path: String        ← Unvalidated                │
│  ├── api_url: String          ← Unvalidated                │
│  └── #[derive(Deserialize)]   ← Can parse from YAML        │
│                                                             │
│              │ ExtensionConfig::validate()                  │
│              │ (consumes Raw, produces Validated)           │
│              ▼                                              │
│                                                             │
│  MyExtensionConfigValidated                                 │
│  ├── data_path: PathBuf       ← Canonicalized, exists      │
│  ├── api_url: Url             ← Parsed, valid              │
│  └── NO Deserialize           ← Only via validate()        │
│                                                             │
│  IMPOSSIBLE to have invalid config at runtime              │
└─────────────────────────────────────────────────────────────┘
```

---

## Step 4: Implement ExtensionError Trait

> **Note:** Step 3 (ExtensionConfig) is optional - only implement if your extension needs configuration.

### src/error.rs

```rust
use systemprompt_traits::ExtensionError;
use thiserror::Error;
use axum::http::StatusCode;

#[derive(Error, Debug)]
pub enum MyExtensionError {
    #[error("Item not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl ExtensionError for MyExtensionError {
    fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NOT_FOUND",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Database(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(self, Self::Database(_))
    }
}
```

---

## Step 4: Define Models

### src/models/item.rs

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt_identifiers::ItemId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Item {
    /// SQL column list for SELECT queries
    pub const COLUMNS: &'static str = r#"
        id as "id: ItemId", name, description, created_at, updated_at
    "#;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateItemParams {
    pub name: String,
    pub description: Option<String>,
}
```

**Rules:**
- Use typed identifiers from `systemprompt_identifiers`
- Use `DateTime<Utc>` for timestamps
- Use `COLUMNS` constant for DRY SQL queries
- Use builders for types with 3+ fields

---

## Step 5: Create Schema

### schema/001_items.sql

```sql
CREATE TABLE IF NOT EXISTS items (
    id VARCHAR(36) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_items_name ON items(name);
```

---

## Step 6: Implement Repository

### src/repository/item.rs

```rust
use std::sync::Arc;
use sqlx::PgPool;
use chrono::Utc;
use systemprompt_identifiers::ItemId;

use crate::models::{Item, CreateItemParams};

#[derive(Debug, Clone)]
pub struct ItemRepository {
    pool: Arc<PgPool>,
}

impl ItemRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, params: &CreateItemParams) -> Result<Item, sqlx::Error> {
        let id = ItemId::new(uuid::Uuid::new_v4().to_string());
        let now = Utc::now();

        let query = format!(
            "INSERT INTO items (id, name, description, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING {}",
            Item::COLUMNS
        );

        sqlx::query_as::<_, Item>(&query)
            .bind(id.as_str())
            .bind(&params.name)
            .bind(&params.description)
            .bind(now)
            .bind(now)
            .fetch_one(&*self.pool)
            .await
    }

    pub async fn get_by_id(&self, id: &ItemId) -> Result<Option<Item>, sqlx::Error> {
        let query = format!(
            "SELECT {} FROM items WHERE id = $1",
            Item::COLUMNS
        );

        sqlx::query_as::<_, Item>(&query)
            .bind(id.as_str())
            .fetch_optional(&*self.pool)
            .await
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Item>, sqlx::Error> {
        let query = format!(
            "SELECT {} FROM items ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            Item::COLUMNS
        );

        sqlx::query_as::<_, Item>(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await
    }
}
```

**Rules:**
- Repository takes `Arc<PgPool>` in constructor
- Use `COLUMNS` constant for DRY queries
- Use typed identifiers in bindings
- No business logic in repositories

---

## Step 7: Implement Service

### src/services/item.rs

```rust
use std::sync::Arc;
use sqlx::PgPool;
use systemprompt_identifiers::ItemId;

use crate::error::MyExtensionError;
use crate::models::{Item, CreateItemParams};
use crate::repository::ItemRepository;

#[derive(Debug, Clone)]
pub struct ItemService {
    repo: ItemRepository,
}

impl ItemService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            repo: ItemRepository::new(pool),
        }
    }

    pub async fn create(&self, params: &CreateItemParams) -> Result<Item, MyExtensionError> {
        self.repo.create(params).await.map_err(MyExtensionError::from)
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Item>, MyExtensionError> {
        let id = ItemId::new(id.to_string());
        self.repo.get_by_id(&id).await.map_err(MyExtensionError::from)
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Item>, MyExtensionError> {
        self.repo.list(limit, offset).await.map_err(MyExtensionError::from)
    }
}
```

**Rules:**
- Services inject repositories via constructor
- Services contain business logic
- Services NEVER execute SQL directly
- Map repository errors to extension errors

---

## Step 8: Create API Routes

### src/api/handlers/item.rs

```rust
use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use systemprompt_traits::ExtensionError;

use crate::api::MyExtensionState;
use crate::models::CreateItemParams;
use crate::services::ItemService;

pub async fn create_item(
    State(state): State<MyExtensionState>,
    Json(params): Json<CreateItemParams>,
) -> Response {
    let service = ItemService::new(state.pool.clone());

    match service.create(&params).await {
        Ok(item) => (StatusCode::CREATED, Json(item)).into_response(),
        Err(e) => (e.status(), Json(serde_json::json!({"error": e.to_string(), "code": e.code()}))).into_response(),
    }
}

pub async fn get_item(
    State(state): State<MyExtensionState>,
    Path(id): Path<String>,
) -> Response {
    let service = ItemService::new(state.pool.clone());

    match service.get_by_id(&id).await {
        Ok(Some(item)) => Json(item).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Not found"}))).into_response(),
        Err(e) => (e.status(), Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}
```

**Rules:**
- Handlers follow: extract → delegate → respond
- No business logic in handlers
- No direct repository access
- Use `ExtensionError::status()` for HTTP codes

---

## Step 9: Implement Background Jobs

### src/jobs/my_job.rs

```rust
use std::sync::Arc;
use anyhow::Result;
use sqlx::PgPool;
use systemprompt_traits::{Job, JobContext, JobResult};

#[derive(Debug, Clone, Copy, Default)]
pub struct MyJob;

#[async_trait::async_trait]
impl Job for MyJob {
    fn name(&self) -> &'static str {
        "my_extension_job"
    }

    fn description(&self) -> &'static str {
        "Performs scheduled work for my extension"
    }

    fn schedule(&self) -> &'static str {
        "0 0 * * * *"  // Default: every hour
    }

    async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
        let pool = ctx
            .db_pool::<PgPool>()
            .ok_or_else(|| anyhow::anyhow!("Database pool not available"))?;

        tracing::info!("My job started");

        // Use services, not direct repository access
        let service = crate::services::ItemService::new(Arc::new(pool.clone()));
        let items = service.list(100, 0).await?;

        tracing::info!(count = items.len(), "My job completed");

        Ok(JobResult::success()
            .with_stats(items.len() as u64, 0))
    }
}
```

Jobs are registered via `Extension::jobs()` and scheduled via YAML:

```yaml
# services/scheduler/config.yml
scheduler:
  jobs:
    - extension: my_extension
      job: my_extension_job
      schedule: "0 */15 * * * *"  # Override: every 15 minutes
      enabled: true
```

---

## Step 10: Public API (lib.rs)

### src/lib.rs

```rust
#![allow(clippy::module_name_repetitions)]

pub mod api;
pub mod config;
pub mod error;
pub mod extension;
pub mod jobs;
pub mod models;
pub mod repository;
pub mod services;

// Re-export main types
pub use config::MyExtensionConfig;
pub use error::MyExtensionError;
pub use extension::MyExtension;

// Re-export models
pub use models::{Item, CreateItemParams};

// Re-export services
pub use services::ItemService;

// Re-export repositories
pub use repository::ItemRepository;

// Re-export jobs
pub use jobs::MyJob;
```

---

## Checklist: New Extension

- [ ] Create `extensions/{name}/` directory
- [ ] Add `Cargo.toml` with core dependencies
- [ ] Add to workspace in root `Cargo.toml`
- [ ] Implement `Extension` trait in `src/extension.rs`
- [ ] Implement `ExtensionConfig` trait in `src/config.rs` (if config needed)
  - [ ] Define `ConfigRaw` type with `#[derive(Deserialize)]`
  - [ ] Define `ConfigValidated` type with rich types (`PathBuf`, `Url`)
  - [ ] Implement `validate()` to transform Raw → Validated
  - [ ] Add `register_config_extension!` call
- [ ] Implement `ExtensionError` trait in `src/error.rs`
- [ ] Create `schema/` directory with migrations
- [ ] Create `src/models/` with domain types (use `COLUMNS` constant)
- [ ] Create `src/repository/` with data access
- [ ] Create `src/services/` with business logic
- [ ] Create `src/api/` with HTTP routes
- [ ] Create `src/jobs/` if background processing needed
- [ ] Create `src/lib.rs` with public exports
- [ ] Single `register_extension!` call (+ `register_config_extension!` if config)

---

## Reference: Blog Extension

The blog extension (`extensions/blog/`) demonstrates all patterns:

| Feature | Location |
|---------|----------|
| Extension trait impl | `src/extension.rs` |
| ExtensionConfig impl | `src/config.rs` (Raw/Validated types) |
| ExtensionError impl | `src/error.rs` |
| Schemas (7 tables) | `schema/*.sql` |
| Models with `COLUMNS` | `src/models/` |
| Repositories | `src/repository/` |
| Services | `src/services/` |
| API handlers | `src/api/` |
| Background job | `src/jobs/ingestion.rs` |
| Content validation | `src/services/validation.rs` |

### Blog Config Example

The blog extension config in profile:

```yaml
extensions:
  blog:
    base_url: https://myblog.com
    enable_link_tracking: true
    content_sources:
      - source_id: blog
        category_id: blog
        path: content/blog       # Relative to services path
        enabled: true
      - source_id: guides
        category_id: guides
        path: content/guides
        enabled: true
```

At startup, Core validates:
- All enabled `content_sources[].path` directories exist
- `base_url` is a valid URL with http/https scheme
- All `source_id` and `category_id` are non-empty

If validation fails, the app refuses to start with actionable error messages.
