---
title: "Extension Architecture"
description: "Complete architecture guide for building extensions on systemprompt-core."
author: "SystemPrompt"
slug: "build-architecture"
keywords: "architecture, extensions, boundaries, layers"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Extension Architecture

Complete architecture guide for building extensions.

> **Reference Implementation**: See `extensions/web/` for a working example.

---

## Core Principle

**If it's Rust code, it's an extension. If it's YAML/Markdown, it's a service.**

| Category | Format | Location |
|----------|--------|----------|
| Extensions | `.rs` | `/extensions/` |
| Services | YAML/Markdown | `/services/` |

The `core/` directory is a git submodule. **Never modify it.**

---

## Project Structure

```
systemprompt-template/
├── core/                     # READ-ONLY submodule
├── extensions/               # ALL Rust code
│   ├── web/                 # Reference implementation
│   ├── cli/                 # CLI extensions
│   └── mcp/                 # MCP servers
│       └── systemprompt/    # MCP reference
├── services/                 # YAML/Markdown only
│   ├── agents/              # Agent definitions
│   ├── config/              # Configuration
│   ├── content/             # Markdown content
│   ├── playbook/            # Playbooks
│   ├── scheduler/           # Job schedules
│   ├── skills/              # Skills
│   └── web/                 # Theme config
├── profiles/                 # Environment configs
│   └── *.profile.yml        # Extension config
└── src/main.rs              # Server entry
```

---

## Layer Model

```
┌─────────────────────────────────────────────┐
│              src/main.rs                     │
│  Loads config, connects DB, mounts routers   │
└──────────────────────┬──────────────────────┘
                       │
┌──────────────────────▼──────────────────────┐
│              extensions/                     │
│  Schemas, Models, Repos, Services, API, Jobs │
└──────────────────────┬──────────────────────┘
                       │
┌──────────────────────▼──────────────────────┐
│              services/ (config)              │
│  Agent YAML, AI config, schedules, content   │
└──────────────────────┬──────────────────────┘
                       │ imports
┌──────────────────────▼──────────────────────┐
│              core/ (read-only)               │
│  Traits, Models, IDs, DB, Logging, Security  │
└─────────────────────────────────────────────┘
```

---

## Extension Internal Layers

```
┌─────────────────────────────┐
│     API (handlers)          │  HTTP requests
└─────────────┬───────────────┘
              │ calls
┌─────────────▼───────────────┐
│     Services                │  Business logic
└─────────────┬───────────────┘
              │ calls
┌─────────────▼───────────────┐
│     Repository              │  SQL queries
└─────────────┬───────────────┘
              │ uses
┌─────────────▼───────────────┐
│     Models                  │  Domain types
└─────────────────────────────┘
```

**Rules:**
- API → Services → Repository (never skip)
- No SQL in services
- No business logic in repositories
- Jobs use services, not direct repository access

---

## Dependency Rules

### Extensions CAN Import

```toml
systemprompt-models = { git = "..." }
systemprompt-identifiers = { git = "..." }
systemprompt-traits = { git = "..." }
systemprompt-core-database = { git = "..." }
systemprompt-blog-extension = { path = "../blog" }  # Other extensions
```

### Extensions CANNOT Import

```toml
systemprompt-core-api = { git = "..." }        # FORBIDDEN - entry layer
systemprompt-core-scheduler = { git = "..." }  # FORBIDDEN - app layer
```

---

## What Extensions CANNOT Do

| Forbidden | Alternative |
|-----------|-------------|
| Edit files in `core/` | Create extension |
| Access core tables directly | Use core services |
| `SELECT * FROM other_extension_table` | Call extension's service |
| `.rs` files in `services/` | Move to `extensions/` |
| MCP servers in `services/mcp/` | Move to `extensions/mcp/` |
| Inherent methods only | Implement `Extension` trait |

---

## Configuration Flow

**Services declare content. Extensions implement logic. Profiles configure both.**

```
profiles/*.profile.yml      → Extension config (validated at STARTUP)
        ↓
extensions/ (Rust)          → Raw → Validated type-state pattern
        ↓
services/ (Content)         → Markdown, job schedules
```

Config lives in profiles, not `services/config/`. Validate at startup, not runtime.

---

## Cross-Extension Communication

**Pattern 1: Service Import (Preferred)**
```rust
use systemprompt_blog_extension::ContentService;
let content = content_service.get_by_slug("post").await?;
```

**Pattern 2: Event-Driven**
```rust
event_bus.publish(ContentCreatedEvent { id }).await;
```

**Pattern 3: Shared IDs** — Store typed IDs in each extension's own tables.

---

## Boundary Validation

| Check | Command |
|-------|---------|
| Forbidden imports | `grep -E "systemprompt-core-(api\|scheduler)" Cargo.toml` |
| SQL in services | `grep -rn "sqlx::" src/services/` |

---

## Quick Reference

| Task | Command |
|------|---------|
| Build | `cargo build -p systemprompt-{name}-extension` |
| Test | `cargo test -p systemprompt-{name}-extension` |
| Lint | `cargo clippy -p systemprompt-{name}-extension -- -D warnings` |

## Reference Implementations

| Concept | Location |
|---------|----------|
| Extension trait | `extensions/web/src/extension.rs` |
| ExtensionError | `extensions/web/src/error.rs` |
| Repository | `extensions/web/src/repository/` |
| Service | `extensions/web/src/services/` |
| API | `extensions/web/src/api/` |
| Jobs | `extensions/web/src/jobs/` |
| MCP server | `extensions/mcp/systemprompt/` |