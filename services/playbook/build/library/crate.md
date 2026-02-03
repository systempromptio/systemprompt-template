---
title: "SystemPrompt Crate Playbook"
description: "Using the systemprompt umbrella crate for extensions."
keywords:
  - crate
  - umbrella
  - prelude
  - features
category: build
---

# The `systemprompt` Umbrella Crate

The `systemprompt` crate is a facade/umbrella crate that provides a single entry point to the entire SystemPrompt framework.

> **Help**: `{ "command": "playbook build" }` via `systemprompt_help`

---

## Why an Umbrella Crate?

Instead of depending on multiple internal crates:

```toml
# Without umbrella crate (verbose)
systemprompt-models = "0.1"
systemprompt-identifiers = "0.1"
systemprompt-traits = "0.1"
systemprompt-core-database = "0.1"
systemprompt-runtime = "0.1"
```

You use a single dependency:

```toml
# With umbrella crate (simple)
systemprompt = { version = "0.1", features = ["api"] }
```

This pattern is used by `tokio`, `bevy`, and `aws-sdk-rust`.

---

## Architecture

```
systemprompt/
├── src/
│   ├── lib.rs       # Re-exports internal crates as modules
│   └── prelude.rs   # Convenient glob import for extension authors
└── Cargo.toml       # Features control what's included
```

### Feature-Gated Re-exports (`lib.rs`)

Each internal crate becomes a public module controlled by features:

```rust
#[cfg(feature = "core")]
pub mod models {
    pub use systemprompt_models::*;
}

#[cfg(feature = "database")]
pub mod database {
    pub use systemprompt_core_database::*;
}

#[cfg(feature = "api")]
pub mod api {
    pub use systemprompt_core_api::*;
}
```

### The Prelude (`prelude.rs`)

The prelude provides common imports for extension authors:

```rust
use systemprompt::prelude::*;

// This single import gives you:
// - Extension, ExtensionMetadata, ExtensionContext
// - Job, JobContext, JobResult
// - ExtensionError, ApiError
// - Router, Arc, PgPool
// - register_extension! macro
```

---

## Feature Flags

| Feature | What's Included | Use Case |
|---------|-----------------|----------|
| `core` (default) | Traits, models, identifiers, extension framework | Minimal extensions |
| `database` | Database abstraction, repository patterns | Extensions with persistence |
| `mcp` | MCP derive macros (`#[tool]`, `#[mcp_server]`) | MCP server development |
| `api` | HTTP server, AppContext, includes `core` + `database` | Full API extensions |
| `full` | Everything: agent, oauth, users, content, analytics, scheduler | Complete applications |

### Feature Hierarchy

```
full
└── api
    ├── core
    │   ├── systemprompt-traits
    │   ├── systemprompt-models
    │   ├── systemprompt-identifiers
    │   └── systemprompt-extension
    └── database
        └── systemprompt-core-database
```

---

## Usage Examples

### Extension Development

```toml
[dependencies]
systemprompt = { version = "0.1", features = ["api"] }
```

```rust
use systemprompt::prelude::*;

#[derive(Default)]
struct MyExtension;

impl Extension for MyExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "my-extension",
            name: "My Extension",
            version: "1.0.0",
        }
    }

    fn router(&self, _ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
        Some(ExtensionRouter::new(
            Router::new().route("/hello", get(|| async { "Hello!" })),
            "/api/v1/my-ext",
        ))
    }

    fn schemas(&self) -> Vec<SchemaDefinition> {
        vec![]
    }
}

register_extension!(MyExtension);
```

### MCP Server Development

```toml
[dependencies]
systemprompt = { version = "0.1", features = ["mcp"] }
```

```rust
use systemprompt::prelude::*;

#[mcp_server]
struct MyMcpServer;

#[mcp_tools]
impl MyMcpServer {
    #[tool(description = "Say hello")]
    async fn hello(&self, #[arg(description = "Name")] name: String) -> String {
        format!("Hello, {name}!")
    }
}
```

### Accessing Specific Modules

```rust
// Via prelude (recommended)
use systemprompt::prelude::*;

// Via specific modules
use systemprompt::models::Content;
use systemprompt::identifiers::ContentId;
use systemprompt::database::GenericRepository;
```

---

## Migration from Individual Crates

### Before (Multiple Dependencies)

```toml
[dependencies]
systemprompt-models = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-identifiers = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-traits = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-core-database = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-runtime = { git = "https://github.com/systempromptio/systemprompt-core" }
```

```rust
use systemprompt_models::Content;
use systemprompt_identifiers::ContentId;
use systemprompt_traits::Extension;
```

### After (Single Dependency)

```toml
[dependencies]
systemprompt = { version = "0.1", features = ["api"] }
```

```rust
use systemprompt::prelude::*;
// or
use systemprompt::models::Content;
use systemprompt::identifiers::ContentId;
```

---

## Benefits

| Aspect | Individual Crates | Umbrella Crate |
|--------|-------------------|----------------|
| **Dependencies** | 5+ lines in Cargo.toml | 1 line |
| **Version sync** | Must align manually | Automatic |
| **Imports** | Multiple `use` statements | Single prelude |
| **Discoverability** | Search multiple crates | One place |
| **Compile time** | Same | Same (features control inclusion) |

---

## Quick Reference

| Feature | Use Case |
|---------|----------|
| `core` | Minimal extensions, CLI tools |
| `database` | Extensions with database tables |
| `mcp` | MCP servers with proc macros |
| `api` | Full HTTP API extensions |
| `full` | Complete applications |
