---
name: "Architecture Standards"
description: "Layer architecture, module boundaries, extension registration, plugin structure, config system, and build pipeline standards for foodles.com development"
---

# foodles.com Architecture Standards

**foodles.com is a world-class Rust programming brand.** Every architectural decision must enforce strict layer boundaries, explicit dependency direction, and zero-tolerance for cross-cutting violations.

---

## 1. Crate Layers

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
Entry (api, cli)
    |
    v
App (runtime, scheduler, generator)
    |
    v
Domain (agent, ai, mcp, oauth, users, files, content, analytics, templates)
    |
    v
Infra (database, events, security, config, logging, loader, cloud)
    |
    v
Shared (models, traits, identifiers, extension, provider-contracts, client)
```

### Forbidden Dependencies

| Layer | Cannot Depend On |
|-------|------------------|
| Shared | Any systemprompt crate (except within shared/) |
| Infra | domain/, app/, entry/ |
| Domain | Other domain crates, app/, entry/ |
| App | entry/ |

---

## 2. Layer Definitions

### Shared (`crates/shared/`)

Pure types with zero dependencies on other systemprompt crates.

| Rule | Enforcement |
|------|-------------|
| No SQL/Database | `grep "sqlx" crates/shared/*/Cargo.toml` must be empty |
| No Repository | No repository modules |
| No Service | No service modules |
| No State | No singletons, no mutability |
| No I/O | No file, network, database |

| Crate | Purpose |
|-------|---------|
| `provider-contracts/` | Provider trait contracts (`LlmProvider`, `ToolProvider`, `Job`, `ComponentRenderer`) |
| `identifiers/` | Typed IDs (`UserId`, `TaskId`, `SessionId`, `AgentId`, `ContextId`, `TraceId`) |
| `models/` | Domain models, API types, configuration structs, validation report types |
| `traits/` | Infrastructure trait definitions (`DomainConfig`, `ConfigProvider`, `DatabaseHandle`) |
| `template-provider/` | Template loading and rendering abstractions |
| `client/` | HTTP client for external API access |
| `extension/` | Extension framework for user customization |

### Infrastructure (`crates/infra/`)

Stateless utilities. May have I/O but no persistent domain state. Can depend on `shared/` only.

| Crate | Purpose |
|-------|---------|
| `database/` | SQLx abstraction, connection pooling, base repository trait |
| `events/` | Event bus, broadcasters, SSE infrastructure |
| `security/` | JWT validation, token extraction, cookie handling |
| `config/` | Configuration loading, environment handling |
| `logging/` | Tracing setup, log sinks, database layer |
| `cloud/` | Cloud API client, tenant management |

### Domain (`crates/domain/`)

Full bounded contexts. Each crate owns its database tables, repositories, and services. Can depend on `shared/` and `infra/`.

Required structure:

```
domain/{name}/
  Cargo.toml
  schema/             # SQL schema files
  src/
    lib.rs            # Public API, exports extension
    extension.rs      # Extension trait implementation
    error.rs          # Domain-specific errors
    models/           # Domain models
    repository/       # Data access layer
    services/         # Business logic
```

| Crate | Bounded Context |
|-------|-----------------|
| `users/` | User identity |
| `oauth/` | Authentication |
| `files/` | File storage |
| `analytics/` | Metrics & tracking |
| `content/` | Content management |
| `ai/` | LLM integration |
| `mcp/` | MCP protocol |
| `agent/` | A2A protocol |
| `templates/` | Template registry and rendering |

### Application (`crates/app/`)

Orchestration without business logic. Can depend on `shared/`, `infra/`, `domain/`.

| Crate | Purpose |
|-------|---------|
| `scheduler/` | Job scheduling, cron execution |
| `generator/` | Static site generation |
| `runtime/` | StartupValidator, AppContext, lifecycle management |

### Entry (`crates/entry/`)

Entry points that wire the application together. Can depend on all layers.

| Crate | Purpose |
|-------|---------|
| `cli/` | Command-line interface |
| `api/` | HTTP gateway, route handlers, middleware |

---

## 3. Module Boundary Guidelines

### Repositories Are Public API

Using a repository from another module is the correct pattern for cross-module data access. No extra abstraction layers needed.

### Avoid Over-Abstraction

If only one implementation exists, use the concrete type. Traits add complexity without benefit for single implementations.

### Service Instantiation

Services receive dependencies through constructors, not global state:

```rust
pub fn new(db: DbPool, config: &AiConfig) -> Self

pub fn new(app_context: &AppContext) -> Self  // BAD: Hides true dependencies
```

---

## 4. Extension Registration

Extensions register via `register_extension!()` macro from `systemprompt_extension::prelude`:

```rust
use systemprompt::extension::prelude::*;

struct MyExtension;

impl Extension for MyExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "my-extension",
            name: "My Extension",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    fn schemas(&self) -> Vec<SchemaDefinition> {
        vec![SchemaDefinition::inline("table", include_str!("../schema/table.sql"))]
    }

    fn migration_weight(&self) -> u32 { 100 }
}

register_extension!(MyExtension);
```

### Migration Weights

| Weight | Scope |
|--------|-------|
| 1-10 | Core modules (database, users, oauth) |
| 11-40 | Domain modules (ai, mcp, agent, content) |
| 100+ | User extensions |

### Product Binary Pattern

The `inventory` crate registers extensions at compile time. Extensions must be linked into the final binary:

```rust
use my_product as _;  // Forces linkage

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    systemprompt_cli::run().await
}
```

---

## 5. Plugin Structure

```
services/plugins/{plugin-id}/
  config.yaml              # Plugin manifest
  skills/
    {skill-name}/
      SKILL.md             # Generated skill content (do not edit)
  agents/
    {agent-name}.md        # Generated agent docs (do not edit)
```

### Plugin Config Format

```yaml
plugin:
  id: my-plugin
  name: "My Plugin"
  description: "Description"
  version: "1.0.0"
  enabled: true
  skills:
    source: explicit
    include:
      - skill_id_1
  agents:
    source: explicit
    include:
      - agent_name
  mcp_servers: []
  roles:
    - dev
  keywords:
    - keyword1
  category: development
  author:
    name: "foodles.com"
```

### Skill Config Format

```yaml
id: skill_id
name: "Skill Name"
description: "Description"
enabled: true
version: "1.0.0"
file: "index.md"
assigned_agents:
  - agent_name
tags:
  - tag1
```

The actual skill content lives in `services/skills/{id}/index.md`. The plugin's `SKILL.md` is generated from it by `systemprompt core plugins generate`.

---

## 6. Config System

### Config Profiles Are Mandatory

All code must use config profiles. No environment variable fallbacks:

```rust
let path = &config.web_path;  // Required field, validated at startup

let path = std::env::var("SYSTEMPROMPT_WEB_PATH").unwrap_or_default();  // FORBIDDEN
```

### Startup Validation

1. `ProfileBootstrap::init()` -- load profile YAML
2. `Config::from_profile()` -- build config, validate paths
3. `StartupValidator::validate()` -- run all domain + extension validators
4. Errors halt startup. No `--force` bypass.

### Subprocess Propagation

When spawning subprocesses (agents, MCP servers), pass explicitly:

| Env Var | Purpose |
|---------|---------|
| `SYSTEMPROMPT_PROFILE` | Path to profile.yaml |
| `JWT_SECRET` | JWT signing secret |
| `DATABASE_URL` | Database connection string |

---

## 7. Database Conventions

| Convention | Rule |
|------------|------|
| Timestamps | Always `TIMESTAMPTZ` in PostgreSQL, `DateTime<Utc>` in Rust |
| Identifiers | Use typed wrappers from `systemprompt_identifiers` |
| Queries | Only `sqlx::query!()`, `sqlx::query_as!()`, `sqlx::query_scalar!()` |
| Raw queries | `sqlx::query()` and `sqlx::query_as()` are FORBIDDEN |
| Schema location | `{crate}/schema/{table}.sql` embedded via `include_str!()` |
| Test location | ALL tests in `crates/tests/` -- never `#[cfg(test)]` in source |

---

## 8. File Organization Map

```
systemprompt-claude-marketplace/
  core/                          # READ-ONLY git submodule
  extensions/
    web/src/                     # Web extension (Rust)
    mcp/systemprompt/            # MCP server (Rust)
  services/
    agents/*.yaml                # Agent configs
    skills/{id}/config.yaml      # Skill configs
    skills/{id}/index.md         # Skill content (source of truth)
    plugins/{id}/config.yaml     # Plugin configs
    mcp/*.yaml                   # MCP server configs
    config/config.yaml           # Master config (includes all)
    hooks/                       # Hook scripts
    web/templates/               # Handlebars templates
  storage/files/
    css/admin/                   # CSS source files
    js/admin/                    # JS source files
  web/dist/                      # Generated output (never edit)
```

### Critical Rules

| Rule | Description |
|------|-------------|
| `core/` is READ-ONLY | Never modify. It is a git submodule. |
| Rust code in `extensions/` | All `.rs` files live here |
| Config only in `services/` | YAML/Markdown only. No Rust code. |
| CSS in `storage/files/css/` | NEVER in `extensions/*/assets/css/` |
| Brand name | `foodles.com` -- always lowercase |
| Not a framework | It is a library. NEVER call it a "framework". |

---

## 9. Build Pipeline

```bash
just build          # Build Rust workspace
just clippy         # Run clippy with -D warnings
just publish        # Compile templates, bundle CSS/JS, copy assets, prerender
just start          # Start services
just migrate        # Run database migrations
```

### Publish Flow (strict order)

1. `compile_admin_templates` -- Handlebars partials to compiled HTML
2. `bundle_admin_css` -- Concatenate CSS per `CSS_MODULE_ORDER`
3. `bundle_admin_js` -- Concatenate JS per `bundle-order.txt` + per-page bundles
4. `copy_extension_assets` -- Copy bundles to `web/dist/`
5. `content_prerender` -- Generate static content pages

### Service Configuration

| Setting | Value |
|---------|-------|
| Agent port range | 9000-9999 |
| MCP port range | 5000-5999 |
| Auto-start | Enabled |
| Validation | Strict |

---

## 10. Detection Commands

```bash
# Layer violations: infra depending on domain
grep "systemprompt-agent" crates/infra/*/Cargo.toml
grep "systemprompt-ai" crates/infra/*/Cargo.toml

# Domain isolation: cross-domain deps
grep "systemprompt-" crates/domain/*/Cargo.toml | grep -v "systemprompt-models\|systemprompt-traits\|systemprompt-identifiers\|systemprompt-database\|systemprompt-events"

# Shared layer purity: no SQL
grep "sqlx" crates/shared/*/Cargo.toml

# Forbidden constructs
rg '#\[cfg\(test\)\]' --type rust -g '!target/*'
rg 'env::var\(' --type rust -g '!*test*' -g '!target/*'

# CSS location violations
find extensions/ -path '*/assets/css/*' -name '*.css' 2>/dev/null

# Brand violations
rg -i 'SystemPrompt' --type html --type rust -g '!target/*' | grep -v 'systemprompt'
rg 'framework' --type html --type rust -g '!target/*' -g '!*test*'
```
