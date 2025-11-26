# Dependency Management Deep Dive

**Technical guide to how dependencies work in SystemPrompt Core**

---

## Table of Contents

1. [Overview](#overview)
2. [Workspace Inheritance Pattern](#workspace-inheritance-pattern)
3. [Path Dependencies](#path-dependencies)
4. [Git Dependencies Explained](#git-dependencies-explained)
5. [Cargo Cache Structure](#cargo-cache-structure)
6. [Comparison: Git vs Crates.io](#comparison-git-vs-cratesio)
7. [Dependency Resolution Flow](#dependency-resolution-flow)

---

## Overview

SystemPrompt Core uses a sophisticated dependency management strategy:

- **Workspace Inheritance**: Single version for all crates
- **Path Dependencies**: Internal crates reference via filesystem paths
- **Git Distribution**: Published as one repository, not individual crates
- **Cargo Resolution**: Automatic dependency tree building

---

## Workspace Inheritance Pattern

### The Root Definition

All metadata is defined once in the root `Cargo.toml`:

```toml
# Root: Cargo.toml
[workspace.package]
version = "0.1.0"          # ← Single source of truth
edition = "2021"
authors = ["SystemPrompt Team"]
license = "MIT"
repository = "https://github.com/systempromptio/systemprompt-core"
```

### Crate Inheritance

Each of the 14 crates inherits from workspace:

```toml
# crates/shared/models/Cargo.toml
[package]
name = "systemprompt-models"
version.workspace = true    # → Resolves to "0.1.0"
edition.workspace = true    # → Resolves to "2021"
authors.workspace = true    # → Resolves to ["SystemPrompt Team"]
license.workspace = true    # → Resolves to "MIT"
```

### Why This Works

When Cargo builds, it:

1. Reads root `Cargo.toml`
2. Sees `[workspace.package] version = "0.1.0"`
3. For each crate with `version.workspace = true`:
   - Substitutes with `version = "0.1.0"`
4. Result: All crates have identical version

### Changing Versions

To bump version for all 14 crates:

```bash
# Edit ONE line
vim Cargo.toml
# [workspace.package]
# version = "0.1.0" → "0.2.0"

# ALL crates now version 0.2.0
cargo metadata | jq '.packages[] | select(.name | startswith("systemprompt")) | .version'
# "0.2.0"
# "0.2.0"
# "0.2.0"
# ... (14 times)
```

---

## Path Dependencies

### How They're Defined

Internal crates use **relative paths** to reference each other:

```toml
# crates/modules/api/Cargo.toml
[dependencies]
# Path relative to THIS crate's directory
systemprompt-models = { path = "../../shared/models" }
#                              ^^^^^^^^^^^^^^^^^^^^
#                              From: crates/modules/api/
#                              To:   crates/shared/models/
```

### Resolution Process

```
1. Cargo reads: crates/modules/api/Cargo.toml
2. Sees dependency: { path = "../../shared/models" }
3. Resolves path:
   - Start: crates/modules/api/
   - Up two: crates/
   - Down: crates/shared/models/
4. Finds: crates/shared/models/Cargo.toml
5. Reads package:
   [package]
   name = "systemprompt-models"
   version.workspace = true  # → 0.1.0
6. Builds systemprompt-models
7. Links into api crate
```

### Why No Version Specification?

Path dependencies in a workspace **don't need version numbers**:

```toml
# ❌ Not needed (and would be ignored)
systemprompt-models = { path = "../../shared/models", version = "0.1.0" }

# ✅ Correct
systemprompt-models = { path = "../../shared/models" }
```

Because:
1. Both crates in same workspace
2. Workspace defines single version
3. Cargo knows they're the same version
4. No version conflict possible

---

## Git Dependencies Explained

### What Happens When You Add a Git Dependency

User adds to their `Cargo.toml`:

```toml
[dependencies]
systemprompt-core-api = {
    git = "https://github.com/systempromptio/systemprompt-core",
    tag = "v0.1.0"
}
```

### Step-by-Step Cargo Process

```
Step 1: Check Cache
├─ Look in: ~/.cargo/git/db/systemprompt-core-<hash>/
├─ If not found: git clone <url>
└─ If found: git fetch (if needed)

Step 2: Checkout Version
├─ Extract tag: v0.1.0
├─ Checkout to: ~/.cargo/git/checkouts/systemprompt-core-<hash>/v0.1.0/
└─ Result: Full workspace at v0.1.0

Step 3: Read Workspace
├─ Read: v0.1.0/Cargo.toml
├─ Parse: [workspace.package] version = "0.1.0"
├─ Find requested crate: crates/modules/api/
└─ Read: crates/modules/api/Cargo.toml

Step 4: Resolve Dependencies
├─ api depends on: systemprompt-models { path = "../../shared/models" }
├─ Resolve path within checkout: v0.1.0/crates/shared/models/
├─ models depends on: systemprompt-traits { path = "../traits" }
├─ Resolve: v0.1.0/crates/shared/traits/
└─ Continue recursively...

Step 5: Build Dependency Tree
├─ systemprompt-traits (0.1.0)
├─ systemprompt-identifiers (0.1.0)
├─ systemprompt-models (0.1.0) → depends on traits
├─ systemprompt-core-database (0.1.0) → depends on traits
├─ systemprompt-core-logging (0.1.0) → depends on database
└─ systemprompt-core-api (0.1.0) → depends on all above

Step 6: Build in Order
└─ Build from bottom up (dependencies first)
```

### Key Insight: Entire Workspace Downloaded

**You CANNOT download just one crate with git dependencies.**

Even if you only want `systemprompt-core-api`, Cargo downloads:

```
✓ All 14 crates
✓ web/ directory
✓ justfile
✓ schema/
✓ docs/
✓ tests/
✓ Everything in the repository (~50MB)
```

**But only BUILDS what's needed** (typically 6-8 crates).

---

## Cargo Cache Structure

### Directory Layout

```
~/.cargo/git/
│
├── db/                                        # Bare git repositories
│   └── systemprompt-core-1a2b3c4d/
│       ├── objects/                          # Git objects
│       ├── refs/                             # Git references
│       └── config                            # Git config
│
└── checkouts/                                 # Extracted source code
    └── systemprompt-core-1a2b3c4d/
        │
        ├── v0.1.0/                           # Tag checkout
        │   ├── Cargo.toml                    # Workspace root
        │   ├── crates/
        │   │   ├── shared/
        │   │   │   ├── models/               # All crates available
        │   │   │   ├── traits/
        │   │   │   └── identifiers/
        │   │   └── modules/
        │   │       ├── database/
        │   │       ├── api/
        │   │       └── ... (11 more)
        │   ├── web/
        │   ├── justfile
        │   └── schema/
        │
        ├── v0.2.0/                           # Different version
        │   └── ... (full workspace again)
        │
        └── main-abc123/                      # Branch checkout
            └── ... (full workspace)
```

### Cache Behavior

**First Build**:
```bash
cargo build
# → Clones entire repo to db/ (~5 minutes)
# → Extracts to checkouts/v0.1.0/ (~30 seconds)
# → Builds dependencies (~2-5 minutes)
# Total: 5-10 minutes
```

**Second Build (same version)**:
```bash
cargo build
# → Uses cached checkout (instant)
# → Uses cached binaries (instant)
# Total: 30 seconds
```

**After Version Change**:
```bash
# Edit: tag = "v0.1.0" → "v0.2.0"
cargo update
cargo build
# → Fetches new commits (if needed)
# → Extracts to checkouts/v0.2.0/
# → Rebuilds all crates
# Total: 2-3 minutes
```

### Clearing Cache

```bash
# Remove all checkouts
rm -rf ~/.cargo/git/checkouts/systemprompt-core-*

# Remove bare repository
rm -rf ~/.cargo/git/db/systemprompt-core-*

# Clean project
cargo clean

# Next build will re-download everything
cargo build
```

---

## Comparison: Git vs Crates.io

### Git Dependencies (Current Approach)

**What You Publish**:
```bash
git tag v0.1.0
git push origin v0.1.0
```

**What Users Get**:
```
ONE repository download containing:
├── All 14 crates (with path dependencies)
├── Workspace structure preserved
├── web/, justfile, schema/ included
└── ~50MB total
```

**Cargo Behavior**:
```
1. Clone entire repository
2. Checkout specific tag
3. Resolve path dependencies within checkout
4. Build dependency tree
5. Compile needed crates
```

**Storage**:
```
~/.cargo/git/checkouts/systemprompt-core-<hash>/v0.1.0/
```

### Crates.io Alternative (Not Used)

**What You Would Publish**:
```bash
# Publish 14 separate crates
cd crates/shared/traits && cargo publish
sleep 60
cd ../../shared/models && cargo publish
sleep 60
# ... (repeat 12 more times)
```

**What Users Would Get**:
```
14 SEPARATE crate downloads:
├── systemprompt-traits-0.1.0.crate (~500KB)
├── systemprompt-models-0.1.0.crate (~1MB)
├── systemprompt-core-api-0.1.0.crate (~800KB)
└── ... (14 total, ~5-10MB)
```

**Cargo Behavior**:
```
1. Fetch metadata from crates.io
2. Download ONLY requested crate
3. Read dependencies from metadata
4. Download each dependency (recursively)
5. Compile in dependency order
```

**Storage**:
```
~/.cargo/registry/src/github.com-<hash>/
├── systemprompt-traits-0.1.0/
├── systemprompt-models-0.1.0/
├── systemprompt-core-api-0.1.0/
└── ... (14 separate directories)
```

### Key Differences

| Aspect | Git Dependencies | Crates.io |
|--------|------------------|-----------|
| **Download** | Entire workspace (~50MB) | Individual crates (~5MB) |
| **Structure** | Workspace preserved | Flat structure |
| **Dependencies** | Path-based | Version-based |
| **Storage** | `~/.cargo/git/` | `~/.cargo/registry/` |
| **Publishing** | One git tag (2 min) | 14 publications (30-60 min) |
| **Versioning** | Workspace inheritance | Per-crate versions |
| **Can download one crate?** | No (always full repo) | Yes (selective) |
| **Discoverability** | Must know GitHub URL | Searchable on crates.io |
| **Documentation** | Manual (GitHub) | Auto (docs.rs) |
| **Private repos** | ✅ Supported | ❌ Public only (or paid) |
| **Dev versions** | `branch = "main"` | Publish pre-release |

---

## Dependency Resolution Flow

### Complete Example

User adds dependency:

```toml
[dependencies]
systemprompt-core-users = {
    git = "https://github.com/systempromptio/systemprompt-core",
    tag = "v0.1.0"
}
```

### Resolution Tree

```
systemprompt-core-users (v0.1.0)
├── systemprompt-models (v0.1.0)
│   ├── systemprompt-traits (v0.1.0)
│   ├── systemprompt-identifiers (v0.1.0)
│   ├── systemprompt-core-database (v0.1.0)
│   │   └── systemprompt-traits (v0.1.0) [duplicate, shared]
│   └── systemprompt-core-logging (v0.1.0)
│       └── systemprompt-core-database (v0.1.0) [duplicate, shared]
├── systemprompt-core-database (v0.1.0) [duplicate, shared]
├── systemprompt-core-oauth (v0.1.0)
│   ├── systemprompt-models (v0.1.0) [duplicate, shared]
│   └── systemprompt-core-database (v0.1.0) [duplicate, shared]
└── systemprompt-core-logging (v0.1.0) [duplicate, shared]
```

### Deduplication

Cargo automatically deduplicates:

```
Unique crates to build:
1. systemprompt-traits (v0.1.0)
2. systemprompt-identifiers (v0.1.0)
3. systemprompt-core-database (v0.1.0)
4. systemprompt-core-logging (v0.1.0)
5. systemprompt-models (v0.1.0)
6. systemprompt-core-oauth (v0.1.0)
7. systemprompt-core-users (v0.1.0)

Total: 7 crates (not 20+ with duplicates)
```

### Build Order

```
Level 1 (no dependencies):
  1. systemprompt-traits

Level 2 (depend on level 1):
  2. systemprompt-identifiers
  3. systemprompt-core-database

Level 3 (depend on level 2):
  4. systemprompt-core-logging
  5. systemprompt-models

Level 4 (depend on level 3):
  6. systemprompt-core-oauth

Level 5 (depend on level 4):
  7. systemprompt-core-users
```

---

## Version Consistency Guarantee

### The Problem with Version Deps

If crates were on crates.io with version dependencies:

```toml
# systemprompt-core-api/Cargo.toml
[dependencies]
systemprompt-models = "0.1.0"
systemprompt-core-database = "0.2.0"  # ← Different version!
```

**Risk**: `models` and `database` from different versions might be incompatible.

### The Solution with Git + Workspace

With git dependencies + workspace inheritance:

```
1. User specifies: tag = "v0.1.0"
2. Cargo clones: entire workspace at v0.1.0
3. ALL crates resolve to: version = "0.1.0" (from workspace)
4. Path dependencies resolve: within same checkout
5. Impossible to have version mismatch!
```

**Guarantee**: All core crates always from the same commit.

---

## External Dependencies

Core crates also depend on external crates (tokio, serde, etc.):

```toml
# Root Cargo.toml
[workspace.dependencies]
tokio = { version = "1.47", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

```toml
# crates/modules/api/Cargo.toml
[dependencies]
tokio = { workspace = true }  # → Resolves to 1.47
serde = { workspace = true }  # → Resolves to 1.0
```

**Implementation repos should match these versions** to avoid conflicts.

---

## See Also

- [Architecture Guide](./ARCHITECTURE.md) - Workspace structure
- [Installation Guide](./INSTALLATION.md) - How to add dependencies
- [Publishing Guide](./PUBLISHING.md) - Releasing new versions
- [FAQ](./FAQ.md) - Common questions
