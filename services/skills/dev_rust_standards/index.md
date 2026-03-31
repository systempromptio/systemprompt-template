---
name: "Rust Standards"
description: "Complete Rust coding, linting, testing, architecture, and layer boundary standards for demo.systemprompt.io development"
---

# demo.systemprompt.io Rust Standards

**demo.systemprompt.io is a world-class Rust programming brand.** Every Rust file must be instantly recognizable as idiomatic Rust as Steve Klabnik would write it.

Run `cargo clippy --workspace -- -D warnings` and `cargo fmt --all` after changes.

---

## 1. Idiomatic Rust

Prefer iterator chains, combinators, and pattern matching over imperative control flow.

```rust
let name = request.name.as_deref().map(str::trim);
let value = opt.unwrap_or_else(|| compute_default());
let result = input.ok_or_else(|| Error::Missing)?;

let valid_items: Vec<_> = items
    .iter()
    .filter(|item| item.is_active())
    .map(|item| item.to_dto())
    .collect();
```

| Anti-Pattern | Idiomatic |
|--------------|-----------|
| `if let Some(x) = opt { x } else { default }` | `opt.unwrap_or(default)` |
| `match opt { Some(x) => Some(f(x)), None => None }` | `opt.map(f)` |
| `if condition { Some(x) } else { None }` | `condition.then(\|\| x)` |
| Nested `if let` / `match` | Combine with `and_then`, `map`, `ok_or` |
| Manual loops building `Vec` | Iterator chains with `collect()` |

---

## 2. Limits

| Metric | Limit |
|--------|-------|
| Source file length | 300 lines |
| Cognitive complexity | 15 |
| Function length | 75 lines |
| Parameters | 5 |

---

## 3. Forbidden Constructs

| Construct | Resolution |
|-----------|------------|
| `unsafe` | Remove - forbidden in this codebase |
| `unwrap()` | Use `?`, `ok_or_else()`, or `expect()` with message |
| `unwrap_or_default()` | Fail explicitly - never use fuzzy defaults |
| `panic!()` / `todo!()` / `unimplemented!()` | Return `Result` or implement |
| Inline comments (`//`) | Delete - code documents itself through naming |
| Doc comments (`///`, `//!`) | Delete - no rustdoc (rare `//!` module docs excepted) |
| TODO/FIXME/HACK comments | Fix immediately or don't write |
| Tests in source files (`#[cfg(test)]`) | Move to `crates/tests/` |
| Raw `env::var()` | Use `Config::init()` / `AppContext` |
| Magic numbers/strings | Use constants or enums |
| Commented-out code | Delete - git has history |

---

## 4. Mandatory Patterns

### Typed Identifiers

All identifier fields use wrappers from `systemprompt_identifiers`:

```rust
use systemprompt_identifiers::{TaskId, UserId};
pub struct Task { pub id: TaskId, pub user_id: UserId }
```

Available: `SessionId`, `UserId`, `AgentId`, `TaskId`, `ContextId`, `TraceId`, `ClientId`, `AgentName`, `AiToolCallId`, `McpExecutionId`, `SkillId`, `SourceId`, `CategoryId`, `ArtifactId`.

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

### Repository Pattern

Services NEVER execute queries directly. All SQL in repositories using SQLX macros:

```rust
pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(User, "SELECT id, email, name FROM users WHERE email = $1", email)
        .fetch_optional(&**self.pool)
        .await
}
```

| Allowed | Forbidden |
|---------|-----------|
| `sqlx::query!()` | `sqlx::query()` |
| `sqlx::query_as!()` | `sqlx::query_as()` |
| `sqlx::query_scalar!()` | `sqlx::query_scalar()` |

The `!` suffix enables compile-time verification.

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

Never use `NaiveDateTime` or `TIMESTAMP`.

### Builder Pattern

**Required** for types with 3+ fields OR mixed required/optional fields.

```rust
impl AiRequest {
    pub fn builder(messages: Vec<AiMessage>, provider: &str, model: &str,
                   max_tokens: u32, ctx: RequestContext) -> AiRequestBuilder {
        AiRequestBuilder::new(messages, provider, model, max_tokens, ctx)
    }
}

let request = AiRequest::builder(messages, "gemini", "gemini-2.5-flash", 8192, ctx)
    .with_sampling(params)
    .with_tools(tools)
    .build();
```

| Rule | Description |
|------|-------------|
| Required fields in `new()` | All non-optional fields as constructor parameters |
| Optional fields via `with_*()` | Each optional field gets a builder method |
| `build()` consumes builder | Returns final struct |
| No `Default` for complex types | Explicit construction prevents invalid states |

---

## 5. Naming

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
rg '\.ok\(\)' --type rust -g '!*test*'
rg 'let _ =' --type rust -g '!*test*'
rg 'unwrap_or_default\(\)' --type rust -g '!*test*'
```

---

## 7. Multi-Process Broadcasting

Events from agent/worker processes must go through HTTP webhook to API process:

```
Agent Process -> HTTP POST /webhook -> API Process -> CONTEXT_BROADCASTER -> SSE clients
```

Use `BroadcastClient` trait:
- `create_webhook_broadcaster(token)` - for agent services
- `create_local_broadcaster()` - for API routes (same process)

---

# Clippy Linting Standards

**Reference:** This document extends the Rust Standards above. All code must meet world-class idiomatic Rust standards.

---

## Enforcement

```bash
cargo clippy --workspace -- -D warnings
cargo fmt --all
```

Zero tolerance for warnings. All clippy lints are enforced at the workspace level.

---

## Workspace Lint Configuration

All crates inherit from `[workspace.lints.clippy]` in root `Cargo.toml`. No per-crate overrides except test crates.

### Lint Groups

| Group | Level | Purpose |
|-------|-------|---------|
| `all` | deny | Core correctness lints |
| `pedantic` | deny | Stricter code quality |
| `nursery` | warn | Experimental quality lints |
| `cargo` | warn | Cargo manifest quality |
| `perf` | warn | Performance suggestions |
| `suspicious` | deny | Potentially incorrect code |

### Denied Lints (Errors)

These lints cause compilation to fail:

| Lint | Rationale |
|------|-----------|
| `unwrap_used` | Use `?`, `ok_or_else()`, or `expect()` with context |
| `panic` | Return `Result`, never panic in library code |
| `unimplemented` | Implement or return `Result::Err` |
| `todo` | Complete implementation before merge |
| `too_many_arguments` | Refactor into builder pattern or config struct |
| `dbg_macro` | Remove debug macros before merge |
| `exit` | Only allowed in CLI entry points with justification |
| `rc_mutex` | Use `Arc<Mutex<T>>` or redesign |

### Warned Lints (Must Address)

| Lint | Action Required |
|------|-----------------|
| `cognitive_complexity` | Refactor function, extract helpers |
| `too_many_lines` | Split into smaller functions (limit: 75 lines) |
| `type_complexity` | Create type alias |
| `expect_used` | Prefer `?` operator, use `expect` only with clear message |
| `inefficient_to_string` | Use `to_owned()` or avoid allocation |
| `unnecessary_wraps` | Remove unnecessary `Result`/`Option` wrapper |
| `unused_async` | Remove `async` or add async operation |
| `if_not_else` | Invert condition for clarity |
| `redundant_else` | Remove unnecessary `else` after early return |
| `manual_let_else` | Use `let ... else { }` pattern |
| `match_bool` | Use `if`/`else` instead |
| `option_if_let_else` | Use combinators: `map_or`, `map_or_else` |
| `needless_pass_by_value` | Take `&T` instead of `T` |
| `items_after_statements` | Move items before statements |
| `semicolon_if_nothing_returned` | Add semicolon for clarity |
| `or_fun_call` | Use `unwrap_or_else` instead of `unwrap_or` with fn call |
| `redundant_clone` | Remove unnecessary `.clone()` |
| `unnecessary_to_owned` | Avoid `.to_owned()` when reference suffices |
| `implicit_clone` | Make cloning explicit |
| `large_futures` | Box large futures |
| `match_wild_err_arm` | Handle specific error variants |
| `print_stdout` | Use `tracing` for output |
| `print_stderr` | Use `tracing` for errors |
| `empty_structs_with_brackets` | Remove `{}` from unit structs |
| `rest_pat_in_fully_bound_structs` | Bind all fields explicitly |
| `clone_on_ref_ptr` | Use `Arc::clone(&x)` for clarity |
| `separated_literal_suffix` | Use `100_u32` not `100u32` |
| `try_err` | Use `?` operator instead of `return Err` |

### Allowed Lints (Workspace Exceptions)

These are allowed at workspace level due to practical constraints:

| Lint | Reason |
|------|--------|
| `cargo_common_metadata` | Not publishing to crates.io |
| `multiple_crate_versions` | Transitive dependency conflicts |
| `return_self_not_must_use` | Builder pattern returns |
| `must_use_candidate` | Too many false positives |
| `trivially_copy_pass_by_ref` | Consistency over micro-optimization |
| `cast_possible_truncation` | TUI coordinate math requires casts |
| `cast_sign_loss` | TUI coordinate math requires casts |
| `cast_precision_loss` | Progress bar calculations |
| `cast_possible_wrap` | TUI coordinate math |
| `format_push_string` | String building patterns |
| `uninlined_format_args` | Readability preference |
| `result_unit_err` | Legacy error patterns |
| `missing_docs_in_private_items` | No rustdoc requirement |
| `module_name_repetitions` | Domain naming clarity |
| `missing_errors_doc` | No rustdoc requirement |
| `missing_panics_doc` | No rustdoc requirement |
| `derive_partial_eq_without_eq` | Generated code compatibility |

---

## Inline Allow Policy

**Default: FORBIDDEN**

Inline `#[allow(clippy::...)]` attributes are prohibited except for documented exceptions.

### Process for New Exceptions

1. Verify the lint cannot be fixed through refactoring
2. Document the technical constraint requiring the exception
3. Add to exceptions documentation with file path and justification
4. Include explanatory comment on the allow attribute

### Exception Categories

Only these categories may qualify for exceptions:

| Category | Example |
|----------|---------|
| **Serde/Serialization** | Empty structs for JSON `{}` |
| **FFI/External Constraints** | Generic closures in async contexts |
| **Fundamental Type Design** | Capability flags requiring many bools |
| **CLI Entry Points** | Intentional `exit()` for fatal errors |

---

## Test Crates

Test crates (`crates/tests/**`) may have relaxed lints:

```toml
[lints.clippy]
expect_used = "allow"
unwrap_used = "allow"
panic = "allow"
```

This does NOT apply to:
- Unit tests in library crates
- Integration tests in `src/` directories
- Any production code

---

## Rust Lints

| Lint | Level |
|------|-------|
| `unsafe_code` | forbid |
| `missing_debug_implementations` | warn |
| `missing_copy_implementations` | warn |
| `trivial_casts` | warn |
| `trivial_numeric_casts` | warn |
| `unused_import_braces` | warn |
| `unused_qualifications` | warn |
| `variant_size_differences` | warn |

---

## Pre-Commit Checklist

Before every commit:

1. `cargo clippy --workspace -- -D warnings` passes
2. `cargo fmt --all` applied
3. No new inline `#[allow(...)]` without exception documentation
4. All `cognitive_complexity` warnings addressed or justified
5. No `todo!()`, `unimplemented!()`, or `panic!()` in library code

---

# Test Coverage Analysis and Implementation

> Analyze test coverage in this crate, identify gaps, fix broken tests, and implement missing test coverage.

---

## Required Reading

Before beginning test work, understand:

1. **Architecture** -- see Architecture Overview section below
2. **Testing Guidelines** -- this section
3. **Rust Standards** -- see Rust Standards section above

---

## Critical Rules

### Test Location Policy

**ALL tests MUST be in separate crates** -- NEVER inline, ALWAYS in the separate test workspace.

| Pattern | Status |
|---------|--------|
| `#[cfg(test)] mod tests { }` in source | FORBIDDEN |
| `crates/domain/users/tests/` | FORBIDDEN |
| `crates/tests/unit/domain/users/` | REQUIRED |

### Test Workspace Structure

```
crates/tests/
+-- Cargo.toml              # Separate workspace manifest
+-- unit/                   # Unit tests mirroring source structure
|   +-- shared/
|   |   +-- models/
|   |   +-- traits/
|   |   +-- identifiers/
|   |   +-- client/
|   +-- infra/
|   |   +-- database/
|   |   +-- logging/
|   |   +-- config/
|   |   +-- cloud/
|   |   +-- loader/
|   |   +-- events/
|   +-- domain/
|   |   +-- users/
|   |   +-- oauth/
|   |   +-- files/
|   |   +-- analytics/
|   |   +-- content/
|   |   +-- ai/
|   |   +-- mcp/
|   |   +-- agent/
|   +-- app/
|   |   +-- runtime/
|   |   +-- generator/
|   +-- entry/
|       +-- api/
+-- integration/            # Integration tests by feature
    +-- extension/
```

---

## Analysis Steps

### Phase 1: Inventory Source Crates

For each layer, list all source crates and their key modules:

```bash
ls -la crates/shared/
ls -la crates/infra/
ls -la crates/domain/
ls -la crates/app/
ls -la crates/entry/
```

### Phase 2: Inventory Test Crates

Map existing test coverage:

```bash
ls -la crates/tests/unit/
ls -la crates/tests/integration/
```

### Phase 3: Coverage Gap Analysis

For each source crate, determine:

1. **Does a corresponding test crate exist?**
2. **What modules lack test coverage?**
3. **What is the current test health?**
   ```bash
   cargo test --manifest-path crates/tests/Cargo.toml --workspace 2>&1
   ```

### Phase 4: Coverage Metrics

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --manifest-path crates/tests/Cargo.toml --workspace --html
cargo llvm-cov --manifest-path crates/tests/Cargo.toml --workspace
```

---

## Test Implementation Guidelines

### Unit Test Template

```rust
use systemprompt_{crate}::{Module, function_to_test};

#[test]
fn function_name_with_valid_input_returns_expected() {
    let input = create_valid_input();
    let result = function_to_test(input);
    assert_eq!(result, expected_output);
}

#[test]
fn function_name_with_invalid_input_returns_error() {
    let input = create_invalid_input();
    let result = function_to_test(input);
    assert!(result.is_err());
}
```

### Async Test Template

```rust
use systemprompt_{crate}::{AsyncService};

#[tokio::test]
async fn async_operation_completes_successfully() {
    let service = AsyncService::new();
    let result = service.operation().await;
    assert!(result.is_ok());
}
```

### Integration Test Template (7-Phase)

```rust
#[tokio::test]
async fn integration_test_name() -> Result<()> {
    // Phase 1: Setup
    let ctx = TestContext::new().await?;
    let unique_id = ctx.fingerprint().to_string();

    // Phase 2: Action
    let response = ctx.make_request("/endpoint").await?;
    assert!(response.status().is_success());

    // Phase 3: Wait
    wait_for_async_processing().await;

    // Phase 4: Query
    let rows = ctx.db.fetch_all(&query, &[&unique_id]).await?;

    // Phase 5: Assert
    assert!(!rows.is_empty());

    // Phase 6: Cleanup
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(unique_id);
    cleanup.cleanup_all().await?;

    // Phase 7: Log
    println!("Test passed");
    Ok(())
}
```

---

## Creating Missing Test Crates

For each missing test crate:

1. Create `Cargo.toml`:
   ```toml
   [package]
   name = "systemprompt-{name}-tests"
   version.workspace = true
   edition.workspace = true
   publish = false

   [dependencies]
   systemprompt-{name} = { path = "../../../../{layer}/{name}" }

   [dev-dependencies]
   tokio = { workspace = true, features = ["test-util", "macros"] }
   ```

2. Create `src/lib.rs` with module declarations
3. Add to workspace members in `crates/tests/Cargo.toml`

---

## Status File Format

After completing test work, update `{crate_path}/status.md`:

```markdown
# {Crate Name} Status

## Test Status

| Metric | Value |
|--------|-------|
| Test crate | `crates/tests/unit/{layer}/{name}/` |
| Tests passing | X/Y |
| Coverage | Z% |
| Last verified | YYYY-MM-DD |

## Test Commands

cargo test --manifest-path crates/tests/Cargo.toml -p systemprompt-{name}-tests
cargo llvm-cov --manifest-path crates/tests/Cargo.toml -p systemprompt-{name}-tests
```

---

## Test Checklist

Before completing:

- [ ] All tests compile (`cargo build --manifest-path crates/tests/Cargo.toml --workspace`)
- [ ] All tests pass (`cargo test --manifest-path crates/tests/Cargo.toml --workspace`)
- [ ] No inline tests in source crates (`grep -r "#[cfg(test)]" crates/shared crates/infra crates/domain crates/app crates/entry`)
- [ ] Test crate naming follows convention (`systemprompt-{name}-tests`)
- [ ] Test files mirror source structure
- [ ] High-priority gaps addressed
- [ ] Crate `status.md` updated with current test state

---

# Architecture Overview

## Crate Layers

```
crates/
  shared/     # Pure types, zero internal dependencies
  infra/      # Stateless infrastructure utilities
  domain/     # Bounded contexts with SQL + repos + services
  app/        # Orchestration, no business logic
  entry/      # Entry points (binaries, public APIs)

systemprompt/   # Facade: Public API for external consumers (crates.io)
```

### Dependency Direction

```
Entry (api, cli) -> App (runtime, scheduler, generator)
                 |
           Domain (agent, ai, mcp, oauth, users, files, content, analytics, templates)
                 |
           Infra (database, events, security, config, logging, loader, cloud)
                 |
           Shared (models, traits, identifiers, extension, provider-contracts, client)
```

### Shared Layer (`crates/shared/`)

Pure types with zero dependencies on other systemprompt crates (except within shared/).

| Crate | Purpose |
|-------|---------|
| `provider-contracts/` | Provider trait contracts (`LlmProvider`, `ToolProvider`, `Job`, `ComponentRenderer`, etc.) |
| `identifiers/` | Typed IDs (`UserId`, `TaskId`, etc.) |
| `models/` | Domain models, API types, configuration structs |
| `traits/` | Infrastructure trait definitions (`DomainConfig`, `ConfigProvider`, `DatabaseHandle`) |
| `template-provider/` | Template loading and rendering abstractions |
| `client/` | HTTP client for external API access |
| `extension/` | Extension framework for user customization |

### Infrastructure Layer (`crates/infra/`)

Stateless utilities providing cross-cutting concerns. Can depend on `shared/` only.

| Crate | Purpose |
|-------|---------|
| `database/` | SQLx abstraction, connection pooling, base repository trait |
| `events/` | Event bus, broadcasters, SSE infrastructure |
| `security/` | JWT validation, token extraction, cookie handling |
| `config/` | Configuration loading, environment handling |
| `logging/` | Tracing setup, log sinks, database layer |
| `cloud/` | Cloud API client, tenant management |

### Domain Layer (`crates/domain/`)

Full bounded contexts. Each crate owns its database tables, repositories, and services. Can depend on `shared/` and `infra/`.

| Crate | Bounded Context | Key Entities |
|-------|-----------------|--------------|
| `users/` | User identity | User, Role |
| `oauth/` | Authentication | Token, Client, Grant, Session |
| `files/` | File storage | File, FileMetadata |
| `analytics/` | Metrics & tracking | Session, Event, Metric |
| `content/` | Content management | Content, Category, Tag |
| `ai/` | LLM integration | Request, Response, Provider |
| `mcp/` | MCP protocol | Server, Tool, Deployment |
| `agent/` | A2A protocol | Agent, Task, Context, Skill |

**Required domain crate structure:**
```
domain/{name}/
  Cargo.toml
  schema/             # SQL schema files
  src/
    lib.rs            # Public API
    extension.rs      # Extension trait implementation
    error.rs          # Domain-specific errors
    models/           # Domain models
    repository/       # Data access layer
    services/         # Business logic
```

### Application Layer (`crates/app/`)

Orchestration without business logic. Can depend on `shared/`, `infra/`, `domain/`.

| Crate | Purpose |
|-------|---------|
| `scheduler/` | Job scheduling, cron execution |
| `generator/` | Static site generation |
| `runtime/` | StartupValidator, AppContext, lifecycle management |

### Entry Layer (`crates/entry/`)

Entry points that wire the application together. Can depend on all layers.

| Crate | Purpose |
|-------|---------|
| `cli/` | Command-line interface |
| `api/` | HTTP gateway, route handlers, middleware |
| `tui/` | Terminal UI |

### Facade (`systemprompt/`)

Public API for external consumers. Re-exports only, no new code.

---

# Module Boundary Guidelines

## Guiding Principles

### 1. Repositories Are Public API

Using a repository from another module is the **correct pattern** for cross-module data access:
- Repositories are intentionally exposed via `pub mod repository`
- This is idiomatic Rust -- no need for extra abstraction layers
- Dependencies are clear: caller depends on callee's repository

### 2. Downward Dependencies Are Fine

Dependencies are acceptable when:
- They flow downward (higher-level -> lower-level)
- There are no circular dependencies
- The boundary is clear (using public API)

Example: `agent -> mcp` is correct because agent orchestrates MCP tools.

### 3. Avoid Over-Abstraction

Do NOT add traits just for the sake of abstraction:
- If only one implementation exists, use the concrete type
- Traits add complexity without benefit for single implementations
- This is not Java -- avoid dependency injection patterns

### 4. Config Profiles Are Mandatory

All code must use config profiles -- no environment variable fallbacks:
- `Config::from_profile()` is the only way to build configuration
- Missing paths cause **startup errors**, not runtime fallbacks
- Each domain validates its config via `DomainConfig` trait
- Extensions validate via `ConfigExtensionTyped` trait
- Validation is **always blocking** -- no `--force` bypass

### 5. Subprocess Config/Secrets Propagation

When spawning subprocesses (agents, MCP servers), config and secrets MUST be passed explicitly:

**Required env vars for ALL subprocesses:**
- `SYSTEMPROMPT_PROFILE` -- Path to profile.yaml
- `JWT_SECRET` -- JWT signing secret (passed directly, no file discovery)
- `DATABASE_URL` -- Database connection string

**Rules:**
- Parent MUST pass secrets explicitly -- no fuzzy profile discovery in subprocesses
- Subprocesses MUST prioritize `JWT_SECRET` env var over file loading
- All processes in the system MUST use identical JWT secrets for token validation
- Never use `if let Ok(...)` patterns for secrets -- fail loudly if missing

### 6. Module System

Modules are defined in Rust code at `crates/infra/loader/src/modules/`. Each module uses `include_str!()` to embed SQL schemas at compile time.

### 7. Extension Linkage via Product Binary

Extensions register jobs, schemas, and routes via `inventory` macros. These are static initializers that only execute if the crate is linked into the final binary.

**Key rule:** Core's CLI binary does NOT link extension crates. Products must own the binary.

```rust
use my_product as _;  // Forces linkage

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    systemprompt_cli::run().await
}
```

---

## Layer Dependencies

### Forbidden Dependencies

| Layer | Cannot Depend On |
|-------|------------------|
| Shared | Any systemprompt crate (except within shared/) |
| Infra | domain/, app/, entry/ |
| Domain | Other domain crates, app/, entry/ |
| App | entry/ |

### Acceptable Cross-Domain Dependencies

Domain crates using another domain's public API is acceptable when:
- Dependency is downward (orchestration layer using lower-level service)
- Uses public repository/service API, not internal types
- No circular dependencies exist

---

## Current Architecture Boundaries

### Agent Module (`crates/domain/agent/`)

**Depends on:**
- `systemprompt-oauth` -- Authentication
- `systemprompt-users` -- User lookup
- `systemprompt-logging` -- Logging
- `systemprompt-database` -- Database pool
- `systemprompt-mcp` -- Tool orchestration (legitimate downward dependency)

### API Module (`crates/entry/api/`)

**Depends on:** All domain and app crates (entry layer wires everything)

### Scheduler Module (`crates/app/scheduler/`)

**Depends on:** Domain crates via `Job` trait abstractions

---

## Design Patterns

### Config Validation

Unified startup validation via `DomainConfig` trait and `StartupValidator`:

```rust
pub trait DomainConfig: Send + Sync {
    fn domain_id(&self) -> &'static str;
    fn load(&mut self, config: &Config) -> Result<(), DomainConfigError>;
    fn validate(&self) -> Result<ValidationReport, DomainConfigError>;
}
```

### Error Handling

Each domain defines its own error types. Use `thiserror` for derivation. Convert at boundaries using `From` implementations.

### Service Instantiation

Services receive dependencies through constructors, not global state:

```rust
// GOOD: Explicit dependencies
pub fn new(db: DbPool, config: &AiConfig) -> Self

// BAD: Service locator
pub fn new(app_context: &AppContext) -> Self  // Hides true dependencies
```

---

## Layer Violation Prevention

### Validation Commands

```bash
# Check for forbidden dependencies
grep "systemprompt-agent" crates/infra/*/Cargo.toml  # Should be empty
grep "systemprompt-ai" crates/infra/*/Cargo.toml     # Should be empty

# Verify domain isolation
grep "systemprompt-" crates/domain/*/Cargo.toml | grep -v "systemprompt-models\|systemprompt-traits\|systemprompt-identifiers\|systemprompt-database\|systemprompt-events"
```

### Core Rules

1. **Downward dependencies only** -- Higher layers depend on lower layers
2. **No cross-domain imports** except via public API for orchestration
3. **Config profiles required** -- No env var fallbacks
4. **Explicit subprocess propagation** -- Pass secrets directly
5. **Single implementations = concrete types** -- Traits only for polymorphism
