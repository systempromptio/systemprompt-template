# Frequently Asked Questions

**Common questions about SystemPrompt Core architecture and distribution**

---

## Distribution & Installation

### Why git dependencies instead of crates.io?

SystemPrompt Core uses git dependencies because:

1. **Workspace Integrity**: All 14 crates stay together in one workspace with path dependencies
2. **Simple Publishing**: One `git tag` publishes all crates (vs 14 separate publications)
3. **Version Consistency**: Impossible to have version mismatches between core crates
4. **Framework Nature**: SystemPrompt is a platform/framework (like Rails), not a library
5. **Development Flexibility**: Easy to use branches, forks, and private repos

Git distribution is legitimate and used by projects like Substrate/Polkadot and Bevy (during development).

---

### Will you publish to crates.io eventually?

Maybe! We're considering it for v1.0 to improve discoverability and adoption. However, this would require:

- Converting ~70 path dependencies to version dependencies
- Publishing 14 crates individually in dependency order
- 30-60 minute publish process (vs 2 minutes with git)
- Loss of workspace structure advantages

For now, git distribution provides the best developer experience for a tightly-coupled framework.

---

### Can I download just one crate?

**No.** With git dependencies, Cargo always downloads the entire repository (~50MB).

**Why?** Internal crates use path dependencies (e.g., `path = "../../shared/models"`). These paths only exist within the full workspace, so Cargo must download everything.

**However**, Cargo only **builds** the crates you actually need:

```
Downloaded: All 14 crates + web/ + justfile + schema/ (~50MB)
Built: Only dependencies you use (~6-8 crates typically)
```

This is different from crates.io where you would download individual .crate files (~5MB total).

---

### Is this standard practice in Rust?

**No**, most Rust libraries publish to crates.io. **But** it's legitimate and used by:

- **Substrate/Polkadot** (blockchain framework with 100+ interdependent crates)
- **Bevy** (game engine, during development)
- **Corporate frameworks** (private, can't use public crates.io)
- **Pre-1.0 projects** (API not stable yet)

Git distribution is **standard for frameworks** before stable 1.0 release.

---

## Dependencies & Versions

### How do dependencies work with git repos?

```toml
# User adds:
systemprompt-core-api = { git = "...", tag = "v0.1.0" }

# Cargo:
1. Clones entire repository
2. Checks out tag v0.1.0
3. Finds systemprompt-core-api in workspace
4. Reads its dependencies: { path = "../../shared/models" }
5. Resolves paths within same checkout
6. Builds dependency tree
7. Compiles needed crates
```

All path dependencies resolve within the cached checkout, guaranteeing version consistency.

See [Dependency Guide](./DEPENDENCY_GUIDE.md) for technical details.

---

### Why do all crates have the same version?

All crates use **workspace inheritance**:

```toml
# Root Cargo.toml
[workspace.package]
version = "0.1.0"  # ← Single source of truth

# Each crate
[package]
version.workspace = true  # → Inherits "0.1.0"
```

This ensures:
- ✅ All crates from same git tag have same version
- ✅ No version conflicts possible
- ✅ Single version bump updates all 14 crates
- ✅ Clear compatibility guarantees

---

### What happens when I update to a new version?

```bash
# Edit Cargo.toml
# tag = "v0.1.0" → "v0.2.0"

cargo update
cargo build
```

**Cargo does**:
1. Checks out new tag in cache: `~/.cargo/git/checkouts/.../v0.2.0/`
2. Resolves all dependencies from v0.2.0 workspace
3. Rebuilds affected crates
4. Links new versions into your project

**Time**: 1-2 minutes (uses cached git repo)

---

### Do I need to import all 14 crates?

**No!** Import only what you need:

```toml
# Minimal setup
systemprompt-models = { git = "...", tag = "v0.1.0" }
systemprompt-core-database = { git = "...", tag = "v0.1.0" }

# Full stack
systemprompt-core-api = { git = "...", tag = "v0.1.0" }
systemprompt-core-mcp = { git = "...", tag = "v0.1.0" }
systemprompt-core-agent = { git = "...", tag = "v0.1.0" }
```

Cargo automatically downloads all crates but only builds what you use.

---

## Performance & Caching

### Why is first build so slow?

**First build** (2-5 minutes):
- Clones repository (~50MB, ~1-2 min)
- Resolves dependencies (~30 sec)
- Compiles core crates (~2-5 min)

**Subsequent builds** (<30 seconds):
- Uses Cargo cache (no download)
- Uses compiled artifacts (no rebuild)
- Only rebuilds changed code

**Tip**: Use `cargo check` (faster than `cargo build`) during development.

---

### Where does Cargo cache git dependencies?

```
~/.cargo/git/
├── db/                              # Bare git repos
│   └── systemprompt-core-<hash>/
└── checkouts/                       # Extracted code
    └── systemprompt-core-<hash>/
        ├── v0.1.0/                 # Each tag/branch cached
        ├── v0.2.0/
        └── main-<commit>/
```

Multiple versions can coexist. Cargo picks the right one based on your `Cargo.toml`.

---

### How do I clear the cache?

```bash
# Remove git checkouts
rm -rf ~/.cargo/git/checkouts/systemprompt-core-*

# Remove bare repo
rm -rf ~/.cargo/git/db/systemprompt-core-*

# Clean project
cargo clean

# Next build re-downloads
cargo build
```

---

## Development Workflow

### How do I use a development version?

```toml
# Use latest main branch (not a tag)
systemprompt-core-api = {
    git = "https://github.com/systempromptio/systemprompt-core",
    branch = "main"  # ← Latest code
}
```

**Warning**: `main` branch is unstable. Use tags for production.

---

### How do I test local changes to core?

**Method 1: Patch directive** (temporary override)

```toml
# .cargo/config.toml
[patch."https://github.com/systempromptio/systemprompt-core"]
systemprompt-core-api = { path = "/path/to/local/systemprompt-core/crates/modules/api" }
```

**Method 2: Fork and modify**

```toml
systemprompt-core-api = {
    git = "https://github.com/yourname/systemprompt-core",
    branch = "my-feature"
}
```

---

### Can I use different versions of different crates?

**No.** All core crates must use the same git tag:

```toml
# ❌ Wrong - different versions
systemprompt-models = { git = "...", tag = "v0.1.0" }
systemprompt-core-api = { git = "...", tag = "v0.2.0" }  # Different!

# ✅ Correct - same version
systemprompt-models = { git = "...", tag = "v0.1.0" }
systemprompt-core-api = { git = "...", tag = "v0.1.0" }  # Same!
```

This is enforced by the workspace structure - all crates share one version.

---

## Architecture

### What's the difference between shared/ and modules/?

```
crates/shared/        # Foundation libraries
├── traits/          # Core trait definitions
├── models/          # Data structures
└── identifiers/     # ID types

crates/modules/       # Platform modules
├── database/        # Database abstraction
├── api/             # HTTP server
├── mcp/             # MCP protocol
├── agent/           # Agent orchestration
└── ... (11 total)
```

**Shared crates**: Low-level building blocks used by modules
**Module crates**: Higher-level platform functionality

See [Architecture Guide](./ARCHITECTURE.md) for dependency graph.

---

### Can I use individual modules?

Yes! For example, just database + models:

```toml
systemprompt-models = { git = "...", tag = "v0.1.0" }
systemprompt-core-database = { git = "...", tag = "v0.1.0" }
```

However, you still download the entire workspace (but only build these crates).

---

### What's in the non-Rust directories?

```
web/        # React frontend for admin UI
justfile    # Build commands (like Makefile)
schema/     # Database migration files
docs/       # Documentation
tests/      # Integration tests
```

These are included in git downloads but not needed for Rust compilation.

---

## Comparison to Other Tools

### How is this different from npm/yarn workspaces?

Similar concepts:

| Aspect | Cargo Workspace | npm Workspace |
|--------|-----------------|---------------|
| **Monorepo** | ✅ Yes | ✅ Yes |
| **Shared version** | ✅ Yes (workspace.package) | ⚠️ Possible (lerna) |
| **Path deps** | ✅ Yes | ✅ Yes (workspace:*) |
| **Git distribution** | ✅ Yes | ✅ Yes |
| **Registry** | crates.io | npmjs.com |

SystemPrompt uses Cargo workspaces with git distribution (like monorepo npm projects before publishing to npmjs).

---

### How is this different from Git submodules?

| Feature | Git Dependencies | Git Submodules |
|---------|------------------|----------------|
| **Setup** | Add to Cargo.toml | `git submodule add` |
| **Updates** | `cargo update` | `git submodule update` |
| **Version** | Tag/branch in Cargo.toml | Locked commit SHA |
| **Management** | Cargo handles it | Manual git commands |
| **Cloning** | Automatic (Cargo) | Need `--recursive` |

Git dependencies are **much simpler** for Rust projects.

---

## Troubleshooting

### "failed to load manifest for dependency"

**Cause**: Git URL wrong or tag doesn't exist.

**Solution**:
```bash
# Verify tag exists
git ls-remote --tags https://github.com/systempromptio/systemprompt-core | grep v0.1.0

# Clear cache and retry
rm -rf ~/.cargo/git/checkouts/systemprompt-core-*
cargo build
```

---

### "multiple packages with same name"

**Cause**: Mixing git and path dependencies.

**Solution**: Use **only** git dependencies, not path dependencies to `core/` subtree:

```toml
# ❌ Wrong
systemprompt-models = { path = "core/crates/shared/models" }

# ✅ Correct
systemprompt-models = { git = "...", tag = "v0.1.0" }
```

---

### Build takes forever every time

**Cause**: Using `branch = "main"` (Cargo checks for updates).

**Solution**: Pin to a tag:

```toml
# ❌ Slow - checks for updates
systemprompt-core-api = { git = "...", branch = "main" }

# ✅ Fast - cached
systemprompt-core-api = { git = "...", tag = "v0.1.0" }
```

---

## Future Roadmap

### When will 1.0 be released?

No specific date yet. v1.0 will mark:
- Stable API (no more breaking changes)
- Production-ready
- Possible crates.io publishing

Subscribe to [GitHub releases](https://github.com/systempromptio/systemprompt-core/releases) for updates.

---

### Will the architecture change?

The core architecture (workspace with path dependencies) is stable. Possible future changes:

- **Dual publishing** (git + crates.io)
- **More modules** (additional platform features)
- **Better tooling** (automated updates, migration helpers)

The git distribution model will remain for development versions.

---

## Getting Help

### Where can I ask questions?

- **GitHub Discussions**: https://github.com/systempromptio/systemprompt-core/discussions
- **GitHub Issues**: https://github.com/systempromptio/systemprompt-core/issues
- **Documentation**: https://github.com/systempromptio/systemprompt-core/tree/main/docs

---

### How do I report bugs?

[Create an issue](https://github.com/systempromptio/systemprompt-core/issues/new) with:
- SystemPrompt Core version (git tag)
- Rust version (`rustc --version`)
- Steps to reproduce
- Expected vs actual behavior

---

### How do I contribute?

See [CONTRIBUTING.md](../CONTRIBUTING.md) (coming soon) or:

1. Fork repository
2. Create feature branch
3. Make changes with tests
4. Submit pull request

All contributions welcome!

---

## See Also

- [Architecture Guide](./ARCHITECTURE.md) - Workspace structure and design
- [Installation Guide](./INSTALLATION.md) - How to use SystemPrompt Core
- [Dependency Guide](./DEPENDENCY_GUIDE.md) - Technical dependency details
- [Publishing Guide](./PUBLISHING.md) - How to release new versions
