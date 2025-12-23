# SystemPrompt Rust Standards

**SystemPrompt is a world-class Rust programming brand.** Every Rust file in this codebase must be instantly recognizable as on-brand, world-class idiomatic Rust as Steve Klabnik would write it. No exceptions. No shortcuts. No compromise.

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
| `if condition { Some(x) } else { None }` | `condition.then(|| x)` |
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
| `panic!()` / `todo!()` / `unimplemented!()` | Return `Result` or implement |
| Inline comments (`//`) | ZERO TOLERANCE - delete all. Code documents itself through naming and structure |
| Doc comments (`///`, `//!`) | ZERO TOLERANCE - no API docs, no rustdoc. Only exception: rare `//!` module docs at file top when absolutely necessary |
| TODO/FIXME/HACK comments | Fix immediately or don't write |
| Tests in source files (`#[cfg(test)]`) | Move to `core/tests/` |

---

## 3. Mandatory Patterns

### Module Manifest (Domain Crates Only)

Every domain crate (`crates/domain/*`) MUST have a validated `module.yaml` at its root. See [architecture.md](./architecture.md) for schema.

| Forbidden | Resolution |
|-----------|------------|
| Domain crate without `module.yaml` | Add manifest with required fields |
| `module.yaml` with missing required fields | Add: name, version, display_name, type |
| Mismatched name/directory | Module name must match directory name |

### Typed Identifiers

All identifier fields use wrappers from `systemprompt_identifiers`:

```rust
// WRONG
pub struct Task { pub id: String, pub user_id: String }

// RIGHT
use systemprompt_identifiers::{TaskId, UserId};
pub struct Task { pub id: TaskId, pub user_id: UserId }
```

Available: `SessionId`, `UserId`, `AgentId`, `TaskId`, `ContextId`, `TraceId`, `ClientId`, `AgentName`, `AiToolCallId`, `McpExecutionId`, `SkillId`, `SourceId`, `CategoryId`.

### Logging

All logging via `tracing` with database persistence via `DatabaseLayer`. No `LogService`, `log::`, or `println!`.

**Request-scoped logging (handlers, services with request context):**

```rust
let _guard = req_ctx.span().enter();
tracing::info!("Processing request");
tracing::error!(error = %e, "Operation failed");
```

**System/background logging (schedulers, startup, no request context):**

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

**Structured fields (preferred over format strings):**

```rust
// WRONG
tracing::info!("Created user {}", user.id);

// RIGHT
tracing::info!(user_id = %user.id, "Created user");
```

| Forbidden | Resolution |
|-----------|------------|
| `LogService::new()` | Use `req_ctx.span().enter()` |
| `LogService::system()` | Use `SystemSpan::new("component").enter()` |
| `logger.info().await.ok()` | Use `tracing::info!()` |
| `tracing::info!` without span | Enter a span first |
| `println!` in library code | Use `tracing::info!()` |

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

### Repository Constructor

```rust
pub struct UserRepository {
    pool: Arc<PgPool>,
}

impl UserRepository {
    pub fn new(db: DbPool) -> Self {
        let pool = db.pool_arc().expect("Database must be PostgreSQL");
        Self { pool }
    }
}
```

### Error Handling

Use domain-specific errors with `thiserror`. `anyhow` only at application boundaries:

```rust
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("User not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
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

Exception: `logger.method().await.ok()` - logging must never fail main operation.

### Builder Pattern (MANDATORY for Complex Types)

**Required** for types with 3+ fields OR any type that mixes required and optional fields.

**Structure:**

```rust
pub struct AiRequest {
    pub messages: Vec<AiMessage>,
    pub provider_config: ProviderConfig,
    pub context: RequestContext,
    pub sampling: Option<SamplingParams>,
    pub tools: Option<Vec<McpTool>>,
}

pub struct AiRequestBuilder {
    messages: Vec<AiMessage>,
    provider_config: ProviderConfig,
    context: RequestContext,
    sampling: Option<SamplingParams>,
    tools: Option<Vec<McpTool>>,
}

impl AiRequestBuilder {
    pub fn new(
        messages: Vec<AiMessage>,
        provider: impl Into<String>,
        model: impl Into<String>,
        max_output_tokens: u32,
        context: RequestContext,
    ) -> Self {
        Self {
            messages,
            provider_config: ProviderConfig::new(provider, model, max_output_tokens),
            context,
            sampling: None,
            tools: None,
        }
    }

    pub fn with_sampling(mut self, sampling: SamplingParams) -> Self {
        self.sampling = Some(sampling);
        self
    }

    pub fn with_tools(mut self, tools: Vec<McpTool>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn build(self) -> AiRequest {
        AiRequest {
            messages: self.messages,
            provider_config: self.provider_config,
            context: self.context,
            sampling: self.sampling,
            tools: self.tools,
        }
    }
}

impl AiRequest {
    pub fn builder(
        messages: Vec<AiMessage>,
        provider: impl Into<String>,
        model: impl Into<String>,
        max_output_tokens: u32,
        context: RequestContext,
    ) -> AiRequestBuilder {
        AiRequestBuilder::new(messages, provider, model, max_output_tokens, context)
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
| Static `builder()` on target type | Entry point: `AiRequest::builder(...)` |

**FORBIDDEN:**

```rust
// WRONG - optional fields mixed in constructor
AiRequest::new(messages, Some(provider), None, Some(8192), None, None)

// WRONG - mutable builder
let mut builder = AiRequest::builder(...);
builder.sampling = Some(params);  // Direct field access forbidden

// WRONG - Default trait for required fields
impl Default for AiRequest { ... }  // Forces invalid defaults
```

**CORRECT:**

```rust
let request = AiRequest::builder(
    messages,
    "gemini",
    "gemini-2.5-flash",
    8192,
    context,
)
.with_sampling(SamplingParams { temperature: Some(0.7), ..Default::default() })
.with_tools(tools)
.build();
```

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
| LogService | `logger` |
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
| Orphan tracing calls | Enter a span first (see Section 3) |
| Fuzzy strings / hardcoded fallbacks | Use typed constants, enums, or fail explicitly |
| `unwrap_or_default()` | Fail explicitly - never use fuzzy defaults |
| Direct `env::var()` | Use `Config::init()` / `AppContext` - profiles REQUIRED |
| Hardcoded fallback values | Return `Result::Err`, never silently default |
| Unused code / dead code | Delete immediately |
| Tech debt / TODO comments | Fix now or don't write it |
| Commented-out code | Delete - git has history |

---

## 6. Multi-Process Broadcasting

Events from agent/worker processes must go through HTTP webhook to API process:

```
Agent Process → HTTP POST /webhook → API Process → CONTEXT_BROADCASTER → SSE clients
```

Direct `CONTEXT_BROADCASTER` access only works in the API process (where SSE connections live).

Use `BroadcastClient` trait:
- `create_webhook_broadcaster(token)` - for agent services
- `create_local_broadcaster()` - for API routes (same process)
