---
title: "Extension Boundaries Playbook"
description: "Rules for what extensions can and cannot do in systemprompt-core."
keywords:
  - boundaries
  - extensions
  - rules
  - constraints
---

# Extension Boundaries

This document defines what extensions can and cannot do, and how to integrate with core services.

> **Help**: `{ "command": "playbook build" }` via `systemprompt_help`

---

## Core Principle: Separation of Concerns

**If it's Rust code, it belongs in `/extensions/`. If it's YAML/Markdown, it belongs in `/services/`.**

| Category | Purpose | Location | Examples |
|----------|---------|----------|----------|
| **Extensions** | Rust implementations | `/extensions/` | Blog, MCP servers |
| **Services** | Declarative config | `/services/` | Agent YAML, theme config |

---

## Directory Boundaries

### Extensions Directory (`/extensions/`)

Contains ALL Rust code:

```
extensions/
├── blog/                      # Content management extension
└── mcp/                       # MCP server implementations
    ├── admin/                 # Admin tools
    ├── system-tools/          # File operations
    └── infrastructure/        # Deployment tools
```

### Services Directory (`/services/`)

Contains ONLY configuration (no `.rs` files):

```
services/
├── agents/                    # Agent YAML definitions
├── ai/                        # AI provider config
├── config/                    # Root config aggregator
├── content/                   # Markdown content
├── scheduler/                 # Job schedules (refs extension jobs)
├── skills/                    # Skill definitions
└── web/                       # Theme config
```

---

## Core Principles

### 1. Core is Read-Only

The `core/` directory is a git submodule. Never modify it directly.

### 2. Extensions Own Their Data

Each extension owns its database tables. No extension should directly access another extension's tables.

### 3. Use Core Abstractions

Access core functionality through traits and public APIs, not internal implementations.

### 4. Implement Core Traits

Extensions should implement the unified `Extension` trait, not just use inherent methods.

---

## What Extensions Can Do

### Implement Extension Trait

Extensions receive context and provide capabilities:

```rust
use systemprompt_traits::{Extension, ExtensionContext, ExtensionMetadata};

impl Extension for MyExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "my_extension",
            name: "My Extension",
            version: "1.0.0",
            ..Default::default()
        }
    }

    fn schemas(&self) -> Vec<SchemaDefinition> {
        vec![SchemaDefinition::inline("my_table", include_str!("../schema/001_table.sql"))]
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

### Define HTTP Routes

Extensions can define API routes mounted by the server:

```rust
pub fn router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route("/items", get(list_items))
        .route("/items/:id", get(get_item))
        .with_state(MyState { pool })
}
```

### Register Background Jobs

Jobs are defined in Rust and scheduled via YAML:

```rust
use systemprompt_traits::{Job, JobContext, JobResult};

#[derive(Default)]
pub struct MyJob;

#[async_trait::async_trait]
impl Job for MyJob {
    fn name(&self) -> &'static str { "my_job" }
    fn schedule(&self) -> &'static str { "0 0 * * * *" }

    async fn execute(&self, ctx: &JobContext) -> anyhow::Result<JobResult> {
        let pool = ctx.db_pool::<PgPool>()?;
        Ok(JobResult::success())
    }
}
```

```yaml
# services/scheduler/config.yml
scheduler:
  jobs:
    - extension: my_extension
      job: my_job
      schedule: "0 */15 * * * *"  # Override default
      enabled: true
```

### Implement ExtensionError Trait

Use consistent error handling:

```rust
use systemprompt_traits::ExtensionError;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Database: {0}")]
    Database(#[from] sqlx::Error),
}

impl ExtensionError for MyError {
    fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NOT_FOUND",
            Self::Database(_) => "DATABASE_ERROR",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(self, Self::Database(_))
    }
}
```

---

## What Extensions Cannot Do

### Modify Core Code

| Forbidden | Alternative |
|-----------|-------------|
| Edit files in `core/` | Create extension with new functionality |
| Fork core for changes | Submit PR to systemprompt-core |
| Copy core code locally | Import from core crates |

### Access Core Internals

| Forbidden | Alternative |
|-----------|-------------|
| Import private core modules | Use public re-exports |
| Access core database tables directly | Use core services/traits |
| Bypass core authentication | Use core auth middleware |

### Cross-Extension Table Access

| Forbidden | Alternative |
|-----------|-------------|
| `SELECT * FROM other_extension_table` | Call other extension's service |
| Foreign keys to other extension's tables | Use IDs and service calls |
| Shared mutable state | Use event system |

### Place Rust Code in Services

| Forbidden | Alternative |
|-----------|-------------|
| `.rs` files in `services/` | Move to `extensions/` |
| MCP servers in `services/mcp/` | Move to `extensions/mcp/` |
| Rust logic in config directory | Separate code from config |

---

## Using Core Services

### Database

```rust
use systemprompt_core_database::DbPool;

pub fn new(pool: Arc<PgPool>) -> Self {
    Self { pool }
}
```

### Logging

```rust
use tracing::{info, error};

info!(user_id = %user.id, "Created user");
error!(error = %e, "Operation failed");
```

### Configuration

```rust
// Via ExtensionContext
fn router(&self, ctx: &ExtensionContext) -> Option<Router> {
    let config = ctx.config().get::<MyConfig>("my_extension")?;
    // ...
}
```

### Identifiers

```rust
use systemprompt_identifiers::{ContentId, UserId, TaskId};

let id = ContentId::new(uuid::Uuid::new_v4().to_string());
```

---

## Cross-Extension Communication

### Pattern 1: Service Import (Preferred)

One extension can import another's public service:

```rust
// In extension A's Cargo.toml
[dependencies]
systemprompt-blog-extension = { path = "../blog" }

// In extension A's code
use systemprompt_blog_extension::ContentService;

pub async fn handler(pool: Arc<PgPool>) {
    let content_service = ContentService::new(pool);
    let content = content_service.get_by_slug("my-post").await?;
}
```

### Pattern 2: Event-Driven (For Decoupling)

Use core's event system for loose coupling:

```rust
// Publisher (in extension A)
event_bus.publish(ContentCreatedEvent { id, title }).await;

// Subscriber (in extension B)
event_bus.subscribe::<ContentCreatedEvent>(|event| {
    // React to content creation
});
```

### Pattern 3: Shared IDs

Extensions can share IDs without table dependencies:

```rust
// Extension A stores content_id
// Extension B stores content_id in its own table
// Both can look up by ID through their own services
```

---

## Boundary Violations to Avoid

### Direct SQL to Core Tables

```rust
// WRONG - accessing core's users table directly
sqlx::query!("SELECT * FROM users WHERE id = $1", user_id)

// RIGHT - use core's user service or trait
let user = user_service.get_by_id(&user_id).await?;
```

### Bypassing Service Layer

```rust
// WRONG - repository access from handler
pub async fn handler(State(state): State<AppState>) -> Response {
    let repo = UserRepository::new(state.pool.clone());
    let user = repo.get_by_id(&id).await?;  // Direct repo access
}

// RIGHT - service access from handler
pub async fn handler(State(state): State<AppState>) -> Response {
    let service = UserService::new(state.pool.clone());
    let user = service.get_by_id(&id).await?;  // Through service
}
```

### Inherent Methods Instead of Traits

```rust
// WRONG - inherent methods only (no polymorphism)
impl MyExtension {
    pub const fn id() -> &'static str { "my_ext" }
    pub fn schemas() -> Vec<(&'static str, &'static str)> { ... }
}

// RIGHT - implement Extension trait
impl Extension for MyExtension {
    fn metadata(&self) -> ExtensionMetadata { ... }
    fn schemas(&self) -> Vec<SchemaDefinition> { ... }
}
```

### Placing MCP Servers in Services

```rust
// WRONG - MCP server in services/mcp/
services/mcp/my-server/src/main.rs

// RIGHT - MCP server in extensions/mcp/
extensions/mcp/my-server/src/main.rs
```

---

## Extension Dependencies in Cargo.toml

### Correct

```toml
[dependencies]
# Core shared types
systemprompt-models = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-identifiers = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-traits = { git = "https://github.com/systempromptio/systemprompt-core" }

# Core infrastructure
systemprompt-core-database = { git = "https://github.com/systempromptio/systemprompt-core" }

# Another extension
systemprompt-blog-extension = { path = "../blog" }
```

### Incorrect

```toml
[dependencies]
# WRONG - importing core's internal crates
systemprompt-core-api = { git = "..." }        # Entry layer - not for extensions
systemprompt-core-scheduler = { git = "..." }  # App layer - use Job trait instead
systemprompt-core-users = { git = "..." }      # Domain - use via public API
```

---

## Summary

| Aspect | Allowed | Forbidden |
|--------|---------|-----------|
| Core code | Read, import public APIs | Modify, fork |
| Core tables | Via core services | Direct SQL |
| Own tables | Full control | FK to other extensions |
| Other extensions | Import public APIs | Access internals |
| Background jobs | Define in Rust, schedule in YAML | Define in YAML only |
| Configuration | Own config in `services/` | Rust code in `services/` |
| MCP servers | In `extensions/mcp/` | In `services/mcp/` |
| Extension struct | Implement `Extension` trait | Inherent methods only |
| Error types | Implement `ExtensionError` trait | Ad-hoc error types |

---

## Quick Reference

| Task | Command |
|------|---------|
| Check boundaries | `grep -E "systemprompt-core-(api\|scheduler)" Cargo.toml` |
| Find SQL in services | `grep -rn "sqlx::" src/services/` |
| Test extension | `cargo test -p systemprompt-{name}-extension` |
| Lint extension | `cargo clippy -p systemprompt-{name}-extension -- -D warnings` |
