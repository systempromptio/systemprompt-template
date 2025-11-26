# SystemPrompt Core Architecture

**Understanding the workspace structure, module organization, and dependency management**

---

## Table of Contents

1. [Overview](#overview)
2. [Workspace Structure](#workspace-structure)
3. [The 14 Core Crates](#the-14-core-crates)
4. [Version Management](#version-management)
5. [Dependency Resolution](#dependency-resolution)
6. [Distribution Model](#distribution-model)

---

## Overview

SystemPrompt Core is a **monorepo workspace** containing 14 separate but interdependent Rust crates. All crates are versioned together and distributed as a single unit via Git.

### Key Principles

- **Single Version**: All 14 crates share the same version number
- **Workspace Inheritance**: Version, edition, and metadata inherited from root
- **Path Dependencies**: Internal crates use path-based dependencies
- **Git Distribution**: Published via Git tags (not crates.io)

---

## Workspace Structure

```
systemprompt-core/
├── Cargo.toml                    # Workspace root - defines shared version
├── src/main.rs                   # Reference CLI binary
├── crates/
│   ├── shared/                   # Foundation crates (3)
│   │   ├── traits/              # systemprompt-traits
│   │   ├── models/              # systemprompt-models
│   │   └── identifiers/         # systemprompt-identifiers
│   │
│   └── modules/                  # Platform modules (11)
│       ├── database/            # systemprompt-core-database
│       ├── config/              # systemprompt-core-config
│       ├── api/                 # systemprompt-core-api
│       ├── mcp/                 # systemprompt-core-mcp
│       ├── agent/               # systemprompt-core-agent
│       ├── oauth/               # systemprompt-core-oauth
│       ├── users/               # systemprompt-core-users
│       ├── log/                 # systemprompt-core-logging
│       ├── core/                # systemprompt-core-system
│       ├── ai/                  # systemprompt-core-ai
│       └── rag/                 # systemprompt-rag
│
├── web/                          # React frontend
├── justfile                      # Build commands
└── schema/                       # Database migrations
```

### Workspace Definition

```toml
# Root Cargo.toml
[workspace]
members = [
    "crates/shared/traits",
    "crates/shared/models",
    "crates/shared/identifiers",
    "crates/modules/core",
    "crates/modules/database",
    "crates/modules/config",
    "crates/modules/users",
    "crates/modules/oauth",
    "crates/modules/log",
    "crates/modules/agent",
    "crates/modules/api",
    "crates/modules/mcp",
    "crates/modules/ai",
    "crates/modules/rag",
]

[workspace.package]
version = "0.1.0"          # ← Single version for ALL crates
edition = "2021"
authors = ["SystemPrompt Team"]
license = "MIT"
```

---

## The 14 Core Crates

### Shared Foundation (3 crates)

| Crate | Purpose | Dependencies |
|-------|---------|--------------|
| `systemprompt-traits` | Trait definitions and interfaces | None |
| `systemprompt-identifiers` | ID types and generators | None |
| `systemprompt-models` | Data models and types | traits, identifiers, database, log |

### Platform Modules (11 crates)

| Crate | Purpose | Key Dependencies |
|-------|---------|------------------|
| `systemprompt-core-database` | Database abstraction (SQLite/PostgreSQL) | traits |
| `systemprompt-core-config` | Configuration management | traits |
| `systemprompt-core-logging` | Structured logging | database, models |
| `systemprompt-core-oauth` | OAuth2 authentication | database, models |
| `systemprompt-core-users` | User management | database, models, oauth, log |
| `systemprompt-core-system` | System core | All above |
| `systemprompt-core-agent` | Agent orchestration (A2A protocol) | database, models, log |
| `systemprompt-core-mcp` | MCP protocol implementation | database, models |
| `systemprompt-core-ai` | AI service integrations | models |
| `systemprompt-core-api` | HTTP API server | ALL modules |
| `systemprompt-rag` | RAG functionality | ai, models, database |

### Dependency Levels

```
Level 1 (No internal dependencies):
  └─ systemprompt-traits

Level 2 (Depends only on traits):
  ├─ systemprompt-identifiers
  └─ systemprompt-core-database

Level 3:
  ├─ systemprompt-models
  └─ systemprompt-core-config

Level 4:
  ├─ systemprompt-core-logging
  ├─ systemprompt-core-oauth
  └─ systemprompt-core-users

Level 5:
  ├─ systemprompt-core-system
  ├─ systemprompt-core-agent
  ├─ systemprompt-core-mcp
  └─ systemprompt-core-ai

Level 6 (Top level):
  ├─ systemprompt-core-api
  └─ systemprompt-rag
```

---

## Version Management

### Single Source of Truth

All crates inherit their version from the workspace:

```toml
# crates/modules/api/Cargo.toml
[package]
name = "systemprompt-core-api"
version.workspace = true    # ← Inherits "0.1.0" from root
edition.workspace = true
authors.workspace = true
license.workspace = true
```

### Version Bump Process

To release a new version, change **ONE** line:

```toml
# Root Cargo.toml
[workspace.package]
version = "0.1.0"  # Change to "0.2.0"
```

All 14 crates automatically become version 0.2.0.

### Git Tag = Version Snapshot

```bash
# Tag workspace version
git tag v0.2.0
git push origin v0.2.0

# Result: Immutable snapshot where ALL 14 crates = 0.2.0
```

---

## Dependency Resolution

### Internal Dependencies Use Paths

Crates within the workspace reference each other via **path dependencies**:

```toml
# crates/modules/api/Cargo.toml
[dependencies]
systemprompt-models = { path = "../../shared/models" }
systemprompt-core-database = { path = "../database" }
systemprompt-core-mcp = { path = "../mcp" }
```

**No version specified** - Cargo knows they're all the same version from workspace.

### How Path Resolution Works

```
1. api crate depends on: { path = "../../shared/models" }
2. Cargo resolves relative to: crates/modules/api/
3. Finds: crates/shared/models/Cargo.toml
4. Reads: version.workspace = true → resolves to 0.1.0
5. All crates from same workspace → guaranteed same version
```

### External Dependencies

External crates use workspace-shared dependencies:

```toml
# Root Cargo.toml
[workspace.dependencies]
tokio = { version = "1.47", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

```toml
# crates/modules/api/Cargo.toml
[dependencies]
tokio = { workspace = true }
serde = { workspace = true }
```

---

## Distribution Model

### Git Dependencies (Current Approach)

SystemPrompt Core is distributed via **Git repository**, not crates.io.

#### Why Git?

1. **Workspace Integrity**
   - All 14 crates downloaded together
   - Path dependencies work naturally
   - Guaranteed version consistency

2. **Simple Publishing**
   - One `git tag` publishes all crates
   - No need to publish 14 crates individually
   - 2 minutes vs 30-60 minutes

3. **Development Flexibility**
   - Can use branches for development
   - Can fork and customize
   - Private repository support

4. **Framework Nature**
   - SystemPrompt is a platform (like Rails)
   - Users typically deploy entire system
   - Not individual libraries

#### How Users Import

```toml
# Implementation Cargo.toml
[workspace.dependencies]
systemprompt-models = {
    git = "https://github.com/systempromptio/systemprompt-core",
    tag = "v0.1.0"
}
systemprompt-core-api = {
    git = "https://github.com/systempromptio/systemprompt-core",
    tag = "v0.1.0"
}
```

#### What Cargo Does

```
1. Clones entire repository → ~/.cargo/git/checkouts/.../v0.1.0/
2. Checks out tag v0.1.0 (immutable snapshot)
3. Resolves all path dependencies within checkout
4. Builds only needed crates (not all 14)
5. Caches for future builds
```

#### Download Characteristics

- **Size**: ~50MB (entire repository)
- **First build**: 2-5 minutes
- **Cached builds**: 30 seconds
- **What you get**: Workspace + all crates + web/ + justfile + schema/

### Future: Crates.io Consideration

Publishing to crates.io is **possible** but would require:

1. Converting ~70 path dependencies to version dependencies
2. Publishing 14 crates individually in dependency order
3. 30-60 minute publish process
4. Loss of workspace structure
5. No private repository support

**Current decision**: Stay with git distribution for simplicity and workspace integrity.

---

## Comparison: Monorepo vs Multi-repo

SystemPrompt Core uses a **monorepo** approach:

### Monorepo (Current)
```
One repository → 14 crates → One version → One tag
```

**Pros**:
- ✅ Atomic updates (all or nothing)
- ✅ Guaranteed compatibility
- ✅ Simple dependency management
- ✅ Easy to refactor across crates

**Cons**:
- ❌ Larger downloads
- ❌ All crates versioned together
- ❌ Can't update individual crates

### Multi-repo Alternative
```
14 repositories → 14 crates → 14 versions → 14 tags
```

**Pros**:
- ✅ Independent versioning
- ✅ Smaller downloads
- ✅ Selective updates

**Cons**:
- ❌ Version coordination nightmare
- ❌ Risk of incompatible combinations
- ❌ Complex dependency management
- ❌ 14× maintenance burden

**SystemPrompt's choice**: Monorepo for simplicity and reliability.

---

## Architecture Principles

### 1. Workspace Inheritance
All metadata (version, edition, authors, license) inherited from root.

### 2. Path Dependencies
Internal crates use paths, not versions.

### 3. Single Version
One version number for entire platform.

### 4. Git Distribution
Published via git tags, not crates.io.

### 5. Dependency Layering
Foundation → Platform → Services (clear hierarchy).

### 6. Module Ownership
Each module owns its schema, migrations, and business logic.

---

## See Also

- [Installation Guide](./INSTALLATION.md) - How to use SystemPrompt Core
- [Dependency Guide](./DEPENDENCY_GUIDE.md) - Deep dive on dependency resolution
- [Publishing Guide](./PUBLISHING.md) - How to release new versions
- [FAQ](./FAQ.md) - Common questions
