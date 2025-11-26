# Installation Guide

**How to use SystemPrompt Core in your project**

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Installation Methods](#installation-methods)
3. [Version Pinning](#version-pinning)
4. [First Build Expectations](#first-build-expectations)
5. [Troubleshooting](#troubleshooting)

---

## Quick Start

SystemPrompt Core is distributed via **Git** (not crates.io). Add it to your project's `Cargo.toml`:

```toml
[workspace.dependencies]
systemprompt-models = {
    git = "https://github.com/systempromptio/systemprompt-core",
    tag = "v0.1.0"
}
systemprompt-core-api = {
    git = "https://github.com/systempromptio/systemprompt-core",
    tag = "v0.1.0"
}
systemprompt-core-database = {
    git = "https://github.com/systempromptio/systemprompt-core",
    tag = "v0.1.0"
}
```

Then in your crate:

```toml
[dependencies]
systemprompt-models.workspace = true
systemprompt-core-api.workspace = true
systemprompt-core-database.workspace = true
```

Build your project:

```bash
cargo build
```

**First build will take 2-5 minutes** as Cargo clones the repository (~50MB). Subsequent builds are fast (<30 seconds) thanks to caching.

---

## Installation Methods

### Method 1: For Application Developers (Recommended)

Use SystemPrompt Core as git dependencies in your implementation.

**Step 1: Create or update workspace Cargo.toml**

```toml
[workspace]
members = [
    "crates/services/mcp/*",
    # Your service crates
]

[workspace.dependencies]
# Import only the core modules you need
systemprompt-models = {
    git = "https://github.com/systempromptio/systemprompt-core",
    tag = "v0.1.0"
}
systemprompt-core-database = {
    git = "https://github.com/systempromptio/systemprompt-core",
    tag = "v0.1.0"
}
systemprompt-core-mcp = {
    git = "https://github.com/systempromptio/systemprompt-core",
    tag = "v0.1.0"
}
systemprompt-core-api = {
    git = "https://github.com/systempromptio/systemprompt-core",
    tag = "v0.1.0"
}

# External dependencies (match core's versions)
tokio = { version = "1.47", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

**Step 2: Use in your service crates**

```toml
# crates/services/mcp/my-service/Cargo.toml
[package]
name = "my-service"
version = "0.1.0"

[dependencies]
systemprompt-models.workspace = true
systemprompt-core-mcp.workspace = true
tokio.workspace = true
serde.workspace = true
```

**Step 3: Build**

```bash
cargo build --workspace
```

---

### Method 2: For Core Contributors

Clone the repository to work on core itself:

```bash
# Clone repository
git clone https://github.com/systempromptio/systemprompt-core
cd systemprompt-core

# Build all modules
cargo build --workspace

# Run tests
cargo test --workspace

# Build specific crate
cargo build -p systemprompt-core-api

# Run CLI
cargo run --bin systemprompt -- --help
```

---

### Method 3: Hybrid Approach (Git Subtree)

For implementation repositories that want local access to core assets (web UI, justfile, schemas):

**Step 1: Add core as git subtree**

```bash
# In your implementation repo
git subtree add \
    --prefix=core \
    https://github.com/systempromptio/systemprompt-core \
    v0.1.0 \
    --squash
```

**Step 2: Add git dependencies to Cargo.toml**

```toml
[workspace.dependencies]
# Use git dependencies (NOT path to core/ subtree!)
systemprompt-models = {
    git = "https://github.com/systempromptio/systemprompt-core",
    tag = "v0.1.0"
}
```

**Step 3: Use core assets**

```bash
# Import core justfile
echo 'import? "core/justfile"' >> justfile

# Copy web UI for customization
cp -r core/web custom-web
```

**Step 4: Update core**

```bash
git subtree pull \
    --prefix=core \
    https://github.com/systempromptio/systemprompt-core \
    v0.2.0 \
    --squash
```

**Note**: The `core/` directory contains assets for reference. Rust dependencies still come from git (not path dependencies to `core/crates/`).

---

## Version Pinning

### Production: Use Tags (Recommended)

```toml
systemprompt-core-api = {
    git = "https://github.com/systempromptio/systemprompt-core",
    tag = "v0.1.0"  # ← Pinned to specific release
}
```

**Characteristics**:
- ✅ Immutable (tag never changes)
- ✅ Reproducible builds
- ✅ Aggressive caching (fast builds)
- ✅ Production-safe

**Available versions**: See [GitHub releases](https://github.com/systempromptio/systemprompt-core/releases)

### Development: Use Branch

```toml
systemprompt-core-api = {
    git = "https://github.com/systempromptio/systemprompt-core",
    branch = "main"  # ← Latest development code
}
```

**Characteristics**:
- ✅ Latest features
- ✅ Updates with `cargo update`
- ⚠️ Unstable (main branch changes)
- ⚠️ Slower (Cargo checks for updates)
- ❌ Not for production

### Testing: Use Commit

```toml
systemprompt-core-api = {
    git = "https://github.com/systempromptio/systemprompt-core",
    rev = "abc123def456"  # ← Specific commit
}
```

**Use cases**:
- Testing specific fixes
- Bisecting issues
- Temporary patches

---

## First Build Expectations

### What Gets Downloaded

When you first build with git dependencies, Cargo downloads the **entire repository**:

```
~/.cargo/git/checkouts/systemprompt-core-<hash>/v0.1.0/
├── Cargo.toml (workspace)
├── crates/ (all 14 crates)
├── web/ (React frontend)
├── justfile (build commands)
├── schema/ (database migrations)
├── docs/ (documentation)
└── tests/ (integration tests)
```

**Size**: ~50MB

**Why the entire repo?** Git dependencies always clone the full repository because internal crates use path dependencies that must resolve within the workspace.

### What Gets Built

Cargo only builds the crates you actually depend on:

```
If you depend on: systemprompt-core-api

Cargo builds:
  ✓ systemprompt-traits (dependency)
  ✓ systemprompt-identifiers (dependency)
  ✓ systemprompt-models (dependency)
  ✓ systemprompt-core-database (dependency)
  ✓ systemprompt-core-config (dependency)
  ✓ systemprompt-core-logging (dependency)
  ✓ systemprompt-core-api (requested)

Cargo DOES NOT build:
  ✗ systemprompt-core-mcp (not needed)
  ✗ systemprompt-core-agent (not needed)
  ✗ systemprompt-core-ai (not needed)
  ✗ systemprompt-rag (not needed)
```

### Build Times

| Build Type | Time | Notes |
|-----------|------|-------|
| First build (cold cache) | 2-5 min | Clones repo, builds all dependencies |
| Second build (warm cache) | 30 sec | Uses Cargo cache |
| After code change | 10 sec | Only rebuilds changed crates |
| After version update | 1-2 min | Checks out new version, rebuilds |

### Cache Location

Cargo caches git dependencies in:

```
~/.cargo/git/
├── db/                           # Bare git repositories
│   └── systemprompt-core-<hash>/
└── checkouts/                    # Actual code
    └── systemprompt-core-<hash>/
        ├── v0.1.0/              # Tag checkout
        ├── v0.2.0/              # Another tag
        └── main-<commit>/       # Branch checkout
```

To clear cache:
```bash
rm -rf ~/.cargo/git/checkouts/systemprompt-core-*
cargo clean
cargo build  # Re-downloads
```

---

## Available Modules

You don't need to import all 14 crates - only what you need:

### Foundation Crates

```toml
systemprompt-traits = { git = "...", tag = "v0.1.0" }
systemprompt-identifiers = { git = "...", tag = "v0.1.0" }
systemprompt-models = { git = "...", tag = "v0.1.0" }
```

### Platform Modules

```toml
systemprompt-core-database = { git = "...", tag = "v0.1.0" }
systemprompt-core-config = { git = "...", tag = "v0.1.0" }
systemprompt-core-logging = { git = "...", tag = "v0.1.0" }
systemprompt-core-oauth = { git = "...", tag = "v0.1.0" }
systemprompt-core-users = { git = "...", tag = "v0.1.0" }
systemprompt-core-system = { git = "...", tag = "v0.1.0" }
```

### Service Modules

```toml
systemprompt-core-agent = { git = "...", tag = "v0.1.0" }
systemprompt-core-mcp = { git = "...", tag = "v0.1.0" }
systemprompt-core-api = { git = "...", tag = "v0.1.0" }
systemprompt-core-ai = { git = "...", tag = "v0.1.0" }
systemprompt-rag = { git = "...", tag = "v0.1.0" }
```

**Tip**: Import only what you need. Cargo will automatically pull in dependencies.

---

## Troubleshooting

### "failed to load manifest for dependency"

**Cause**: Git repository not accessible or tag doesn't exist.

**Solution**:
```bash
# Check tag exists
git ls-remote --tags https://github.com/systempromptio/systemprompt-core | grep v0.1.0

# Check network access
curl -I https://github.com/systempromptio/systemprompt-core

# Clear cache and retry
rm -rf ~/.cargo/git/checkouts/systemprompt-core-*
cargo build
```

### "package requires rustc X.Y.Z"

**Cause**: Rust version mismatch.

**Solution**:
```bash
# Update Rust
rustup update

# Or use specific version
rustup override set 1.75.0
```

### Build takes forever

**Causes**:
1. Using `branch = "main"` (Cargo checks for updates)
2. Slow network
3. Building in release mode

**Solutions**:
```bash
# Switch to tag for faster builds
# Change: branch = "main"
# To:     tag = "v0.1.0"

# Use debug builds (default)
cargo build  # Not cargo build --release

# Enable sccache for faster rebuilds
cargo install sccache
export RUSTC_WRAPPER=sccache
```

### "multiple packages with same name"

**Cause**: Mixing git and path dependencies.

**Solution**: Ensure you're using git dependencies, NOT path dependencies to `core/` subtree:

```toml
# ❌ Wrong (if using git subtree)
systemprompt-models = { path = "core/crates/shared/models" }

# ✅ Correct
systemprompt-models = { git = "...", tag = "v0.1.0" }
```

---

## External Dependencies

SystemPrompt Core uses specific versions of external crates. Your implementation should **match these versions**:

```toml
# Match core's dependency versions
tokio = { version = "1.47", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
```

See [Cargo.toml](https://github.com/systempromptio/systemprompt-core/blob/main/Cargo.toml#L109-L193) for complete list.

---

## Updating Dependencies

To update to a new SystemPrompt Core version:

```bash
# Step 1: Update Cargo.toml tags
sed -i 's/tag = "v0.1.0"/tag = "v0.2.0"/' Cargo.toml

# Step 2: Update dependencies
cargo update

# Step 3: Rebuild
cargo build --workspace

# Step 4: Run tests
cargo test --workspace
```

Or use the automated update command (if using hybrid approach):
```bash
just update v0.2.0
```

---

## See Also

- [Architecture Guide](./ARCHITECTURE.md) - Understanding the workspace
- [Dependency Guide](./DEPENDENCY_GUIDE.md) - How dependencies resolve
- [Publishing Guide](./PUBLISHING.md) - Releasing new versions
- [FAQ](./FAQ.md) - Common questions
