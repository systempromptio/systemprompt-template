# Logging Migration Plan: LogService → tracing + DatabaseLayer

## Overview

Replace the custom `LogService` with idiomatic `tracing` infrastructure. Each process initializes its own `DatabaseLayer` with its own `DbPool`. PostgreSQL handles concurrent writes atomically — no coordination needed.

---

## Multi-Process Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   API Server    │    │  Agent Process  │    │   Scheduler     │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │DatabaseLayer│ │    │ │DatabaseLayer│ │    │ │DatabaseLayer│ │
│ │  (DbPool)   │ │    │ │  (DbPool)   │ │    │ │  (DbPool)   │ │
│ └──────┬──────┘ │    │ └──────┬──────┘ │    │ └──────┬──────┘ │
└────────┼────────┘    └────────┼────────┘    └────────┼────────┘
         │                      │                      │
         └──────────────────────┼──────────────────────┘
                                │
                                ▼
                    ┌───────────────────────┐
                    │   PostgreSQL          │
                    │   logs table          │
                    │   (concurrent writes) │
                    └───────────────────────┘
```

**How it works:**
1. Each process calls `init_logging(db_pool)` at startup
2. Each process has its own `DatabaseLayer` with its own connection pool
3. PostgreSQL handles concurrent INSERTs atomically — no locks, no conflicts
4. `trace_id` field correlates logs across processes for the same request

---

## Implementation

### Phase 1: Create DatabaseLayer

**File:** `crates/modules/log/src/layer.rs`

```rust
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{Event, Subscriber};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

pub struct DatabaseLayer {
    sender: mpsc::UnboundedSender<LogEntry>,
}

impl DatabaseLayer {
    pub fn new(db_pool: DbPool) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        // Spawn background task for batched writes
        tokio::spawn(Self::batch_writer(db_pool, receiver));

        Self { sender }
    }

    async fn batch_writer(
        db_pool: DbPool,
        mut receiver: mpsc::UnboundedReceiver<LogEntry>,
    ) {
        let mut buffer = Vec::with_capacity(100);
        let mut interval = tokio::time::interval(Duration::from_secs(10));

        loop {
            tokio::select! {
                Some(entry) = receiver.recv() => {
                    buffer.push(entry);
                    if buffer.len() >= 100 {
                        Self::flush(&db_pool, &mut buffer).await;
                    }
                }
                _ = interval.tick() => {
                    if !buffer.is_empty() {
                        Self::flush(&db_pool, &mut buffer).await;
                    }
                }
            }
        }
    }

    async fn flush(db_pool: &DbPool, buffer: &mut Vec<LogEntry>) {
        // Reuse existing batch insert logic from buffered.rs
        if let Err(e) = batch_insert_logs(db_pool, buffer).await {
            eprintln!("Failed to flush logs: {e}");
        }
        buffer.clear();
    }
}

impl<S> Layer<S> for DatabaseLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        // Extract level, message, module from event
        let level = *event.metadata().level();
        let module = event.metadata().target().to_string();

        // Extract message and fields
        let mut visitor = FieldVisitor::default();
        event.record(&mut visitor);

        // Extract context from current span chain
        let span_context = ctx.current_span()
            .id()
            .and_then(|id| ctx.span(id))
            .map(|span| extract_span_context(span));

        let entry = LogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level: level.into(),
            module,
            message: visitor.message,
            metadata: visitor.fields,
            user_id: span_context.as_ref().and_then(|c| c.user_id.clone()),
            session_id: span_context.as_ref().and_then(|c| c.session_id.clone()),
            task_id: span_context.as_ref().and_then(|c| c.task_id.clone()),
            trace_id: span_context.as_ref().and_then(|c| c.trace_id.clone()),
            context_id: span_context.as_ref().and_then(|c| c.context_id.clone()),
            client_id: span_context.as_ref().and_then(|c| c.client_id.clone()),
        };

        // Non-blocking send to background writer
        let _ = self.sender.send(entry);
    }
}
```

### Phase 2: Initialization Function

**File:** `crates/modules/log/src/lib.rs`

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

pub fn init_logging(db_pool: DbPool) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_filter(env_filter.clone());

    let db_layer = DatabaseLayer::new(db_pool)
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(db_layer)
        .init();
}

// For processes that only want console output
pub fn init_console_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .init();
}
```

### Phase 3: Context Propagation via Spans

**File:** `crates/modules/log/src/spans.rs`

Two span types matching current LogService patterns:

```rust
use systemprompt_identifiers::{UserId, SessionId, TraceId, TaskId, ContextId, ClientId};
use tracing::Span;

/// RequestSpan - matches LogService::new(db, ctx.log_context())
/// Enforces required fields at compile time.
pub struct RequestSpan(Span);

impl RequestSpan {
    /// Create from RequestContext - mirrors ctx.log_context()
    pub fn from_context(ctx: &RequestContext) -> Self {
        let span = tracing::info_span!(
            "request",
            user_id = %ctx.auth.user_id,
            session_id = %ctx.request.session_id,
            trace_id = %ctx.execution.trace_id,
            context_id = tracing::field::Empty,
            task_id = tracing::field::Empty,
            client_id = tracing::field::Empty,
        );

        let result = Self(span);

        // Set optional fields if present (matches log_context() logic)
        if !ctx.execution.context_id.as_str().is_empty() {
            result.record_context_id(&ctx.execution.context_id);
        }
        if let Some(ref task_id) = ctx.execution.task_id {
            result.record_task_id(task_id);
        }
        if let Some(ref client_id) = ctx.request.client_id {
            result.record_client_id(client_id);
        }

        result
    }

    pub fn enter(&self) -> tracing::span::EnteredSpan {
        self.0.clone().entered()
    }

    pub fn record_task_id(&self, task_id: &TaskId) {
        self.0.record("task_id", task_id.as_str());
    }

    pub fn record_context_id(&self, context_id: &ContextId) {
        self.0.record("context_id", context_id.as_str());
    }

    pub fn record_client_id(&self, client_id: &ClientId) {
        self.0.record("client_id", client_id.as_str());
    }
}

/// SystemSpan - matches LogService::system(db)
/// For background tasks, system operations without request context.
pub struct SystemSpan(Span);

impl SystemSpan {
    pub fn new(component: &str) -> Self {
        Self(tracing::info_span!(
            "system",
            user_id = "system",
            session_id = "system",
            trace_id = %TraceId::generate(),
            client_id = %format!("system:{}", component),
            context_id = tracing::field::Empty,
            task_id = tracing::field::Empty,
        ))
    }

    pub fn enter(&self) -> tracing::span::EnteredSpan {
        self.0.clone().entered()
    }
}

/// Extension trait for RequestContext - ergonomic API
impl RequestContext {
    pub fn span(&self) -> RequestSpan {
        RequestSpan::from_context(self)
    }
}
```

**Usage parity:**

| Before | After |
|--------|-------|
| `LogService::new(db, ctx.log_context())` | `let _guard = ctx.span().enter();` |
| `LogService::system(db)` | `let _guard = SystemSpan::new("scheduler").enter();` |
| `log_context.with_task_id(id)` | `span.record_task_id(&id);` |

---

## Migration Instructions

### Step 1: Update Process Entry Points

Each process must initialize logging at startup:

**API Server** (`crates/modules/api/src/services/server/runner.rs`):
```rust
pub async fn run(ctx: Arc<AppContext>) -> Result<()> {
    systemprompt_log::init_logging(ctx.db_pool().clone());
    // ... rest of server startup
}
```

**Agent Process** (`crates/modules/agent/src/services/a2a_server/server.rs`):
```rust
pub async fn start(db_pool: DbPool) -> Result<()> {
    systemprompt_log::init_logging(db_pool.clone());
    // ... rest of agent startup
}
```

**Scheduler** (`crates/modules/scheduler/src/services/scheduler.rs`):
```rust
pub async fn run(db_pool: DbPool) -> Result<()> {
    systemprompt_log::init_logging(db_pool.clone());
    // ... rest of scheduler startup
}
```

**CLI** (`src/main.rs`):
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Console-only for CLI (DB logging happens in spawned processes)
    systemprompt_log::init_console_logging();
    // ...
}
```

### Step 2: Replace LogService Calls

**Pattern 1: Request-scoped logging (most common)**
```rust
// Before
let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());
logger.info("handler", "Processing request").await.ok();
logger.error("handler", &format!("Failed: {e}")).await.ok();

// After
let _guard = req_ctx.span().enter();
tracing::info!("Processing request");
tracing::error!(error = %e, "Failed");
```

**Pattern 2: System/background logging**
```rust
// Before
let logger = LogService::system(db_pool.clone());
logger.info("scheduler", "Running cleanup job").await.ok();

// After
let _guard = SystemSpan::new("scheduler").enter();
tracing::info!("Running cleanup job");
```

**Pattern 3: Adding task_id mid-request**
```rust
// Before
let log_context = req_ctx.log_context().with_task_id(&request.entity_id);
let logger = LogService::new(db.clone(), log_context);

// After
let span = req_ctx.span();
span.record_task_id(&request.entity_id);
let _guard = span.enter();
```

**Logging method mapping:**

| Before | After |
|--------|-------|
| `logger.info("mod", "msg").await.ok();` | `tracing::info!("msg");` |
| `logger.error("mod", &format!("Failed: {e}")).await.ok();` | `tracing::error!(error = %e, "Failed");` |
| `logger.warn("mod", "msg").await.ok();` | `tracing::warn!("msg");` |
| `logger.debug("mod", "msg").await.ok();` | `tracing::debug!("msg");` |

**With structured fields:**
```rust
// Before
logger.info("user_service", &format!("Created user {}", user.id)).await.ok();

// After
tracing::info!(user_id = %user.id, "Created user");
```

### Step 3: Complete Handler Example

```rust
// Before
async fn create_context(
    ctx: Extension<Arc<AppContext>>,
    req_ctx: RequestContext,
    Json(request): Json<CreateContextRequest>,
) -> Result<Json<Context>, ApiError> {
    let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());
    logger.info("contexts", "Creating new context").await.ok();

    let context = context_service.create(&request).await.map_err(|e| {
        logger.error("contexts", &format!("Failed to create context: {e}")).await.ok();
        ApiError::from(e)
    })?;

    logger.info("contexts", &format!("Created context {}", context.id)).await.ok();
    Ok(Json(context))
}

// After
async fn create_context(
    ctx: Extension<Arc<AppContext>>,
    req_ctx: RequestContext,
    Json(request): Json<CreateContextRequest>,
) -> Result<Json<Context>, ApiError> {
    let _guard = req_ctx.span().enter();
    tracing::info!("Creating new context");

    let context = context_service.create(&request).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to create context");
        ApiError::from(e)
    })?;

    tracing::info!(context_id = %context.id, "Created context");
    Ok(Json(context))
}
```

---

## Files to Modify

### Core Implementation (create/modify)

| File | Action | Purpose |
|------|--------|---------|
| `crates/modules/log/src/layer.rs` | Create | DatabaseLayer implementation |
| `crates/modules/log/src/spans.rs` | Create | Span helper functions |
| `crates/modules/log/src/lib.rs` | Modify | Add init_logging(), remove LogService export |
| `crates/modules/log/src/services/mod.rs` | Modify | Export new modules |
| `crates/modules/log/Cargo.toml` | Modify | Add tracing-subscriber features |

### Process Entry Points (modify)

| File | Change |
|------|--------|
| `crates/modules/api/src/services/server/runner.rs` | Add init_logging() |
| `crates/modules/agent/src/services/a2a_server/server.rs` | Add init_logging() |
| `crates/modules/scheduler/src/services/scheduler.rs` | Add init_logging() |
| `src/main.rs` | Add init_console_logging() |

### Call Site Migration (120 files)

**By crate (file count):**
- `agent` - 52 files
- `scheduler` - 16 files
- `oauth` - 16 files
- `api` - 12 files
- `ai` - 10 files
- `mcp` - 8 files
- `blog` - 2 files
- `core` - 1 file
- `cli` - 1 file
- `shared/models` - 1 file

---

## Database Schema

**No changes required.** The existing `logs` table schema works perfectly:

```sql
CREATE TABLE logs (
    id TEXT PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL,
    level VARCHAR(50) NOT NULL,
    module VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    metadata TEXT,
    user_id VARCHAR(255),
    session_id VARCHAR(255),
    task_id VARCHAR(255),
    trace_id VARCHAR(255),
    context_id VARCHAR(255),
    client_id VARCHAR(255)
);
```

---

## Execution Order

1. **Create core infrastructure** (layer.rs, spans.rs, update lib.rs)
2. **Update process entry points** (4 files)
3. **Migrate by crate** (start with smallest: blog, core, cli)
4. **Delete LogService** (after all migrations complete)

---

## Verification

After each crate migration:

```bash
# Run the process
cargo run -- services start

# Check logs table has entries
psql -c "SELECT * FROM logs ORDER BY timestamp DESC LIMIT 10;"

# Verify structured fields
psql -c "SELECT trace_id, user_id, module, message FROM logs WHERE trace_id IS NOT NULL LIMIT 5;"
```

---

## Deleted After Migration

- `crates/modules/log/src/services/logger.rs` - LogService
- `crates/modules/log/src/services/buffered.rs` - BufferedLogService (logic moved to layer.rs)
- `crates/modules/log/src/models/log_context.rs` - LogContext (replaced by spans)

---

## Update Instruction Files

### instructions/rust.md

Replace the Logging section (Section 3: Mandatory Patterns → Logging) with:

```markdown
### Logging

All logging via `tracing` with database persistence via `DatabaseLayer`.

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
```

### instructions/rust-checklist.md

Replace R3.2 in Section 3 (Mandatory Patterns) and add new logging checks:

```markdown
| R3.2 | Logging via `tracing` with spans | No `LogService`, no `println!`, no direct `log::` |
```

Add new Section after Naming:

```markdown
### Section 5: Logging (rust.md)

| ID | Rule | Check |
|----|------|-------|
| R5.1 | Request handlers use `req_ctx.span()` | Verify handlers enter request span |
| R5.2 | Background tasks use `SystemSpan` | Verify schedulers/background use SystemSpan |
| R5.3 | No `LogService` usage | Search for `LogService::` - must be zero |
| R5.4 | No orphan `tracing::` calls | All tracing calls inside entered span |
| R5.5 | Structured fields over format strings | Prefer `field = %value` over `format!()` |
```

Update execution commands:

```bash
# Check for forbidden logging patterns
grep -rn "LogService::" --include="*.rs" [target]  # Must be zero
grep -rn "println!" --include="*.rs" [target]      # Must be zero in libs
grep -rn "log::" --include="*.rs" [target]         # Must be zero
```

### instructions/review.md

Add logging check to the Review Checklist section:

```markdown
- [ ] Logging uses `tracing` with proper spans - no `LogService` usage
- [ ] Request handlers enter `req_ctx.span()` before logging
- [ ] Background tasks use `SystemSpan::new("component")`
- [ ] Structured fields used instead of format strings where applicable
```

Add to Review Output Format:

```markdown
15. **Logging patterns:** Correct / Violation: [describe]
```
