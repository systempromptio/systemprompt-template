---
title: "Coding Standards"
description: "Rust coding standards for systemprompt.io - idiomatic patterns, mandatory requirements, and anti-patterns to avoid"
author: "systemprompt.io"
slug: "coding-standards"
keywords: "rust, coding standards, patterns, style guide, best practices"
image: "/files/images/docs/coding-standards.svg"
kind: "guide"
public: true
tags: ["getting-started", "coding", "rust", "standards"]
published_at: "2025-01-27"
updated_at: "2026-02-02"
after_reading_this:
  - "Write idiomatic Rust code that passes clippy and fmt"
  - "Use typed identifiers and SQLX macros correctly"
  - "Avoid forbidden constructs and anti-patterns"
  - "Structure extensions according to project conventions"
related_playbooks:
  - title: "Coding Standards Guide"
    url: "/playbooks/guide-coding-standards"
  - title: "Rust Standards"
    url: "/playbooks/build-rust-standards"
  - title: "Extension Checklist"
    url: "/playbooks/build-extension-checklist"
  - title: "MCP Server Checklist"
    url: "/playbooks/build-mcp-checklist"
  - title: "Architecture Overview"
    url: "/playbooks/build-architecture"
related_code:
  - title: "Web Extension Implementation"
    url: "https://github.com/systempromptio/systemprompt-template/blob/main/extensions/web/src/extension.rs"
  - title: "Extension Trait"
    url: "https://github.com/systempromptio/systemprompt-core/blob/main/crates/shared/traits/src/extension.rs"
  - title: "Typed Identifiers"
    url: "https://github.com/systempromptio/systemprompt-core/blob/main/crates/shared/identifiers/src/lib.rs"
related_docs:
  - title: "Getting Started"
    url: "/documentation/getting-started"
  - title: "Extensions Overview"
    url: "/documentation/extensions"
  - title: "Extension Trait"
    url: "/documentation/extensions/traits/extension-trait"
  - title: "Error Handling"
    url: "/documentation/extensions/internals/error-handling"
links:
  - title: "Rust Book"
    url: "https://doc.rust-lang.org/book/"
  - title: "Clippy Lints"
    url: "https://rust-lang.github.io/rust-clippy/master/"
---

# Coding Standards

**SystemPrompt is a world-class Rust programming brand.** Every Rust file in extensions and MCP servers must be instantly recognizable as on-brand, world-class idiomatic Rust. No exceptions.

## Core Principle

Write code that would pass Steve Klabnik's review. Prefer iterator chains, combinators, and pattern matching over imperative control flow.

## Validation

Run these commands after every change:

```bash
cargo clippy --workspace -- -D warnings
cargo fmt --all
cargo test --workspace
```

## Idiomatic Rust

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

### Anti-Patterns to Avoid

| Anti-Pattern | Idiomatic |
|--------------|-----------|
| `if let Some(x) = opt { x } else { default }` | `opt.unwrap_or(default)` |
| `match opt { Some(x) => Some(f(x)), None => None }` | `opt.map(f)` |
| `if condition { Some(x) } else { None }` | `condition.then(\|\| x)` |
| Manual loops building `Vec` | Iterator chains with `collect()` |

## File Limits

| Metric | Limit |
|--------|-------|
| Source file length | 300 lines |
| Cognitive complexity | 15 |
| Function length | 75 lines |
| Parameters | 5 |

## Forbidden Constructs

| Construct | Resolution |
|-----------|------------|
| `unsafe` | Remove - forbidden in this codebase |
| `unwrap()` | Use `?`, `ok_or_else()`, or `expect()` with message |
| `panic!()` / `todo!()` | Return `Result` or implement |
| Inline comments (`//`) | Delete - code documents itself |
| Doc comments (`///`) | Delete - no rustdoc |
| `println!` in libraries | Use `tracing` |
| Raw SQL strings | Use SQLX macros |

## Mandatory Patterns

### Typed Identifiers

All identifier fields use wrappers from `systemprompt_identifiers`:

```rust
use systemprompt_identifiers::{ContentId, UserId};

pub struct Content {
    pub id: ContentId,
    pub user_id: UserId,
}
```

Available types: `SessionId`, `UserId`, `AgentId`, `TaskId`, `ContextId`, `TraceId`, `ClientId`, `AgentName`, `AiToolCallId`, `McpExecutionId`, `SkillId`, `SourceId`, `CategoryId`, `ContentId`.

### Logging

All logging via `tracing`. No `println!` in library code.

```rust
use tracing::{info, error, debug, warn};

tracing::info!(user_id = %user.id, "Created user");
tracing::error!(error = %e, "Operation failed");
```

### Repository Pattern

Services never execute queries directly. All SQL in repositories using SQLX macros:

```rust
pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(User, "SELECT id, email, name FROM users WHERE email = $1", email)
        .fetch_optional(&**self.pool)
        .await
}
```

### SQLX Macros Only

| Allowed | Forbidden |
|---------|-----------|
| `sqlx::query!()` | `sqlx::query()` |
| `sqlx::query_as!()` | `sqlx::query_as()` |
| `sqlx::query_scalar!()` | `sqlx::query_scalar()` |

### Error Handling

Use `thiserror` for domain-specific errors. `anyhow` only at application boundaries:

```rust
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Item not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

### DateTime

| Layer | Type |
|-------|------|
| Rust | `DateTime<Utc>` |
| PostgreSQL | `TIMESTAMPTZ` |

Never use `NaiveDateTime` or `TIMESTAMP`.

### Builder Pattern

Required for types with 3+ fields or mixed required/optional fields:

```rust
pub struct CreateContentParams {
    pub slug: String,
    pub title: String,
    pub body: String,
    pub description: Option<String>,
}

impl CreateContentParams {
    pub fn builder(slug: impl Into<String>, title: impl Into<String>, body: impl Into<String>) -> CreateContentParamsBuilder {
        CreateContentParamsBuilder::new(slug, title, body)
    }
}

pub struct CreateContentParamsBuilder {
    slug: String,
    title: String,
    body: String,
    description: Option<String>,
}

impl CreateContentParamsBuilder {
    pub fn new(slug: impl Into<String>, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            slug: slug.into(),
            title: title.into(),
            body: body.into(),
            description: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn build(self) -> CreateContentParams {
        CreateContentParams {
            slug: self.slug,
            title: self.title,
            body: self.body,
            description: self.description,
        }
    }
}
```

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
| `is_` / `has_` | `bool` |

### Variables

| Type | Name |
|------|------|
| Database pool | `pool` |
| Repository | `repo` or `{noun}_repo` |
| Service | `service` or `{noun}_service` |

### Allowed Abbreviations

`id`, `uuid`, `url`, `jwt`, `mcp`, `a2a`, `api`, `http`, `json`, `sql`, `ctx`, `req`, `res`, `msg`, `err`, `cfg`

## Extension Structure

Every extension follows this structure:

```text
extensions/{name}/
├── Cargo.toml
├── schema/
│   └── 001_table.sql
└── src/
    ├── lib.rs
    ├── extension.rs    # Extension trait impl
    └── error.rs
```

Requirements:
- `Cargo.toml` with systemprompt dependencies
- `src/extension.rs` implements Extension trait
- `src/error.rs` implements ExtensionError trait
- Schema files numbered: `schema/001_name.sql`
- Registered in root `Cargo.toml` workspace members

## Derive Ordering

When deriving traits, use this order:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MyType { ... }
```

Order: `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Default`, `Serialize`, `Deserialize`

## Playbooks

For detailed operational guidance:

```bash
systemprompt core playbooks show guide_coding-standards
systemprompt core playbooks show build_rust-standards
systemprompt core playbooks show build_extension-checklist
```
