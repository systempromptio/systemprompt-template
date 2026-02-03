---
title: "Extension Checklist Playbook"
description: "Complete checklist for building extensions on systemprompt-core."
keywords:
  - extension
  - checklist
  - build
  - validation
category: build
---

# Extension Checklist

**Applies to:** All crates in `extensions/`

> **Help**: `{ "command": "core playbooks show build_extension-checklist" }`

-> Core traits: [systemprompt-traits](https://github.com/systempromptio/systemprompt-core/tree/main/crates/shared/traits)

---

## Core Principle

Extensions implement the unified `Extension` trait. Use trait-based polymorphism, not inherent methods.

---

## Directory Structure

```
extensions/{name}/
├── Cargo.toml              # Crate manifest
├── schema/                 # SQL migrations (if database)
│   ├── 001_first_table.sql
│   └── 002_second_table.sql
└── src/
    ├── lib.rs              # Public exports
    ├── extension.rs        # Extension trait impl
    ├── config.rs           # ExtensionConfig impl (optional)
    ├── error.rs            # ExtensionError impl
    ├── models/             # Domain types
    ├── repository/         # Data access
    ├── services/           # Business logic
    ├── api/                # HTTP routes
    └── jobs/               # Background tasks
```

> **Reference**: See `extensions/web/` for a complete working example.

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

    fn page_prerenderers(&self) -> Vec<Arc<dyn PagePrerenderer>> {
        vec![Arc::new(MyPagePrerenderer)]
    }

    fn page_data_providers(&self) -> Vec<Arc<dyn PageDataProvider>> {
        vec![Arc::new(MyDataProvider)]
    }

    fn component_renderers(&self) -> Vec<Arc<dyn ComponentRenderer>> {
        vec![Arc::new(MyComponent)]
    }

    fn frontmatter_processors(&self) -> Vec<Arc<dyn FrontmatterProcessor>> {
        vec![Arc::new(MyFrontmatterProcessor)]
    }

    fn content_data_providers(&self) -> Vec<Arc<dyn ContentDataProvider>> {
        vec![Arc::new(MyContentDataProvider)]
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
- [ ] `page_prerenderers()` returns list of `Arc<dyn PagePrerenderer>` (if rendering pages)
- [ ] `page_data_providers()` returns list of `Arc<dyn PageDataProvider>` (if providing page data)
- [ ] `component_renderers()` returns list of `Arc<dyn ComponentRenderer>` (if rendering components)
- [ ] `frontmatter_processors()` returns list of `Arc<dyn FrontmatterProcessor>` (if parsing custom frontmatter)
- [ ] `content_data_providers()` returns list of `Arc<dyn ContentDataProvider>` (if enriching content)
- [ ] Single `register_extension!` call

---

## ExtensionConfig Trait Implementation (If Config Needed)

Extensions with configuration implement `ExtensionConfig` using the type-state pattern:

```rust
use serde::Deserialize;
use std::path::{Path, PathBuf};
use systemprompt::extension::typed::{ExtensionConfig, ExtensionConfigErrors};
use url::Url;

#[derive(Debug, Deserialize)]
pub struct MyConfigRaw {
    pub data_path: String,
    pub api_url: String,
}

#[derive(Debug, Clone)]
pub struct MyConfigValidated {
    data_path: PathBuf,
    api_url: Url,
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
        Ok(JobResult::success())
    }
}
```

Jobs are configured in YAML (schedule override):

```yaml
scheduler:
  jobs:
    - extension: my_extension
      job: my_job
      schedule: "0 */15 * * * *"
      enabled: true
```

---

## Page Prerenderer (if rendering pages)

Extensions can own and render pages by implementing `PagePrerenderer`:

```rust
use std::path::PathBuf;
use anyhow::Result;
use async_trait::async_trait;
use systemprompt_provider_contracts::{PagePrepareContext, PagePrerenderer, PageRenderSpec};

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
        100  // Lower = earlier execution
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

### Checklist

- [ ] Implements `PagePrerenderer` trait
- [ ] `page_type()` returns unique page identifier
- [ ] `priority()` returns render order (100 is default)
- [ ] `prepare()` returns `PageRenderSpec` with template, data, output path
- [ ] Return `Ok(None)` to skip rendering (feature disabled, template missing)
- [ ] Registered via `Extension::page_prerenderers()` method
- [ ] Template exists in `services/web/templates/`

---

## Page Data Provider (if providing data to pages)

Extensions can provide data to pages without owning the prerender:

```rust
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use systemprompt_template_provider::{PageContext, PageDataProvider};

#[derive(Debug, Clone, Copy, Default)]
pub struct MyDataProvider;

#[async_trait]
impl PageDataProvider for MyDataProvider {
    fn provider_id(&self) -> &str {
        "my-data"
    }

    fn applies_to_pages(&self) -> Vec<String> {
        vec!["homepage".to_string(), "docs-index".to_string()]
    }

    async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
        Ok(serde_json::json!({
            "my_field": "value",
            "nested": { "data": 123 }
        }))
    }
}
```

### Checklist

- [ ] Implements `PageDataProvider` trait
- [ ] `provider_id()` returns unique provider identifier
- [ ] `applies_to_pages()` returns list of page types this provider serves
- [ ] `provide_page_data()` returns JSON data to merge into page context
- [ ] Registered via `Extension::page_data_providers()` method
- [ ] Data is merged recursively with base page data

---

## Frontmatter Processor (if parsing custom frontmatter)

Extensions can hook into content ingestion to parse and store custom frontmatter fields:

```rust
use anyhow::Result;
use async_trait::async_trait;
use systemprompt_database::DbPool;
use systemprompt_provider_contracts::{FrontmatterContext, FrontmatterProcessor};

#[derive(Debug, Clone, Copy, Default)]
pub struct MyFrontmatterProcessor;

#[async_trait]
impl FrontmatterProcessor for MyFrontmatterProcessor {
    fn processor_id(&self) -> &'static str {
        "my-frontmatter"
    }

    fn applies_to_sources(&self) -> Vec<String> {
        vec!["docs".to_string()]  // or vec![] for all sources
    }

    fn priority(&self) -> u32 {
        100  // Lower = earlier execution
    }

    async fn process_frontmatter(&self, ctx: &FrontmatterContext<'_>) -> Result<()> {
        let db = ctx.db_pool::<DbPool>()
            .ok_or_else(|| anyhow::anyhow!("DB not available"))?;

        // Extract custom field from raw YAML
        if let Some(my_field) = ctx.raw_frontmatter().get("my_custom_field") {
            sqlx::query!(
                "INSERT INTO my_metadata (content_id, custom_field)
                 VALUES ($1, $2)
                 ON CONFLICT (content_id) DO UPDATE SET custom_field = $2",
                ctx.content_id(),
                my_field.as_str()
            )
            .execute(db.as_ref())
            .await?;
        }
        Ok(())
    }
}
```

### Checklist

- [ ] Implements `FrontmatterProcessor` trait
- [ ] `processor_id()` returns unique processor identifier
- [ ] `applies_to_sources()` returns list of content sources (empty = all)
- [ ] `process_frontmatter()` extracts and stores custom fields
- [ ] Uses `ctx.raw_frontmatter()` to access raw YAML
- [ ] Registered via `Extension::frontmatter_processors()` method
- [ ] Extension owns its own database table for custom fields

---

## Content Data Provider (if enriching content JSON)

Extensions can enrich content items with additional data during prerendering:

```rust
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use systemprompt_database::DbPool;
use systemprompt_provider_contracts::{ContentDataContext, ContentDataProvider};

#[derive(Debug, Clone, Copy, Default)]
pub struct MyContentDataProvider;

#[async_trait]
impl ContentDataProvider for MyContentDataProvider {
    fn provider_id(&self) -> &'static str {
        "my-content-data"
    }

    fn applies_to_sources(&self) -> Vec<String> {
        vec!["docs".to_string()]  // or vec![] for all sources
    }

    fn priority(&self) -> u32 {
        100  // Lower = earlier execution
    }

    async fn enrich_content(
        &self,
        ctx: &ContentDataContext<'_>,
        item: &mut Value,
    ) -> Result<()> {
        let db = ctx.db_pool::<DbPool>()
            .ok_or_else(|| anyhow::anyhow!("DB not available"))?;

        // Fetch from extension table and add to content JSON
        let data = sqlx::query_scalar!(
            "SELECT custom_field FROM my_metadata WHERE content_id = $1",
            ctx.content_id()
        )
        .fetch_optional(db.as_ref())
        .await?;

        if let Some(value) = data.flatten() {
            item["my_custom_field"] = Value::String(value);
        }
        Ok(())
    }
}
```

### Checklist

- [ ] Implements `ContentDataProvider` trait
- [ ] `provider_id()` returns unique provider identifier
- [ ] `applies_to_sources()` returns list of content sources (empty = all)
- [ ] `enrich_content()` adds fields to the content JSON item
- [ ] Registered via `Extension::content_data_providers()` method
- [ ] Data comes from extension's own table (populated by FrontmatterProcessor)

---

## Component Renderer (if rendering HTML fragments)

Extensions can render HTML fragments for pages:

```rust
use anyhow::Result;
use async_trait::async_trait;
use systemprompt_template_provider::{ComponentContext, ComponentRenderer, RenderedComponent};

#[derive(Debug, Clone, Copy, Default)]
pub struct HeroComponent;

#[async_trait]
impl ComponentRenderer for HeroComponent {
    fn component_id(&self) -> &str {
        "hero-section"
    }

    fn variable_name(&self) -> &str {
        "HERO_HTML"
    }

    fn applies_to(&self) -> Vec<String> {
        vec!["homepage".to_string()]
    }

    async fn render(&self, ctx: &ComponentContext<'_>) -> Result<RenderedComponent> {
        let html = format!(
            r#"<section class="hero"><h1>{}</h1></section>"#,
            ctx.web_config.branding.display_sitename
        );
        Ok(RenderedComponent::new("HERO_HTML", html))
    }
}
```

### Checklist

- [ ] Implements `ComponentRenderer` trait
- [ ] `component_id()` returns unique component identifier
- [ ] `variable_name()` returns template variable name (e.g., `HERO_HTML`)
- [ ] `applies_to()` returns list of page types this component serves
- [ ] `render()` returns `RenderedComponent` with variable name and HTML
- [ ] Registered via `Extension::component_renderers()` method
- [ ] HTML is inserted into page data under the variable name

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

## Reference Implementations

| Concept | Location |
|---------|----------|
| Extension trait | `extensions/web/src/extension.rs` |
| ExtensionError | `extensions/web/src/error.rs` |
| Repository | `extensions/web/src/repository/` |
| Service | `extensions/web/src/services/` |
| API | `extensions/web/src/api/` |
| Jobs | `extensions/web/src/jobs/` |

-> See [Architecture](build_architecture) for layer model and dependency rules.
-> See [Rust Standards](build_rust-standards) for code quality.
-> See [Extension Review](build_extension-review) for code review process.
