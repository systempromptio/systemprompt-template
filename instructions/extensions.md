# Extension System

This document explains how to extend SystemPrompt Core in your template project.

---

## What is the Extension System?

The extension system allows you to add custom functionality to SystemPrompt without modifying the core codebase. Extensions are discovered at **compile time** and integrated automatically.

**Key concepts:**

- **Core** = `systemprompt-core` - the prebuilt platform (read-only)
- **Template** = `systemprompt-template` - your customization layer
- **Extension** = A Rust struct that implements extension traits

---

## The Facade Pattern

The `systemprompt-facade` crate provides a single entry point to all core functionality:

```
┌─────────────────────────────────────────────────────────┐
│                  systemprompt-facade                     │
│                                                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │
│  │  traits  │ │  models  │ │extension │ │ database │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘   │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │
│  │  system  │ │   api    │ │  agent   │ │   mcp    │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘   │
└─────────────────────────────────────────────────────────┘
                           ▲
                           │ depends on
                           │
               ┌───────────┴───────────┐
               │   Your Template Code   │
               └───────────────────────┘
```

**Why a facade?**

1. **Single dependency** - Add one crate instead of 15+
2. **Feature flags** - Only compile what you need
3. **Stable API** - Internal refactoring doesn't break your code
4. **Versioned releases** - Upgrade core with a version bump

### Feature Flags

```toml
[dependencies]
systemprompt-facade = { version = "0.1", features = ["api"] }
```

| Feature | What it includes |
|---------|------------------|
| `core` (default) | Traits, models, identifiers, extension framework |
| `database` | Database abstraction, repository patterns |
| `api` | HTTP server, AppContext, system module |
| `full` | Everything: agents, MCP, OAuth, users, content, analytics, scheduler |

---

## Extension Types

Extensions are organized by what they provide:

| Extension Type | Purpose | Example Use Case |
|----------------|---------|------------------|
| `Extension` | Base metadata | All extensions implement this |
| `SchemaExtension` | Database tables | Add custom tables for your domain |
| `ApiExtension` | HTTP routes | Add `/api/v1/myfeature` endpoints |
| `ConfigExtension` | Custom config | Validate your YAML configuration |
| `JobExtension` | Background jobs | Scheduled tasks, cron jobs |
| `ProviderExtension` | LLM/Tool providers | Custom AI provider integration |

---

## How Discovery Works

Extensions use the `inventory` crate for **compile-time registration**:

```
Compile Time                          Runtime
─────────────                         ───────

┌─────────────────┐                   ┌──────────────────┐
│ register_ext!() │ ─── inventory ──► │ ExtensionRegistry│
│ register_ext!() │     collects      │   .discover()    │
│ register_ext!() │     all macros    │                  │
└─────────────────┘                   └──────────────────┘
                                              │
                                              ▼
                                      ┌──────────────────┐
                                      │  AppContext has  │
                                      │  all extensions  │
                                      └──────────────────┘
```

**No runtime configuration needed** - just define your extension and register it.

---

## Creating Extensions

### 1. Basic Extension (Metadata Only)

```rust
use systemprompt_facade::prelude::*;

#[derive(Default)]
struct MyExtension;

impl Extension for MyExtension {
    fn id(&self) -> &str { "my-extension" }
    fn name(&self) -> &str { "My Custom Extension" }
    fn version(&self) -> &str { "1.0.0" }

    // Optional: declare dependencies
    fn dependencies(&self) -> &[&str] { &["users"] }

    // Optional: control init order (lower = earlier)
    fn priority(&self) -> u32 { 100 }
}

// Register at compile time
register_extension!(MyExtension);
```

### 2. API Extension (Add HTTP Routes)

```rust
use systemprompt_facade::prelude::*;
use axum::{Router, routing::get, Json};

#[derive(Default)]
struct InventoryExtension;

impl Extension for InventoryExtension {
    fn id(&self) -> &str { "inventory" }
    fn name(&self) -> &str { "Inventory Management" }
    fn version(&self) -> &str { "1.0.0" }
}

impl ApiExtension for InventoryExtension {
    fn router(&self, _ctx: &dyn ExtensionContext) -> Router {
        Router::new()
            .route("/items", get(list_items))
            .route("/items/:id", get(get_item))
    }

    fn base_path(&self) -> &str { "/api/v1/inventory" }

    fn requires_auth(&self) -> bool { true }
}

async fn list_items() -> Json<Vec<String>> {
    Json(vec!["item1".to_string(), "item2".to_string()])
}

async fn get_item() -> &'static str {
    "Item details"
}

register_extension!(InventoryExtension);
register_api_extension!(InventoryExtension);
```

### 3. Schema Extension (Add Database Tables)

```rust
use systemprompt_facade::prelude::*;

#[derive(Default)]
struct InventoryExtension;

impl Extension for InventoryExtension {
    fn id(&self) -> &str { "inventory" }
    fn name(&self) -> &str { "Inventory" }
    fn version(&self) -> &str { "1.0.0" }
}

impl SchemaExtension for InventoryExtension {
    fn schemas(&self) -> Vec<SchemaDefinition> {
        vec![
            SchemaDefinition::inline(
                "inventory_items",
                r#"
                CREATE TABLE IF NOT EXISTS inventory_items (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    name TEXT NOT NULL,
                    quantity INTEGER NOT NULL DEFAULT 0,
                    created_at TIMESTAMPTZ DEFAULT NOW()
                )
                "#
            ).with_required_columns(vec![
                "id".to_string(),
                "name".to_string(),
                "quantity".to_string(),
            ]),
        ]
    }

    // Core tables use 1-10, extensions use 100+
    fn migration_weight(&self) -> u32 { 100 }
}

register_extension!(InventoryExtension);
register_schema_extension!(InventoryExtension);
```

### 4. Provider Extension (Custom LLM)

```rust
use systemprompt_facade::prelude::*;
use std::sync::Arc;

struct MyLlmProvider { /* ... */ }

#[async_trait::async_trait]
impl LlmProvider for MyLlmProvider {
    async fn chat(&self, request: &ChatRequest) -> LlmProviderResult<ChatResponse> {
        // Your implementation
    }
    // ... other methods
}

#[derive(Default)]
struct MyLlmExtension;

impl Extension for MyLlmExtension {
    fn id(&self) -> &str { "my-llm" }
    fn name(&self) -> &str { "My LLM Provider" }
    fn version(&self) -> &str { "1.0.0" }
}

impl ProviderExtension for MyLlmExtension {
    fn llm_providers(&self) -> Vec<Arc<dyn LlmProvider>> {
        vec![Arc::new(MyLlmProvider::new())]
    }
}

register_extension!(MyLlmExtension);
register_provider_extension!(MyLlmExtension);
```

---

## Startup Flow

When your application starts:

```rust
use systemprompt_facade::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Discover all registered extensions
    let registry = ExtensionRegistry::discover();

    // 2. Validate dependencies and paths
    registry.validate()?;

    // 3. Build AppContext with extensions
    let ctx = AppContext::builder()
        .with_extensions(registry)
        .build()
        .await?;

    // 4. Install extension schemas (runs migrations)
    install_extension_schemas(
        ctx.extension_registry(),
        ctx.database()
    ).await?;

    // 5. Start server (extension routes auto-mounted)
    setup_api_server(&ctx)?.serve(&ctx.server_address()).await
}
```

---

## Reserved API Paths

Extensions **cannot** use these paths (reserved for core):

- `/api/v1/oauth`
- `/api/v1/users`
- `/api/v1/agents`
- `/api/v1/mcp`
- `/api/v1/stream`
- `/api/v1/content`
- `/api/v1/files`
- `/api/v1/analytics`
- `/api/v1/scheduler`
- `/api/v1/core`
- `/.well-known`

All extension paths must start with `/api/`.

---

## Progressive Complexity

Start simple, add complexity as needed:

| Level | What You Create | Code Required |
|-------|----------------|---------------|
| 1. Config-only | YAML files for agents, skills, MCP | None |
| 2. Custom provider | Implement `LlmProvider` or `ToolProvider` | ~100 lines |
| 3. API extension | New HTTP endpoints | ~50-200 lines |
| 4. Full domain | Database + API + services + jobs | Full crate |

---

## File Structure for Extensions

```
services/
  my-extension/
    Cargo.toml
    src/
      lib.rs          # Extension registration
      api.rs          # Route handlers (if ApiExtension)
      schema/         # SQL files (if SchemaExtension)
        001_init.sql
      models.rs       # Your domain types
      repository.rs   # Database access
      service.rs      # Business logic
```

**Cargo.toml:**

```toml
[package]
name = "my-extension"
version = "0.1.0"
edition = "2021"

[dependencies]
systemprompt-facade = { version = "0.1", features = ["api"] }
axum = "0.8"
async-trait = "0.1"
tokio = { version = "1", features = ["full"] }
```

---

## Debugging Extensions

**Check if your extension is registered:**

```rust
let registry = ExtensionRegistry::discover();
println!("Extensions: {:?}", registry.ids());
println!("API extensions: {}", registry.api_extensions().len());
```

**Common issues:**

1. **Extension not found** - Forgot `register_extension!()` macro
2. **Path collision** - Using a reserved path
3. **Missing dependency** - Another extension required but not registered
4. **Schema failed** - SQL syntax error or table already exists
