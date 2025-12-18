# Publishing systemprompt-core

This guide covers syncing changes from the `core/` subtree in systemprompt-blog back to the standalone systemprompt-core repository and publishing a new version.

## Overview

- `core/` - Git subtree integrated into systemprompt-blog for development
- `../systemprompt-core/` - Standalone repository for the core library
- Both point to `https://github.com/systempromptio/systemprompt-core.git`

## Publishing Workflow

### 1. Sync changes to systemprompt-core

Copy all changes from the subtree to the standalone repo:

```bash
# From systemprompt-blog root
rsync -av --delete \
  --exclude '.git' \
  --exclude 'target' \
  --exclude 'node_modules' \
  --exclude '.env' \
  --exclude '*.db' \
  core/ ../systemprompt-core/
```

### 2. Review changes

```bash
cd ../systemprompt-core
git status
git diff
```

### 3. Bump version

Edit `Cargo.toml` and update the workspace version:

```toml
[workspace.package]
version = "X.Y.Z"  # Increment appropriately
```

Follow semantic versioning:
- **Patch (0.0.X)**: Bug fixes, minor changes
- **Minor (0.X.0)**: New features, backward compatible
- **Major (X.0.0)**: Breaking changes

### 4. Commit and push

```bash
cd ../systemprompt-core
git add -A
git commit -m "Release vX.Y.Z

- Summary of changes
- Additional details

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"

git push origin main
```

### 5. Create GitHub release (optional)

```bash
gh release create vX.Y.Z --title "vX.Y.Z" --notes "Release notes here"
```

## Quick Commands

One-liner for syncing:

```bash
rsync -av --delete --exclude '.git' --exclude 'target' --exclude 'node_modules' --exclude '.env' --exclude '*.db' core/ ../systemprompt-core/
```

Check current versions:

```bash
# Blog's core subtree version
grep 'version = ' core/Cargo.toml | head -5

# Standalone repo version
grep 'version = ' ../systemprompt-core/Cargo.toml | head -5
```

## Notes

- Always test the build before publishing: `cd ../systemprompt-core && cargo build --workspace`
- The `core/` subtree in systemprompt-blog uses the `core-remote` git remote
- After publishing, you may want to update the subtree reference in systemprompt-blog
