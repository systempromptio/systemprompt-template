# Git Subtree Workflow Guide

## Overview

This repository uses **git subtree** to embed the [systemprompt-core](https://github.com/systempromptio/systemprompt-core) platform at the `core/` directory. This guide documents the workflow for maintaining this setup.

### Key Concepts

- **systemprompt-blog**: This repository (the blog application with custom services)
- **systemprompt-core**: The core platform repository (the shared engine)
- **Git Subtree**: Embeds the full core repository history as a subdirectory
- **Sibling Repo**: `../systemprompt-core` exists as a separate clone

## Architecture

```
systemprompt-blog (this repo)
├── core/                      ← Git subtree from systemprompt-core
│   ├── crates/               ← Core modules
│   ├── Cargo.toml           ← Core workspace
│   └── ... (full systemprompt-core repo embedded)
├── crates/                    ← Blog-specific services
├── Cargo.toml                ← Blog workspace
└── ... (blog-specific code)
```

**Workflow Model:**
1. Blog-specific code is committed/pushed to `origin` (systemprompt-blog repo)
2. Core platform code is synced via subtree to `core-remote` (systemprompt-core repo)
3. Both repositories maintain independent version histories

## Repository Setup

### Remotes Configuration

```bash
# List remotes
git remote -v

# Expected output:
# core-remote    https://github.com/systempromptio/systemprompt-core.git (fetch)
# core-remote    https://github.com/systempromptio/systemprompt-core.git (push)
# origin         https://github.com/systempromptio/systemprompt-blog.git (fetch)
# origin         https://github.com/systempromptio/systemprompt-blog.git (push)
```

### Initial Setup Reference

If you need to recreate this setup (for reference only):

```bash
# Add the subtree (one-time only)
git subtree add --prefix=core \
  https://github.com/systempromptio/systemprompt-core.git \
  main --squash
```

## Common Workflows

### 1. Syncing Core Updates FROM systemprompt-core TO blog

When changes are made to systemprompt-core and you want to pull them into this blog repo:

```bash
# From systemprompt-blog root
git subtree pull --prefix=core core-remote main --squash
```

**Using justfile (recommended):**
```bash
just update-core
```

**What happens:**
- Fetches latest from core-remote/main
- Creates a squashed merge commit in this repo
- All core changes appear under `core/` directory

### 2. Syncing Blog Changes TO systemprompt-core (Full Sync)

When you have changes in `core/` (from edits or conflicts) and want to push them back:

```bash
# From systemprompt-blog root
git subtree push --prefix=core core-remote main
```

**What happens:**
- Extracts all commits from core/ history
- Pushes to systemprompt-core's main branch
- Note: This overwrites history on the remote side if not done carefully

**Important:** Use `--force` only if explicitly needed and coordinated with team:
```bash
git subtree push --prefix=core core-remote main --force
```

### 3. Complete Sync Workflow (Recommended Approach)

For a full bi-directional sync with version management:

**Step 1: Commit blog changes**
```bash
# From systemprompt-blog root
git add .
git commit -m "chore: sync blog-specific changes"
git push origin main
```

**Step 2: Push core changes**
```bash
git subtree push --prefix=core core-remote main
```

**Step 3: Update version in systemprompt-core**
```bash
# Switch to sibling repo
cd ../systemprompt-core

# Pull the changes pushed in Step 2
git pull origin main

# Update version
# - Edit Cargo.toml files (bump version)
# - Edit any changelog/release notes

git add .
git commit -m "chore: bump version to X.Y.Z"
git push origin main
```

**Step 4: Pull latest back to blog**
```bash
# Return to systemprompt-blog
cd ../systemprompt-blog

# Pull the version bump
git subtree pull --prefix=core core-remote main --squash

# Verify clean state
git status
```

## Understanding Squashed Merges

This setup uses `--squash` option for subtree pulls. This means:

**Pros:**
- Cleaner blog repo history (no core commit noise)
- Easier to review what changed in core
- Simpler conflict resolution

**Cons:**
- Core history is flattened (one commit per sync)
- Can't directly revert a specific core commit

**Example commit message:**
```
Squashed 'core/' changes from 35f1a7ef7..b678fe6c9

git-subtree-dir: core
git-subtree-split: b678fe6c9
```

## Troubleshooting

### Issue: Git subtree push fails with "no commits matched"

**Cause:** The core/ directory doesn't have the right history configured.

**Solution:**
```bash
# Verify the subtree was added correctly
git log --oneline | grep -i subtree

# If nothing found, the subtree may not be properly initialized
# Review the subtree add commit in history
git log --all --source -- core/ | head -20
```

### Issue: Merge conflicts in core/

**When they happen:** Usually when both repos modified the same files simultaneously.

**Resolution:**
```bash
# Stop the merge
git merge --abort

# Option 1: Keep blog's version
git checkout --ours core/

# Option 2: Keep core-remote's version
git checkout --theirs core/

# Option 3: Manual merge
# Edit conflicted files, then:
git add core/
git commit -m "resolve: merge conflicts in core"
```

### Issue: Version mismatch between blog's core/ and systemprompt-core

**Prevent:** Always follow the complete sync workflow above.

**Fix:**
```bash
# Check version in blog
cat core/Cargo.toml | grep "^version"

# Check version in systemprompt-core
cd ../systemprompt-core
cat Cargo.toml | grep "^version"

# They should match. If not, pull latest:
cd ../systemprompt-blog
git subtree pull --prefix=core core-remote main --squash
```

### Issue: Subtree push tries to rewrite a lot of history

**Cause:** The core-remote branch is out of sync with what's in core/

**Prevention:**
- Always use `git subtree pull` before `git subtree push`
- Don't manually edit commits in core/ history
- Coordinate with team if multiple people access core-remote

## Dependency Model

Currently using **path dependencies** for core modules:

```toml
# In systemprompt-blog/Cargo.toml
[workspace.dependencies]
systemprompt-models = { path = "core/crates/shared/models" }
systemprompt-core-mcp = { path = "core/crates/modules/mcp" }
# ... etc
```

**Future:** Can migrate to git dependencies once core is stable:
```toml
systemprompt-models = {
  git = "https://github.com/systempromptio/systemprompt-core",
  branch = "main"
}
```

**When to migrate:**
- Core is published to crates.io, OR
- Core repo is fully public with stable API

## Best Practices

1. **Always pull before push:**
   ```bash
   git subtree pull --prefix=core core-remote main --squash
   git subtree push --prefix=core core-remote main
   ```

2. **Separate commits for blog vs core changes**
   - Commit blog changes: `git add . && git commit ...`
   - Push blog changes: `git push origin main`
   - Then sync core via subtree commands

3. **Document major changes**
   - Clearly note in commit messages what was synced
   - Example: "chore: sync core updates, bump to v0.1.0"

4. **Use justfile for common tasks**
   ```bash
   just update-core      # Pull latest core
   just build            # Build everything
   just test             # Run tests
   ```

5. **Verify after each sync**
   ```bash
   git status            # Should be clean
   git log -1            # Verify last commit
   cargo check           # Verify dependencies resolve
   ```

6. **Coordinate with team**
   - Let team know before doing `git subtree push`
   - Check systemprompt-core for new PRs/changes
   - Communicate version bumps

## Version Management

### Current Version: 0.0.7

Version is tracked in multiple places:

**Blog repo:**
- `Cargo.toml` (workspace version)

**Core repo:** (authoritative)
- `core/Cargo.toml`
- `core/crates/*/Cargo.toml` (crate versions)

**Versioning Strategy:**
- **Patch (X.Y.Z):** Bug fixes, small changes
- **Minor (X.Y.0):** New features, backward compatible
- **Major (X.0.0):** Breaking changes

**When to bump:**
1. Make changes to core
2. Push via `git subtree push`
3. Update version in systemprompt-core
4. Commit and push
5. Pull back to blog

## See Also

- `README.md` - Project overview
- `claude.md` - Architecture guide
- `core/ARCHITECTURE.md` - Core platform architecture
- `core/docs/INSTALLATION.md` - Detailed setup

## Command Reference

| Command | Purpose |
|---------|---------|
| `git subtree pull --prefix=core core-remote main --squash` | Pull latest core |
| `git subtree push --prefix=core core-remote main` | Push core changes |
| `just update-core` | Pull core (via justfile) |
| `git log --oneline -- core/` | View core changes in blog |
| `git diff HEAD~1 -- core/` | See what changed in last sync |
| `git remote -v` | Verify remotes configured |

---

**Last Updated:** 2025-10-29
**Version Reference:** 0.0.7
