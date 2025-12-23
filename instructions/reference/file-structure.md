# Extension Internal Structure

**Applies to:** All extensions in `extensions/` (including MCP servers in `extensions/mcp/`)

---

## Core Principle

**If it's Rust code, it's an extension.**

MCP servers are Rust crates and belong in `/extensions/mcp/`, not `/services/mcp/`.

---

## Extension Directory Structure

```
extensions/{name}/
├── Cargo.toml              # Crate manifest
├── README.md               # Documentation
├── schema/                 # SQL migrations (numbered)
│   ├── 001_first_table.sql
│   ├── 002_second_table.sql
│   └── ...
└── src/
    ├── lib.rs              # Public API exports
    ├── extension.rs        # Implements Extension trait
    ├── config.rs           # Configuration types
    ├── error.rs            # Implements ExtensionError trait
    ├── models/             # Domain models
    │   ├── mod.rs
    │   ├── {entity}.rs
    │   └── builders/       # Builder patterns
    ├── repository/         # Data access layer
    │   ├── mod.rs
    │   └── {entity}.rs
    ├── services/           # Business logic
    │   ├── mod.rs
    │   └── {entity}.rs
    ├── api/                # HTTP routes
    │   ├── mod.rs
    │   ├── types.rs        # Request/response DTOs
    │   └── handlers/
    │       ├── mod.rs
    │       └── {entity}.rs
    └── jobs/               # Background jobs
        ├── mod.rs
        └── {job_name}.rs
```

---

## MCP Server Directory Structure

```
extensions/mcp/{name}/
├── Cargo.toml
├── module.yml              # Server configuration
└── src/
    ├── main.rs             # Entry point
    ├── lib.rs              # Library for testing
    ├── server/             # Server implementation
    │   ├── mod.rs
    │   ├── constructor.rs  # Server initialization
    │   └── handlers/       # Protocol handlers
    │       ├── mod.rs
    │       ├── tools.rs
    │       └── initialization.rs
    ├── tools/              # Tool implementations
    │   ├── mod.rs          # Tool registration & dispatch
    │   └── {tool_name}/    # Each tool in subdirectory
    │       ├── mod.rs
    │       ├── models.rs
    │       ├── repository.rs
    │       └── schema.rs
    ├── prompts/            # Prompt templates
    │   ├── mod.rs
    │   └── {prompt_name}.rs
    └── resources/          # Resource handlers
        ├── mod.rs
        └── {resource_name}.rs
```

---

## Required Files

| File | Purpose | Required |
|------|---------|----------|
| `Cargo.toml` | Crate manifest | Yes |
| `src/lib.rs` | Public exports | Yes |
| `src/extension.rs` | Implements `Extension` trait | Yes |
| `src/error.rs` | Implements `ExtensionError` trait | Yes |
| `README.md` | Documentation | Recommended |

---

## Required Directories

| Directory | Purpose | Required |
|-----------|---------|----------|
| `schema/` | SQL migrations | If using database |
| `src/models/` | Domain types | If any domain logic |
| `src/repository/` | Data access | If using database |
| `src/services/` | Business logic | If any business logic |
| `src/api/` | HTTP routes | If exposing API |
| `src/jobs/` | Background tasks | If scheduled work |

---

## File Placement Rules

| Rule | Description |
|------|-------------|
| Only `lib.rs`, `extension.rs`, `config.rs`, `error.rs` at `src/` root | Other files go in subdirectories |
| Every directory has `mod.rs` | Proper module hierarchy |
| No orphaned files | Every `.rs` file must be included in a `mod.rs` |
| No empty directories | Remove unused directories |

---

## Naming Conventions

### Files

| Type | Pattern | Example |
|------|---------|---------|
| Entity model | `{entity}.rs` | `content.rs` |
| Repository | `{entity}.rs` in `repository/` | `repository/content.rs` |
| Service | `{entity}.rs` in `services/` | `services/content.rs` |
| Handler | `{entity}.rs` in `api/handlers/` | `api/handlers/content.rs` |
| Job | `{job_name}.rs` | `jobs/ingestion.rs` |
| Schema | `{NNN}_{description}.sql` | `001_content_table.sql` |

### Crates

| Type | Pattern | Example |
|------|---------|---------|
| Extension crate | `systemprompt-{name}-extension` | `systemprompt-blog-extension` |
| MCP server crate | `systemprompt-mcp-{name}` | `systemprompt-mcp-admin` |

### Structs

| Type | Pattern | Example |
|------|---------|---------|
| Extension struct | `{Name}Extension` | `BlogExtension` |
| Repository struct | `{Entity}Repository` | `ContentRepository` |
| Service struct | `{Entity}Service` | `ContentService` |
| Job struct | `{Name}Job` | `ContentIngestionJob` |
| MCP Server struct | `{Name}Server` | `AdminServer` |

### Functions

| Type | Pattern | Example |
|------|---------|---------|
| Create function | `create` | `repo.create(&params)` |
| Get by ID | `get_by_id` | `repo.get_by_id(&id)` |
| Get by field | `get_by_{field}` | `repo.get_by_slug(slug)` |
| List | `list` | `repo.list(limit, offset)` |
| Delete | `delete` | `repo.delete(&id)` |

---

## Layer Responsibilities

### Models (`src/models/`)

- Domain entities
- DTOs for API requests/responses
- Builder patterns for complex types
- No database code
- No async functions

### Repository (`src/repository/`)

- SQL queries using `sqlx::query!` macros
- No business logic
- Returns domain models
- Takes `Arc<PgPool>` in constructor
- Use `COLUMNS` constant for DRY queries

```rust
impl Content {
    pub const COLUMNS: &'static str = r#"
        id as "id: ContentId", slug, title, body
    "#;
}
```

### Services (`src/services/`)

- Business logic
- Orchestrates repository calls
- Handles validation
- Maps errors to domain errors
- No direct SQL

### API (`src/api/`)

- HTTP request handling
- Request validation via extractors
- Response formatting
- No business logic
- No direct repository access

### Jobs (`src/jobs/`)

- Background task implementation
- Implements `Job` trait
- Registered via `Extension::jobs()`
- Uses services for business logic

---

## Extension Trait Implementation

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

---

## Error Type Implementation

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

---

## Anti-Patterns

| Anti-Pattern | Correct Approach |
|--------------|------------------|
| SQL in services | Move to repository |
| Business logic in handlers | Move to service |
| Repository access from handlers | Use service layer |
| `helpers.rs` or `utils.rs` | Name by domain concept |
| Flat file structure | Use subdirectories |
| `_service.rs` suffix | Just `{entity}.rs` in `services/` |
| `_repository.rs` suffix | Just `{entity}.rs` in `repository/` |
| MCP servers in `services/mcp/` | Move to `extensions/mcp/` |
| Inherent methods only | Implement `Extension` trait |
| Ad-hoc error types | Implement `ExtensionError` trait |
| Repeated SQL column lists | Use `COLUMNS` constant |

---

## Checklist

### Directory Structure

- [ ] `src/` root only has `lib.rs`, `extension.rs`, `config.rs`, `error.rs`
- [ ] Every directory has `mod.rs`
- [ ] No orphaned files
- [ ] No empty directories
- [ ] MCP servers in `extensions/mcp/`, not `services/mcp/`

### Naming

- [ ] Directories: `snake_case`
- [ ] Files: `snake_case.rs`
- [ ] No redundant suffixes
- [ ] Schema files numbered: `001_`, `002_`, etc.
- [ ] Crate names follow conventions

### Traits

- [ ] Implements `Extension` trait (not just inherent methods)
- [ ] Implements `ExtensionError` trait
- [ ] Single `register_extension!` call

### Layering

- [ ] API → Services → Repository (never skip)
- [ ] No business logic in repository
- [ ] No SQL in services
- [ ] No direct repository calls from API
- [ ] Jobs use services, not direct repository access
