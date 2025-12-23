# Extension Architecture

This document defines the architecture for building extensions on systemprompt-core.

---

## Core Principle: Separation of Concerns

**If it's Rust code, it's an extension. If it's YAML/Markdown, it's a service.**

| Category | Purpose | Format | Location |
|----------|---------|--------|----------|
| **Extensions** | Rust implementations | `.rs` | `/extensions/` |
| **Services** | Declarative config | YAML/Markdown | `/services/` |

---

## Project Structure

```
systemprompt-template/
├── core/                          # READ-ONLY submodule
│   └── crates/                    # Core functionality
│       ├── shared/                # Types, traits, identifiers
│       ├── infra/                 # Database, events, security
│       ├── domain/                # Business domains
│       ├── app/                   # Orchestration, scheduling
│       └── entry/                 # API, CLI, TUI
│
├── extensions/                    # ALL Rust implementations
│   ├── blog/                      # Content management extension
│   └── mcp/                       # MCP servers (Rust crates)
│       ├── admin/                 # Admin tools server
│       ├── system-tools/          # File operations server
│       └── infrastructure/        # Deployment server
│
├── services/                      # PURE CONFIG (no .rs files)
│   ├── agents/                    # Agent YAML definitions
│   ├── ai/                        # AI provider configuration
│   ├── config/                    # Root configuration aggregator
│   ├── content/                   # Markdown content
│   ├── scheduler/                 # Job schedules (refs extension jobs)
│   ├── skills/                    # Skill definitions
│   └── web/                       # Theme configuration
│
├── src/
│   └── main.rs                    # Server entry point
└── Cargo.toml                     # Workspace root
```

---

## Layer Model

Extensions integrate with core at specific points:

```
┌─────────────────────────────────────────────────────────────────┐
│                    TEMPLATE (systemprompt-template)              │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                     src/main.rs                             │ │
│  │  - Loads configuration                                      │ │
│  │  - Connects to database                                     │ │
│  │  - Installs extension schemas                               │ │
│  │  - Mounts extension routers                                 │ │
│  │  - Starts server                                            │ │
│  └────────────────────────────────────────────────────────────┘ │
│                              │                                   │
│  ┌───────────────────────────┼────────────────────────────────┐ │
│  │                  extensions/                                │ │
│  │                                                             │ │
│  │   Extension provides:                                       │ │
│  │   - Schemas (SQL migrations)                                │ │
│  │   - Models (domain types)                                   │ │
│  │   - Repositories (data access)                              │ │
│  │   - Services (business logic)                               │ │
│  │   - API routes (HTTP handlers)                              │ │
│  │   - Jobs (background tasks)                                 │ │
│  │                                                             │ │
│  │   MCP Server provides:                                      │ │
│  │   - Tools (callable functions)                              │ │
│  │   - Prompts (templates)                                     │ │
│  │   - Resources (data sources)                                │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                              │                                   │
│  ┌───────────────────────────┼────────────────────────────────┐ │
│  │                  services/ (config only)                    │ │
│  │                                                             │ │
│  │   Configuration provides:                                   │ │
│  │   - Agent definitions (YAML)                                │ │
│  │   - AI provider routing (YAML)                              │ │
│  │   - Job schedules (references extension jobs)               │ │
│  │   - Theme settings (YAML)                                   │ │
│  │   - Markdown content                                        │ │
│  └─────────────────────────────────────────────────────────────┘ │
└──────────────────────────────┬──────────────────────────────────┘
                               │ imports
┌──────────────────────────────┴──────────────────────────────────┐
│                      CORE (systemprompt-core)                    │
│                                                                  │
│  Provides:                                                       │
│  - Traits (Extension, Job, LlmProvider, ToolProvider)           │
│  - Models (configuration, API types)                            │
│  - Identifiers (UserId, TaskId, ContentId, etc.)                │
│  - Database abstraction (pool, migrations)                      │
│  - Logging infrastructure                                        │
│  - Event system                                                  │
│  - Security (JWT, auth)                                          │
│  - Scheduler (job execution)                                     │
│  - Static site generator                                         │
│  - API server infrastructure                                     │
└──────────────────────────────────────────────────────────────────┘
```

---

## Extension Internal Structure

Every extension follows this internal layering:

```
┌─────────────────────────────────────────┐
│              API Layer                   │
│  src/api/handlers/*.rs                  │
│  - HTTP request handling                │
│  - Request validation                   │
│  - Response formatting                  │
└────────────────────┬────────────────────┘
                     │ calls
┌────────────────────▼────────────────────┐
│            Service Layer                 │
│  src/services/*.rs                      │
│  - Business logic                       │
│  - Orchestration                        │
│  - Error handling                       │
└────────────────────┬────────────────────┘
                     │ calls
┌────────────────────▼────────────────────┐
│          Repository Layer                │
│  src/repository/*.rs                    │
│  - SQL queries (sqlx macros)            │
│  - Data access                          │
│  - No business logic                    │
└────────────────────┬────────────────────┘
                     │ uses
┌────────────────────▼────────────────────┐
│            Model Layer                   │
│  src/models/*.rs                        │
│  - Domain types                         │
│  - DTOs                                 │
│  - Builders                             │
└─────────────────────────────────────────┘
```

---

## What Core Provides

### Shared Types (`systemprompt-models`, `systemprompt-identifiers`)

| Type | Purpose |
|------|---------|
| `UserId`, `TaskId`, `ContentId`, etc. | Type-safe identifiers |
| `Config` | Application configuration |
| `ApiError`, `ApiResponse` | HTTP response types |
| `RequestContext` | Request metadata for tracing |

### Traits (`systemprompt-traits`)

| Trait | Purpose |
|-------|---------|
| `Extension` | Unified extension interface (schemas, router, jobs, providers) |
| `Job` | Background job interface |
| `LlmProvider` | AI provider abstraction |
| `ToolProvider` | MCP tool abstraction |
| `ExtensionError` | Standardized error handling |

### Infrastructure (`systemprompt-core-database`, etc.)

| Crate | Purpose |
|-------|---------|
| `systemprompt-core-database` | SQLx pool, migrations |
| `systemprompt-core-logging` | Tracing setup, database layer |
| `systemprompt-core-config` | Configuration loading |
| `systemprompt-core-security` | JWT, authentication |
| `systemprompt-core-events` | Event bus, SSE |

### Runtime (`systemprompt-runtime`)

| Feature | Purpose |
|---------|---------|
| `AppContext` | Application-wide state |
| Lifecycle hooks | Startup, shutdown |
| Extension discovery | Auto-registration |

---

## What Extensions Provide

### Implementing the Extension Trait

Extensions should implement the unified `Extension` trait:

```rust
use systemprompt_traits::{Extension, ExtensionContext, ExtensionMetadata};

impl Extension for MyExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "my_extension",
            name: "My Extension",
            version: env!("CARGO_PKG_VERSION"),
            ..Default::default()
        }
    }

    fn schemas(&self) -> Vec<SchemaDefinition> {
        vec![SchemaDefinition::inline("table", SCHEMA_SQL)]
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

### Required Components

| Component | Location | Purpose |
|-----------|----------|---------|
| Extension impl | `src/extension.rs` | Implements `Extension` trait |
| Schemas | `schema/*.sql` | Database migrations |
| Error types | `src/error.rs` | Implements `ExtensionError` trait |

### Optional Components

| Component | Location | Purpose |
|-----------|----------|---------|
| Models | `src/models/` | Domain types |
| Repositories | `src/repository/` | Data access |
| Services | `src/services/` | Business logic |
| API routes | `src/api/` | HTTP endpoints |
| Jobs | `src/jobs/` | Background tasks |
| Configuration | `src/config.rs` | Extension config |

---

## MCP Server Structure

MCP servers are Rust crates in `extensions/mcp/`:

```
extensions/mcp/{name}/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point
│   ├── server/              # Server implementation
│   │   ├── mod.rs
│   │   ├── constructor.rs
│   │   └── handlers/
│   ├── tools/               # Tool implementations
│   │   ├── mod.rs
│   │   └── {tool_name}/
│   ├── prompts/             # Prompt templates
│   └── resources/           # Resource handlers
└── module.yml               # Server configuration
```

---

## Dependency Rules

### Extensions Can Import

| Allowed | Source |
|---------|--------|
| `systemprompt-models` | Core shared types |
| `systemprompt-identifiers` | Type-safe IDs |
| `systemprompt-traits` | Core traits |
| `systemprompt-core-database` | Database pool |
| `systemprompt-core-logging` | Tracing |
| `systemprompt-runtime` | AppContext |
| Other extensions | Via public API |

### Extensions Cannot Import

| Forbidden | Reason |
|-----------|--------|
| `systemprompt-core-api` | Entry layer |
| `systemprompt-core-scheduler` | App layer |
| Core domain crates directly | Use traits |

### MCP Servers Can Import

| Allowed | Source |
|---------|--------|
| `systemprompt-core-mcp` | Router, protocol |
| `systemprompt-models` | Shared types |
| Extensions | For tool implementations |

---

## Job Registration

Jobs are defined in Rust and scheduled via YAML:

### Define Job in Extension

```rust
use systemprompt_traits::{Job, JobContext, JobResult};

#[derive(Debug, Clone, Copy, Default)]
pub struct MyJob;

#[async_trait::async_trait]
impl Job for MyJob {
    fn name(&self) -> &'static str { "my_job" }
    fn description(&self) -> &'static str { "Does something" }
    fn schedule(&self) -> &'static str { "0 0 * * * *" }  // Default schedule

    async fn execute(&self, ctx: &JobContext) -> anyhow::Result<JobResult> {
        Ok(JobResult::success())
    }
}
```

### Register in Extension Trait

```rust
impl Extension for MyExtension {
    fn jobs(&self) -> Vec<Arc<dyn Job>> {
        vec![Arc::new(MyJob)]
    }
}
```

### Configure Schedule in YAML

```yaml
# services/scheduler/config.yml
scheduler:
  jobs:
    - extension: my_extension
      job: my_job
      schedule: "0 */15 * * * *"  # Override: every 15 minutes
      enabled: true
```

---

## Configuration Flow

```
services/config/config.yml          # Root config (aggregates includes)
       │
       ├── services/agents/*.yml        # Agent definitions
       ├── services/ai/config.yml       # AI providers
       ├── services/scheduler/config.yml # Job schedules
       ├── services/web/config.yml      # Theme
       └── .env.secrets                 # Secrets (not committed)
```

Services configure behavior. Extensions provide implementation.

---

## Static Content Generation

Extensions can integrate with core's Static Content Generator:

1. **Extension stores content** in database via repository
2. **Core's `PublishContentJob`** renders templates to HTML
3. **Static files** written to `dist/{slug}/index.html`
4. **Server** serves static files directly

The blog extension demonstrates this pattern.

---

## Naming Conventions

| Type | Pattern | Example |
|------|---------|---------|
| Extension crate | `systemprompt-{name}-extension` | `systemprompt-blog-extension` |
| MCP server crate | `systemprompt-mcp-{name}` | `systemprompt-mcp-admin` |
| Extension struct | `{Name}Extension` | `BlogExtension` |
| Service struct | `{Entity}Service` | `ContentService` |
| Repository struct | `{Entity}Repository` | `ContentRepository` |
| Job struct | `{Name}Job` | `ContentIngestionJob` |

---

## Idiomatic Rust Patterns

### Unified Extension Trait

Use default trait methods instead of multiple trait implementations:

```rust
// Good: Single trait with defaults
impl Extension for MyExtension {
    fn metadata(&self) -> ExtensionMetadata { ... }
    fn schemas(&self) -> Vec<SchemaDefinition> { ... }  // Override default
    fn router(&self, ctx: &ExtensionContext) -> Option<Router> { ... }
    // jobs() uses default (empty vec)
}

// Avoid: Separate trait impls
impl Extension for MyExtension { ... }
impl SchemaExtension for MyExtension { ... }  // Fragmented
impl ApiExtension for MyExtension { ... }     // Fragmented
```

### ExtensionError Trait

Implement `ExtensionError` for consistent error handling:

```rust
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
}
```

### Repository Column Constants

Reduce SQL repetition with column constants:

```rust
impl Content {
    pub const COLUMNS: &'static str = r#"
        id as "id: ContentId", slug, title, description, body
    "#;
}

impl ContentRepository {
    pub async fn get_by_id(&self, id: &ContentId) -> Result<Option<Content>> {
        let query = format!("SELECT {} FROM content WHERE id = $1", Content::COLUMNS);
        sqlx::query_as::<_, Content>(&query).bind(id.as_str()).fetch_optional(&*self.pool).await
    }
}
```
