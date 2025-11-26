# Publishing Guide

**How to release new versions of SystemPrompt Core**

---

## Table of Contents

1. [Overview](#overview)
2. [Release Process](#release-process)
3. [Version Bumping](#version-bumping)
4. [Creating Releases](#creating-releases)
5. [Breaking Changes](#breaking-changes)
6. [Changelog Management](#changelog-management)

---

## Overview

SystemPrompt Core uses **git tags** for versioning and distribution. All 14 crates are versioned together and published simultaneously.

### Key Principles

- **Semantic Versioning**: Follow [semver.org](https://semver.org/)
- **Single Version**: All crates share the same version
- **Git Tags**: Versions are immutable git tags
- **Simple Process**: One tag publishes all crates

---

## Release Process

### Quick Release Checklist

```bash
# 1. Make changes
vim crates/modules/api/src/lib.rs

# 2. Run tests
cargo test --workspace

# 3. Update version
vim Cargo.toml
# [workspace.package]
# version = "0.1.0" → "0.2.0"

# 4. Update CHANGELOG.md
vim CHANGELOG.md

# 5. Commit changes
git add .
git commit -m "chore: bump version to 0.2.0"

# 6. Create tag
git tag -a v0.2.0 -m "Release v0.2.0"

# 7. Push tag
git push origin v0.2.0

# Done! All 14 crates now published at v0.2.0
```

**Time**: ~2 minutes

---

## Version Bumping

### Understanding Semantic Versioning

SystemPrompt Core follows [Semantic Versioning 2.0.0](https://semver.org/):

```
MAJOR.MINOR.PATCH
  |     |     |
  |     |     └─ Bug fixes (backward compatible)
  |     └─────── New features (backward compatible)
  └───────────── Breaking changes (not backward compatible)
```

### Examples

| Version | Type | Description |
|---------|------|-------------|
| 0.1.0 → 0.1.1 | Patch | Fix bug in database connection pooling |
| 0.1.1 → 0.2.0 | Minor | Add new `systemprompt-rag` module |
| 0.2.0 → 1.0.0 | Major | Remove deprecated APIs, stable release |
| 1.0.0 → 2.0.0 | Major | Redesign authentication system |

### How to Bump Version

**Edit ONE line in root Cargo.toml**:

```toml
# Before
[workspace.package]
version = "0.1.0"

# After (patch release)
[workspace.package]
version = "0.1.1"

# After (minor release)
[workspace.package]
version = "0.2.0"

# After (major release)
[workspace.package]
version = "1.0.0"
```

**All 14 crates automatically inherit the new version.**

### Verify Version Update

```bash
# Check all crates show new version
cargo metadata --no-deps --format-version 1 | \
  jq '.packages[] | select(.name | startswith("systemprompt")) | {name, version}'

# Output:
# {"name": "systemprompt-models", "version": "0.2.0"}
# {"name": "systemprompt-core-api", "version": "0.2.0"}
# ... (14 total)
```

---

## Creating Releases

### Step-by-Step Release

#### Step 1: Ensure Clean State

```bash
# Check for uncommitted changes
git status

# Should show: "nothing to commit, working tree clean"

# Run tests
cargo test --workspace

# Run clippy
cargo clippy --workspace
```

#### Step 2: Update Version

```bash
vim Cargo.toml
# [workspace.package]
# version = "0.1.0" → "0.2.0"
```

#### Step 3: Update CHANGELOG.md

```bash
vim CHANGELOG.md
```

Add entry:

```markdown
## [0.2.0] - 2025-01-15

### Added
- New RAG module (`systemprompt-rag`)
- PostgreSQL support in database module
- JWT authentication in OAuth module

### Changed
- Improved error handling in API module
- Updated MCP protocol to version 1.2

### Fixed
- Connection pool leak in database module
- Race condition in agent orchestration

### Breaking Changes
None
```

#### Step 4: Commit Changes

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 0.2.0"
```

#### Step 5: Create Annotated Tag

```bash
git tag -a v0.2.0 -m "Release v0.2.0

New Features:
- RAG module for retrieval-augmented generation
- PostgreSQL support
- JWT authentication

See CHANGELOG.md for full details."
```

**Why annotated tags?**
- Include release notes
- Cryptographically signed (optional)
- Show in `git describe`
- Better for releases

#### Step 6: Push Tag

```bash
# Push commit
git push origin main

# Push tag
git push origin v0.2.0
```

#### Step 7: Create GitHub Release (Optional but Recommended)

```bash
# Using GitHub CLI
gh release create v0.2.0 \
    --title "SystemPrompt Core v0.2.0" \
    --notes-file CHANGELOG.md

# Or manually:
# Go to: https://github.com/systempromptio/systemprompt-core/releases/new
# - Tag: v0.2.0
# - Title: "SystemPrompt Core v0.2.0"
# - Description: Paste from CHANGELOG.md
```

---

## Breaking Changes

### What Constitutes a Breaking Change?

A breaking change requires a **major version bump** (e.g., 1.2.3 → 2.0.0):

**API Changes**:
- Removing public functions/types
- Renaming public items
- Changing function signatures
- Removing trait implementations

**Behavior Changes**:
- Changing default behavior
- Modifying error types
- Altering database schema (without migration)

**Dependency Changes**:
- Updating to incompatible external versions
- Removing optional features

### How to Handle Breaking Changes

#### 1. Document in CHANGELOG.md

```markdown
## [2.0.0] - 2025-02-01

### Breaking Changes

- **API**: Renamed `ServiceConfig` to `ServiceDefinition`
  - **Migration**: Find/replace `ServiceConfig` with `ServiceDefinition`

- **Database**: `Database::query()` now returns `Result<Vec<Row>>`
  - **Migration**: Add `.await?` to all `query()` calls

- **Users**: Removed deprecated `User::get_by_email_sync()`
  - **Migration**: Use `User::get_by_email().await` instead
```

#### 2. Provide Migration Guide

Create `docs/migrations/v1-to-v2.md`:

```markdown
# Migrating from v1.x to v2.0

## ServiceConfig → ServiceDefinition

**Before**:
```rust
let config = ServiceConfig::new();
```

**After**:
```rust
let config = ServiceDefinition::new();
```

## Database Query Changes

**Before**:
```rust
let rows = db.query(sql);
```

**After**:
```rust
let rows = db.query(sql).await?;
```
```

#### 3. Deprecation Period (For Minor Versions)

Before breaking in v2.0, deprecate in v1.x:

```rust
// v1.5.0 - Deprecate old API
#[deprecated(since = "1.5.0", note = "Use `new_function()` instead")]
pub fn old_function() {
    // ...
}

pub fn new_function() {
    // New implementation
}

// v2.0.0 - Remove deprecated API
// old_function() removed entirely
```

---

## Changelog Management

### CHANGELOG.md Structure

```markdown
# Changelog

All notable changes to SystemPrompt Core will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- (New features not yet released)

### Changed
- (Changes in existing functionality)

### Fixed
- (Bug fixes)

## [0.2.0] - 2025-01-15

### Added
- New RAG module for retrieval-augmented generation
- PostgreSQL support in database module
- JWT authentication in OAuth module

### Changed
- Improved error handling in API module
- Updated MCP protocol to version 1.2

### Fixed
- Connection pool leak in database module
- Race condition in agent orchestration

### Breaking Changes
None

## [0.1.0] - 2025-01-01

Initial release.

### Added
- Core platform modules (database, config, api, mcp, agent)
- Shared models and traits
- SQLite support
- Basic authentication
- Agent orchestration via A2A protocol
```

### Categories

- **Added**: New features
- **Changed**: Changes to existing features
- **Deprecated**: Soon-to-be removed features
- **Removed**: Removed features (breaking!)
- **Fixed**: Bug fixes
- **Security**: Security fixes
- **Breaking Changes**: Special section for major versions

---

## Release Automation

### Automated Script (Future)

```bash
#!/bin/bash
# scripts/release.sh

set -e

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: ./scripts/release.sh v0.2.0"
    exit 1
fi

# Extract version without 'v' prefix
VERSION_NUM=${VERSION#v}

echo "Releasing version $VERSION..."

# 1. Run tests
echo "Running tests..."
cargo test --workspace --all-features

# 2. Run clippy
echo "Running clippy..."
cargo clippy --workspace --all-features -- -D warnings

# 3. Update version
echo "Updating version to $VERSION_NUM..."
sed -i "s/^version = .*/version = \"$VERSION_NUM\"/" Cargo.toml

# 4. Commit
echo "Committing changes..."
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to $VERSION_NUM"

# 5. Tag
echo "Creating tag $VERSION..."
git tag -a $VERSION -m "Release $VERSION"

# 6. Push
echo "Pushing to origin..."
git push origin main
git push origin $VERSION

echo "✅ Release $VERSION complete!"
echo "Create GitHub release at:"
echo "https://github.com/systempromptio/systemprompt-core/releases/new?tag=$VERSION"
```

Usage:
```bash
chmod +x scripts/release.sh
./scripts/release.sh v0.2.0
```

---

## Pre-release Versions

For testing before official release:

```bash
# Create pre-release tag
git tag -a v0.2.0-beta.1 -m "Beta release"
git push origin v0.2.0-beta.1

# Users can test with:
# systemprompt-core-api = { git = "...", tag = "v0.2.0-beta.1" }
```

**Naming conventions**:
- `v1.0.0-alpha.1` - Alpha releases
- `v1.0.0-beta.1` - Beta releases
- `v1.0.0-rc.1` - Release candidates
- `v1.0.0` - Stable release

---

## Hotfix Releases

For critical bugs in production:

```bash
# From current main (v0.2.0)
git checkout -b hotfix/0.2.1

# Fix bug
vim crates/modules/database/src/pool.rs

# Test fix
cargo test --workspace

# Update version (patch bump)
vim Cargo.toml
# version = "0.2.0" → "0.2.1"

# Update changelog
vim CHANGELOG.md

# Commit
git add .
git commit -m "fix: connection pool leak"

# Merge to main
git checkout main
git merge hotfix/0.2.1

# Tag and push
git tag -a v0.2.1 -m "Hotfix v0.2.1: Fix connection pool leak"
git push origin main v0.2.1

# Delete hotfix branch
git branch -d hotfix/0.2.1
```

---

## Future: Crates.io Publishing

If you decide to publish to crates.io in the future:

### Required Changes

1. **Convert path deps to version deps** (~70 edits)
2. **Remove workspace version inheritance**
3. **Create publish script**

### Publish Process

```bash
# Would require publishing in dependency order:
cd crates/shared/traits && cargo publish && sleep 90
cd ../identifiers && cargo publish && sleep 90
cd ../models && cargo publish && sleep 90
# ... (repeat 11 more times)
```

**Time**: 30-60 minutes (vs 2 minutes with git tags)

**Current decision**: Stay with git distribution.

---

## See Also

- [Architecture Guide](./ARCHITECTURE.md) - Workspace structure
- [Installation Guide](./INSTALLATION.md) - How users install
- [Dependency Guide](./DEPENDENCY_GUIDE.md) - How dependencies work
- [FAQ](./FAQ.md) - Common questions
