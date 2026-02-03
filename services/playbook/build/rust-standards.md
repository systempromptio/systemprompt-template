---
title: "Rust Standards Playbook"
description: "SystemPrompt Rust programming standards and idiomatic patterns."
author: "SystemPrompt"
slug: "build-rust-standards"
keywords: "rust, standards, idiomatic, patterns"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# SystemPrompt Rust Standards

**SystemPrompt is a world-class Rust programming brand.** Every Rust file in extensions and MCP servers must be instantly recognizable as on-brand, world-class idiomatic Rust as Steve Klabnik would write it. No exceptions. No shortcuts. No compromise.

Checkable, actionable patterns. Run `cargo clippy --workspace -- -D warnings` and `cargo fmt --all` after changes.

---

## 0. Idiomatic Rust

Write code that would pass Steve Klabnik's review. Prefer iterator chains, combinators, and pattern matching over imperative control flow.

### Option/Result Combinators

```rust
let name = request.name.as_deref().map(str::trim);

let value = opt.unwrap_or_else(|| compute_default());

let result = input.ok_or_else(|| Error::Missing)?;
```

### Pattern Matching

```rust
match request.name.as_deref().map(str::trim) {
    Some("") => return Err(ApiError::bad_request("Name cannot be empty")),
    Some(name) => name.to_owned(),
    None => generate_default(),
}
```

### Iterator Chains

```rust
let valid_items: Vec<_> = items
    .iter()
    .filter(|item| item.is_active())
    .map(|item| item.to_dto())
    .collect();
```

### Avoid

| Anti-Pattern | Idiomatic |
|--------------|-----------|
| `if let Some(x) = opt { x } else { default }` | `opt.unwrap_or(default)` |
| `match opt { Some(x) => Some(f(x)), None => None }` | `opt.map(f)` |
| `if condition { Some(x) } else { None }` | `condition.then(\|\| x)` |
| Nested `if let` / `match` | Combine with `and_then`, `map`, `ok_or` |
| Manual loops building `Vec` | Iterator chains with `collect()` |
| `match` with guards when combinators suffice | `filter`, `map`, `and_then` |

---

## 1. Limits

| Metric | Limit |
|--------|-------|
| Source file length | 300 lines |
| Cognitive complexity | 15 |
| Function length | 75 lines |
| Parameters | 5 |

---

## 2. Forbidden Constructs

| Construct | Resolution |
|-----------|------------|
| `unsafe` | Remove - forbidden in this codebase |
| `unwrap()` | Use `?`, `ok_or_else()`, or `expect()` with descriptive message |
| `unwrap_or_default()` | Fail explicitly - never use fuzzy defaults |
| `panic!()` / `todo!()` / `unimplemented!()` | Return `Result` or implement |
| Inline comments (`//`) | ZERO TOLERANCE - delete all. Code documents itself through naming and structure |
| Doc comments (`///`, `//!`) | ZERO TOLERANCE - no API docs, no rustdoc, no module docs. All doc comments forbidden |
| TODO/FIXME/HACK comments | Fix immediately or don't write |
| Raw `env::var()` | Use `Config::init()` / `AppContext` |
| Magic numbers/strings | Use constants or enums |
| Commented-out code | Delete - git has history |

---

## 3. Mandatory Patterns

### Typed Identifiers

All identifier fields use wrappers from `systemprompt_identifiers`:

```rust
// WRONG
pub struct Content { pub id: String, pub user_id: String }

// RIGHT
use systemprompt_identifiers::{ContentId, UserId};
pub struct Content { pub id: ContentId, pub user_id: UserId }
```

Available: `SessionId`, `UserId`, `AgentId`, `TaskId`, `ContextId`, `TraceId`, `ClientId`, `AgentName`, `AiToolCallId`, `McpExecutionId`, `SkillId`, `SourceId`, `CategoryId`, `ContentId`.

### Logging

All logging via `tracing`. No `println!` in library code.

**Request-scoped (handlers, services):**
```rust
let _guard = req_ctx.span().enter();
tracing::info!(user_id = %user.id, "Created user");
```

**System/background (schedulers, startup):**
```rust
let _guard = SystemSpan::new("scheduler").enter();
tracing::info!("Running cleanup job");
```

**Adding context mid-request:**
```rust
let span = req_ctx.span();
span.record_task_id(&task_id);
let _guard = span.enter();
```

Use structured fields: `tracing::info!(user_id = %id, "msg")` not `tracing::info!("msg {}", id)`.

| Forbidden | Resolution |
|-----------|------------|
| `println!` in library code | Use `tracing::info!()` |
| Format strings with interpolation | Use structured fields |

### Repository Pattern

Services NEVER execute queries directly. All SQL in repositories using SQLX macros:

```rust
// Repository - uses sqlx::query_as!
pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(User, "SELECT id, email, name FROM users WHERE email = $1", email)
        .fetch_optional(&**self.pool)
        .await
}

// Service - calls repository
let user = self.user_repository.find_by_email(email).await?;
```

### SQLX Macros Only

| Allowed | Forbidden |
|---------|-----------|
| `sqlx::query!()` | `sqlx::query()` |
| `sqlx::query_as!()` | `sqlx::query_as()` |
| `sqlx::query_scalar!()` | `sqlx::query_scalar()` |

The `!` suffix enables compile-time verification. Zero tolerance for runtime query strings.

### Repository Constructors

**Reference Pattern (repositories):**
```rust
impl UserRepository {
    pub fn new(db: &DbPool) -> Result<Self> {
        Ok(Self { pool: db.pool_arc()? })
    }
}
```

**Owned Pattern (services/composites):**
```rust
impl TaskService {
    pub const fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }
}
```

| Pattern | Parameter Name |
|---------|---------------|
| Reference | `db: &DbPool` |
| Owned | `db_pool: DbPool` |

### Error Handling

Use domain-specific errors with `thiserror`. `anyhow` only at application boundaries:

```rust
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Item not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

Log errors once at handling boundary, not at every propagation point.

### DateTime

| Layer | Type |
|-------|------|
| Rust | `DateTime<Utc>` |
| PostgreSQL | `TIMESTAMPTZ` |

Never use `NaiveDateTime` or `TIMESTAMP`. Never format as strings for DB operations.

### Option<T>

Only valid when absence is a meaningful domain state. Invalid uses:
- "I don't have it yet"
- Avoiding validation
- Default values that should be explicit

### Fail Fast

Never return `Ok` for failed paths. Propagate errors immediately with `?`.

### Builder Pattern (MANDATORY for Complex Types)

**Required** for types with 3+ fields OR any type that mixes required and optional fields.

**Structure:**

```rust
pub struct CreateContentParams {
    pub slug: String,
    pub title: String,
    pub body: String,
    pub description: Option<String>,
    pub image: Option<String>,
}

pub struct CreateContentParamsBuilder {
    slug: String,
    title: String,
    body: String,
    description: Option<String>,
    image: Option<String>,
}

impl CreateContentParamsBuilder {
    pub fn new(slug: impl Into<String>, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            slug: slug.into(),
            title: title.into(),
            body: body.into(),
            description: None,
            image: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = Some(image.into());
        self
    }

    pub fn build(self) -> CreateContentParams {
        CreateContentParams {
            slug: self.slug,
            title: self.title,
            body: self.body,
            description: self.description,
            image: self.image,
        }
    }
}

impl CreateContentParams {
    pub fn builder(slug: impl Into<String>, title: impl Into<String>, body: impl Into<String>) -> CreateContentParamsBuilder {
        CreateContentParamsBuilder::new(slug, title, body)
    }
}
```

**Rules:**

| Rule | Description |
|------|-------------|
| Required fields in `new()` | All non-optional fields MUST be constructor parameters |
| Optional fields via `with_*()` | Each optional field gets a `with_*` method |
| `build()` returns owned type | Builder is consumed, returns final struct |
| No `Default` for complex types | Explicit construction prevents invalid states |
| Static `builder()` on target type | Entry point: `CreateContentParams::builder(...)` |

---

## 4. Naming

### Functions

| Prefix | Returns |
|--------|---------|
| `get_` | `Result<T>` - fails if missing |
| `find_` | `Result<Option<T>>` - may not exist |
| `list_` | `Result<Vec<T>>` |
| `create_` | `Result<T>` or `Result<Id>` |
| `update_` | `Result<T>` or `Result<()>` |
| `delete_` | `Result<()>` |
| `is_` / `has_` | `bool` |

### Variables

| Type | Name |
|------|------|
| Database pool | `db_pool` |
| Repository | `{noun}_repository` |
| Service | `{noun}_service` |

### Abbreviations

Allowed: `id`, `uuid`, `url`, `jwt`, `mcp`, `a2a`, `api`, `http`, `json`, `sql`, `ctx`, `req`, `res`, `msg`, `err`, `cfg`

---

## 5. Anti-Patterns

| Pattern | Resolution |
|---------|------------|
| Raw string identifiers | Use typed identifiers |
| Magic numbers/strings | Use constants or enums |
| Direct SQL in services | Move to repository |
| `Option<Id>` for required fields | Use non-optional |
| Fuzzy strings / hardcoded fallbacks | Use typed constants, enums, or fail explicitly |
| Unused code / dead code | Delete immediately |
| Tech debt / TODO comments | Fix now or don't write it |
| Commented-out code | Delete - git has history |

---

## 6. Silent Error Anti-Patterns

These patterns silently swallow errors, making debugging impossible:

| Pattern | Resolution |
|---------|------------|
| `.ok()` on Result | Use `?` or `map_err()` to propagate with context |
| `let _ = result` | Handle error explicitly or use `?` |
| `match { Err(_) => default }` | Propagate error or log with `tracing::error!` |
| `filter_map(\|e\| e.ok())` | Log failures before filtering |
| Error log then `Ok()` | Propagate the error after logging |

**Acceptable `.ok()` usage:**

1. **Cleanup in error paths** - when already returning an error:
```rust
if let Err(e) = operation().await {
    cleanup().await.ok();
    return Err(e);
}
```

2. **Parse with logged warning:**
```rust
serde_json::from_str(s).map_err(|e| {
    tracing::warn!(error = %e, "Parse failed");
    e
}).ok()
```

**Detection commands:**
```bash
rg '\.ok\(\)' --type rust
rg 'let _ =' --type rust
rg 'unwrap_or_default\(\)' --type rust
```

---

## 7. Derive Ordering

When deriving traits, use this order:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MyType { ... }
```

Order: `Debug`, `Clone`, `Copy` (if applicable), `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Default`, `Serialize`, `Deserialize`

---

## 8. Extension-Specific Patterns

### Schema Embedding

```rust
pub const SCHEMA_MY_TABLE: &str = include_str!("../schema/001_my_table.sql");

impl MyExtension {
    pub fn schemas() -> Vec<(&'static str, &'static str)> {
        vec![("my_table", SCHEMA_MY_TABLE)]
    }
}
```

### Job Registration

```rust
use systemprompt_traits::{Job, JobContext, JobResult, submit_job};

#[derive(Debug, Clone, Copy, Default)]
pub struct MyJob;

#[async_trait::async_trait]
impl Job for MyJob {
    fn name(&self) -> &'static str { "my_job" }
    fn description(&self) -> &'static str { "Does something" }
    fn schedule(&self) -> &'static str { "0 0 * * * *" }

    async fn execute(&self, ctx: &JobContext) -> anyhow::Result<JobResult> {
        let pool = ctx.db_pool::<PgPool>()?;
        Ok(JobResult::success())
    }
}

submit_job!(&MyJob);
```

### Router Factory

```rust
impl MyExtension {
    pub fn router(&self, pool: Arc<PgPool>) -> Router {
        let state = MyState { pool };
        Router::new()
            .route("/items", get(list_items).post(create_item))
            .route("/items/:id", get(get_item).delete(delete_item))
            .with_state(state)
    }
}
```

---

## 9. Multi-Process Broadcasting

Events from agent/worker processes must go through HTTP webhook to API process:

```
Agent Process → HTTP POST /webhook → API Process → CONTEXT_BROADCASTER → SSE clients
```

Use `BroadcastClient` trait:
- `create_webhook_broadcaster(token)` - for agent services
- `create_local_broadcaster()` - for API routes (same process)

---

## Quick Reference

| Task | Command |
|------|---------|
| Lint all | `cargo clippy --workspace -- -D warnings` |
| Format all | `cargo fmt --all` |
| Check format | `cargo fmt --all -- --check` |
| Build all | `cargo build --workspace` |