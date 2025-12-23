# SystemPrompt Template

Build extensions on systemprompt-core. Write world-class idiomatic Rust.

## Critical Rules

1. **`core/` is READ-ONLY** — Never modify. It's a git submodule.
2. **Rust code → `extensions/`** — All `.rs` files live here, including MCP servers.
3. **Config only → `services/`** — YAML/Markdown only. No Rust code.
4. **Implement traits** — Use `Extension` + `ExtensionError`, not inherent methods.

## Architecture

```
extensions/                    # ALL Rust code
├── blog/                      # Reference implementation
└── mcp/                       # MCP servers (Rust crates)
    ├── admin/
    └── infrastructure/

services/                      # CONFIG ONLY (no .rs files)
├── agents/                    # Agent YAML
├── scheduler/                 # Job schedules (refs extension jobs)
├── skills/                    # Skill definitions
└── content/                   # Markdown
```

→ Full details: [instructions/learn/architecture.md](instructions/learn/architecture.md)

## Building Extensions

```rust
// 1. Implement Extension trait (not inherent methods)
impl Extension for MyExtension {
    fn metadata(&self) -> ExtensionMetadata { ... }
    fn schemas(&self) -> Vec<SchemaDefinition> { ... }
    fn router(&self, ctx: &ExtensionContext) -> Option<Router> { ... }
    fn jobs(&self) -> Vec<Arc<dyn Job>> { ... }
}
register_extension!(MyExtension);

// 2. Implement ExtensionError trait
impl ExtensionError for MyError {
    fn code(&self) -> &'static str { ... }
    fn status(&self) -> StatusCode { ... }
    fn is_retryable(&self) -> bool { ... }
}
```

→ Step-by-step: [instructions/learn/extension-guide.md](instructions/learn/extension-guide.md)

## Idiomatic Rust

**Combinators over control flow:**
```rust
opt.unwrap_or_else(|| default())      // not if let
opt.map(f)                            // not match Some/None
condition.then(|| x)                  // not if/else Some/None
items.iter().filter().map().collect() // not manual loops
```

**Mandatory patterns:**
```rust
// Typed IDs, never raw strings
use systemprompt_identifiers::{ContentId, UserId};

// COLUMNS constant for DRY SQL
impl Content {
    pub const COLUMNS: &'static str = r#"id as "id: ContentId", slug, title"#;
}

// Structured logging
tracing::info!(user_id = %id, "Created user");

// thiserror for domain errors
#[derive(Error, Debug)]
pub enum MyError { #[error("Not found")] NotFound }
```

**Forbidden:**
| ❌ | ✅ |
|---|---|
| `unwrap()` | `?`, `ok_or_else()`, `expect("msg")` |
| `unsafe` | Remove entirely |
| `// comments` | Self-documenting code |
| `sqlx::query()` | `sqlx::query!()` (compile-time) |
| SQL in services | SQL in repository only |
| `.rs` in `services/` | Move to `extensions/` |

→ Full standards: [instructions/build/rust-standards.md](instructions/build/rust-standards.md)

## Commands

```bash
just build              # Build all
just start              # Start server
just db-up && just db-migrate  # Database

cargo clippy -p {crate} -- -D warnings  # Lint
cargo fmt -p {crate} -- --check         # Format
```

## Checklists

| Task | Checklist |
|------|-----------|
| Build extension | [build/extension-checklist.md](instructions/build/extension-checklist.md) |
| Build MCP server | [build/mcp-server-checklist.md](instructions/build/mcp-server-checklist.md) |
| Review extension | [review/extension-review.md](instructions/review/extension-review.md) |
| Review MCP server | [review/mcp-server-review.md](instructions/review/mcp-server-review.md) |

## Quick Reference

| Topic | Location |
|-------|----------|
| Services ↔ Extensions relationship | [reference/services-extensions.md](instructions/reference/services-extensions.md) |
| What extensions can/cannot do | [reference/boundaries.md](instructions/reference/boundaries.md) |
| Directory structure conventions | [reference/file-structure.md](instructions/reference/file-structure.md) |
| Query logs | [reference/logs.md](instructions/reference/logs.md) |