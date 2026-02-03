---
title: "Coding Standards Guide"
description: "Principal guide for all coding standards. Links to language-specific standards and patterns."
type: guide
keywords:
  - coding
  - standards
  - rust
  - patterns
  - style
  - code
category: guide
playbook_references:
  - id: "build_rust-standards"
    description: "Rust coding standards"
  - id: "build_architecture"
    description: "System architecture"
  - id: "build_extension-checklist"
    description: "Extension building checklist"
  - id: "build_mcp-checklist"
    description: "MCP server requirements"
  - id: "build_mcp-tutorial"
    description: "MCP server tutorial"
  - id: "build_mcp-tools"
    description: "MCP tool patterns"
  - id: "build_mcp-artifacts"
    description: "MCP artifacts and resources"
  - id: "guide_playbook"
    description: "Playbook authoring standards"
---

# Coding Standards Guide

Principal guide for all code written in SystemPrompt. Follow these standards without exception.

> **Read playbooks**: `systemprompt core playbooks show <playbook_id>`

---

## Core Principle

**SystemPrompt is a world-class Rust programming brand.** Every file must be instantly recognizable as on-brand, world-class idiomatic code. No exceptions. No shortcuts. No compromise.

---

## Language Standards

### Rust

**Primary language for all extensions, MCP servers, and core logic.**

-> See [Rust Standards](../build/rust-standards.md) for complete standards

Key points:
- Write idiomatic Rust (Steve Klabnik style)
- Use typed identifiers from `systemprompt_identifiers`
- SQLX macros only (`query!`, `query_as!`, `query_scalar!`)
- Repository pattern for all database access
- Zero tolerance for `unsafe`, `unwrap()`, inline comments

Validation:
```bash
cargo clippy --workspace -- -D warnings
cargo fmt --all
```

### YAML/Markdown

**Configuration and documentation in `services/`.**

-> See [Playbook Authoring](playbook.md) for playbook standards
-> See [Documentation Guide](documentation.md) for doc standards

---

## Code Locations

| Type | Location | Language |
|------|----------|----------|
| Extensions | `extensions/*/src/` | Rust |
| MCP Servers | `extensions/mcp/*/src/` | Rust |
| Agents | `services/agents/*.yaml` | YAML |
| Skills | `services/skills/**/*.yaml` | YAML |
| Playbooks | `services/playbook/**/*.md` | Markdown |
| Config | `services/config/*.yaml` | YAML |

---

## Build Standards

### Extensions

-> See [Extension Checklist](../build/extension-checklist.md)

Every extension requires:
- `Cargo.toml` with systemprompt dependencies
- `src/extension.rs` implementing Extension trait
- `src/error.rs` implementing ExtensionError trait
- Schema files in `schema/` numbered `001_*.sql`

### MCP Servers

-> See [MCP Checklist](../build/mcp-checklist.md) — Full requirements
-> See [MCP Tutorial](../build/mcp-tutorial.md) — Step-by-step guide
-> See [MCP Tool Patterns](../build/mcp-tools.md) — Handler and schema patterns
-> See [MCP Artifacts](../build/mcp-artifacts.md) — Artifact storage and UI resources

Every MCP server requires:
- Tool definitions with JSON schemas
- Error handling with domain-specific errors
- Structured logging via `tracing`

---

## Mandatory Patterns

### Error Handling

Use `thiserror` for domain-specific errors. `anyhow` only at application boundaries.

```rust
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Item not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

### Logging

All logging via `tracing`. No `println!` in library code.

```rust
tracing::info!(user_id = %user.id, "Created user");
tracing::error!(error = %e, "Operation failed");
```

### Builder Pattern

Required for types with 3+ fields or mixed required/optional fields.

-> See [Rust Standards](../build/rust-standards.md) for full pattern

---

## Forbidden Constructs

| Construct | Resolution |
|-----------|------------|
| `unsafe` | Remove - forbidden |
| `unwrap()` | Use `?`, `ok_or_else()`, or `expect()` |
| `panic!()` / `todo!()` | Return `Result` or implement |
| Inline comments (`//`) | Delete - code documents itself |
| Doc comments (`///`) | Delete - no rustdoc |
| `println!` in libraries | Use `tracing` |
| Raw SQL strings | Use SQLX macros |

---

## File Limits

| Metric | Limit |
|--------|-------|
| Source file length | 300 lines |
| Cognitive complexity | 15 |
| Function length | 75 lines |
| Parameters | 5 |

---

## Naming Conventions

### Functions

| Prefix | Returns |
|--------|---------|
| `get_` | `Result<T>` - fails if missing |
| `find_` | `Result<Option<T>>` - may not exist |
| `list_` | `Result<Vec<T>>` |
| `create_` | `Result<T>` or `Result<Id>` |
| `update_` | `Result<T>` or `Result<()>` |
| `delete_` | `Result<()>` |

### Allowed Abbreviations

`id`, `uuid`, `url`, `jwt`, `mcp`, `a2a`, `api`, `http`, `json`, `sql`, `ctx`, `req`, `res`, `msg`, `err`, `cfg`

---

## Validation Workflow

Before committing any code:

1. Run linters:
```bash
cargo clippy --workspace -- -D warnings
cargo fmt --all
```

2. Run tests:
```bash
cargo test --workspace
```

3. Verify build:
```bash
just build
```

---

## Quick Reference

| Task | Playbook |
|------|----------|
| Rust coding standards | `build_rust-standards` |
| System architecture | `build_architecture` |
| Build extension | `build_extension-checklist` |
| Build MCP server | `build_mcp-checklist` |
| Write playbooks | `guide_playbook` |
| Write documentation | `guide_documentation` |

---

## Related

-> See [Rust Standards](../build/rust-standards.md) for complete Rust patterns
-> See [Architecture](../build/architecture.md) for system design
-> See [Extension Checklist](../build/extension-checklist.md) for building extensions
-> See [Playbook Authoring](playbook.md) for writing playbooks
