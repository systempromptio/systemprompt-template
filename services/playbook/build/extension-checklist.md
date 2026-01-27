---
title: "Extension Checklist Playbook"
description: "Complete checklist for building extensions on systemprompt-core."
keywords:
  - extension
  - checklist
  - build
  - validation
---

# Extension Checklist

**Applies to:** All crates in `extensions/`

> **Help**: `{ "command": "playbook build" }` via `systemprompt_help`

---

## Core Principle

Extensions implement the unified `Extension` trait from `systemprompt-traits`. Use trait-based polymorphism, not inherent methods.

---

## Required Structure

- [ ] `Cargo.toml` exists with correct dependencies
- [ ] `src/lib.rs` exports public API
- [ ] `src/extension.rs` implements `Extension` trait
- [ ] `src/config.rs` implements `ExtensionConfig` trait (if config needed)
- [ ] `src/error.rs` implements `ExtensionError` trait
- [ ] `schema/` directory with numbered SQL migrations (if using database)

---

## Cargo.toml

- [ ] Package name follows `systemprompt-{name}-extension` pattern
- [ ] Core dependencies via git:
  - `systemprompt-models`
  - `systemprompt-identifiers`
  - `systemprompt-traits`
  - `systemprompt-core-database`
- [ ] No forbidden dependencies (see boundaries)
- [ ] `[lints] workspace = true` for shared lint config

---

## Extension Trait Implementation

Extensions must implement the unified `Extension` trait:

```rust
use systemprompt_traits::{Extension, ExtensionContext, ExtensionMetadata, SchemaDefinition};
use std::sync::Arc;

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
            SchemaDefinition::inline("table", include_str!("../schema/001_table.sql")),
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

### Checklist

- [ ] Implements `Extension` trait (NOT just inherent methods)
- [ ] `metadata()` returns unique ID, name, version
- [ ] `schemas()` returns list of `SchemaDefinition` (if using database)
- [ ] `router()` returns `Option<Router>` via `ExtensionContext`
- [ ] `jobs()` returns list of `Arc<dyn Job>` (if background tasks)
- [ ] Single `register_extension!` call

---

## ExtensionConfig Trait Implementation (If Config Needed)

Extensions with configuration implement `ExtensionConfig` using the type-state pattern:

```rust
use serde::Deserialize;
use std::path::{Path, PathBuf};
use systemprompt::extension::typed::{ExtensionConfig, ExtensionConfigErrors};
use url::Url;

// RAW - Deserialized from profile YAML, unvalidated
#[derive(Debug, Deserialize)]
pub struct MyConfigRaw {
    pub data_path: String,      // String, not PathBuf
    pub api_url: String,        // String, not Url
}

// VALIDATED - Paths verified, URLs parsed, cannot be invalid
#[derive(Debug, Clone)]
pub struct MyConfigValidated {
    data_path: PathBuf,         // Canonicalized, verified to exist
    api_url: Url,               // Parsed and validated
}

impl ExtensionConfig for MyExtension {
    type Raw = MyConfigRaw;
    type Validated = MyConfigValidated;
    const PREFIX: &'static str = "my_extension";

    fn validate(raw: Self::Raw, base_path: &Path) -> Result<Self::Validated, ExtensionConfigErrors> {
        let mut errors = ExtensionConfigErrors::new(Self::PREFIX);

        // Validate path exists
        let path = base_path.join(&raw.data_path);
        if !path.exists() {
            errors.push_with_path("data_path", "Path does not exist", &path);
        }

        // Parse URL
        let url = Url::parse(&raw.api_url)
            .map_err(|e| errors.push("api_url", e.to_string()))
            .unwrap_or_else(|_| Url::parse("https://invalid").unwrap());

        errors.into_result(MyConfigValidated {
            data_path: path.canonicalize().unwrap_or(path),
            api_url: url,
        })
    }
}

register_config_extension!(MyExtension);
```

### Checklist

- [ ] `Raw` type has `#[derive(Deserialize)]` with `String` for paths/URLs
- [ ] `Validated` type has `PathBuf`, `Url`, typed IDs (NO `Deserialize`)
- [ ] `validate()` consumes `Raw` and produces `Validated`
- [ ] All paths validated to exist (for enabled features)
- [ ] All URLs parsed and scheme validated
- [ ] All errors collected (not just first failure)
- [ ] `register_config_extension!` call added
- [ ] Config stored in profile under `extensions.{PREFIX}`

---

## ExtensionError Trait Implementation

Error types must implement `ExtensionError` for consistent handling:

```rust
use systemprompt_traits::ExtensionError;
use thiserror::Error;
use axum::http::StatusCode;

#[derive(Error, Debug)]
pub enum MyExtensionError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation: {0}")]
    Validation(String),
}

impl ExtensionError for MyExtensionError {
    fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NOT_FOUND",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Validation(_) => "VALIDATION_ERROR",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(self, Self::Database(_))
    }
}
```

### Checklist

- [ ] Uses `thiserror` for error derivation
- [ ] Implements `ExtensionError` trait
- [ ] `code()` returns machine-readable error code
- [ ] `status()` returns appropriate HTTP status
- [ ] `is_retryable()` indicates transient errors
- [ ] Implements `From<sqlx::Error>` for database errors

---

## Repository Quality

- [ ] All queries use SQLX macros (`query!`, `query_as!`, `query_scalar!`)
- [ ] No runtime query strings (`sqlx::query()` without `!`)
- [ ] No business logic in repositories
- [ ] Typed IDs used (not raw strings)
- [ ] Pool is `Arc<PgPool>`
- [ ] Column casts for typed IDs: `id as "id: ContentId"`
- [ ] Uses `COLUMNS` constant for DRY queries

```rust
impl Content {
    pub const COLUMNS: &'static str = r#"
        id as "id: ContentId", slug, title, description, body
    "#;
}

impl ContentRepository {
    pub async fn get_by_id(&self, id: &ContentId) -> Result<Option<Content>> {
        let query = format!("SELECT {} FROM content WHERE id = $1", Content::COLUMNS);
        sqlx::query_as::<_, Content>(&query)
            .bind(id.as_str())
            .fetch_optional(&*self.pool)
            .await
    }
}
```

---

## Service Quality

- [ ] Repositories injected via constructor
- [ ] No direct SQL in services
- [ ] Errors mapped to domain error types
- [ ] Structured logging with `tracing`
- [ ] Business logic contained in services, not handlers

---

## API Quality

- [ ] Handlers follow: extract -> delegate -> respond
- [ ] No business logic in handlers
- [ ] No direct repository access from handlers
- [ ] Service called for all operations
- [ ] Proper error conversion using `ExtensionError::status()`
- [ ] Typed request/response models

---

## Job Quality (if applicable)

- [ ] Implements `Job` trait from `systemprompt_traits`
- [ ] `name()` returns unique job identifier
- [ ] `description()` returns human-readable description
- [ ] `schedule()` returns valid cron expression (default)
- [ ] `execute()` uses `ctx.db_pool::<PgPool>()?`
- [ ] Registered via `Extension::jobs()` method
- [ ] Uses services for business logic

```rust
use systemprompt_traits::{Job, JobContext, JobResult};

#[derive(Debug, Clone, Copy, Default)]
pub struct MyJob;

#[async_trait::async_trait]
impl Job for MyJob {
    fn name(&self) -> &'static str { "my_job" }
    fn description(&self) -> &'static str { "Does something" }
    fn schedule(&self) -> &'static str { "0 0 * * * *" }

    async fn execute(&self, ctx: &JobContext) -> anyhow::Result<JobResult> {
        let pool = ctx.db_pool::<PgPool>()
            .ok_or_else(|| anyhow::anyhow!("DB not available"))?;
        // Use services, not direct repository access
        Ok(JobResult::success())
    }
}
```

Jobs are configured in YAML (schedule override):

```yaml
# services/scheduler/config.yml
scheduler:
  jobs:
    - extension: my_extension
      job: my_job
      schedule: "0 */15 * * * *"  # Override default
      enabled: true
```

---

## Model Quality

- [ ] All IDs use typed wrappers from `systemprompt_identifiers`
- [ ] No `String` for domain identifiers
- [ ] `DateTime<Utc>` for timestamps
- [ ] Builders for types with 3+ fields
- [ ] Derive ordering: `Debug, Clone, PartialEq, Eq, Serialize, Deserialize`

---

## Boundary Rules

- [ ] No entry layer imports (`systemprompt-core-api`)
- [ ] No app layer imports (`systemprompt-core-scheduler`)
- [ ] No direct imports of core domain crates
- [ ] Only `shared/` and `infra/` dependencies from core
- [ ] Other extensions imported via public API only
- [ ] Extension lives in `extensions/`, not `services/`

---

## Idiomatic Rust

- [ ] Iterator chains over imperative loops
- [ ] `?` operator for error propagation
- [ ] No unnecessary `.clone()`
- [ ] `impl Into<T>` for flexible APIs
- [ ] Combinators (`map`, `and_then`, `ok_or`) over match
- [ ] Unified `Extension` trait (not multiple separate traits)
- [ ] `COLUMNS` constant for SQL (not repeated strings)

---

## Code Quality

- [ ] File length <= 300 lines
- [ ] Function length <= 75 lines
- [ ] Cognitive complexity <= 15
- [ ] Function parameters <= 5
- [ ] No `unsafe`
- [ ] No `unwrap()` / `panic!()`
- [ ] No inline comments (`//`)
- [ ] No TODO/FIXME/HACK
- [ ] `cargo clippy -p {crate} -- -D warnings` passes
- [ ] `cargo fmt -p {crate} -- --check` passes

---

## Quick Reference

| Task | Command |
|------|---------|
| Build | `cargo build -p systemprompt-{name}-extension` |
| Test | `cargo test -p systemprompt-{name}-extension` |
| Lint | `cargo clippy -p systemprompt-{name}-extension -- -D warnings` |
| Format | `cargo fmt -p systemprompt-{name}-extension -- --check` |
