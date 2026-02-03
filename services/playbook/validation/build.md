---
title: "Build Playbooks Validation"
description: "Validation results for build category playbooks."
author: "SystemPrompt"
slug: "validation-build"
keywords: "validation, build, audit"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Build Playbooks Validation Report

Generated: 2026-01-28

---

## Summary

| Playbook | Lines | Status | Issues |
|----------|-------|--------|--------|
| build_architecture | 209 | FIXED | `extensions/blog/` → `extensions/web/` |
| build_crate | 247 | EXCEPTION | Exceeds 200 lines |
| build_extension-checklist | 568 | FIXED | `extensions/blog/` → `extensions/web/` |
| build_extension-cli | 405 | EXCEPTION | Comprehensive CLI guide |
| build_extension-review | 164 | PASS | |
| build_mcp-checklist | 361 | EXCEPTION | Comprehensive MCP guide |
| build_mcp-review | 144 | PASS | |
| build_rust-standards | 397 | EXCEPTION | Comprehensive standards |
| build_web-assets | 194 | PASS | |
| build_web-content | 152 | PASS | |
| build_web-templates | 205 | PASS | |

---

## Fixes Applied

### build_architecture

**File**: `build/architecture.md`

| Original | Issue | Fixed |
|----------|-------|-------|
| `extensions/blog/` | Directory doesn't exist | `extensions/web/` |
| `extensions/blog/src/extension.rs` | File doesn't exist | `extensions/web/src/extension.rs` |
| `extensions/blog/src/error.rs` | File doesn't exist | `extensions/web/src/error.rs` |
| `extensions/blog/src/repository/` | Dir doesn't exist | `extensions/web/src/repository/` |
| `extensions/blog/src/services/` | Dir doesn't exist | `extensions/web/src/services/` |
| `extensions/blog/src/api/` | Dir doesn't exist | `extensions/web/src/api/` |
| `extensions/blog/src/jobs/` | Dir doesn't exist | `extensions/web/src/jobs/` |

---

### build_extension-checklist

**File**: `build/extension-checklist.md`

| Original | Issue | Fixed |
|----------|-------|-------|
| All `extensions/blog/` references | Directory doesn't exist | Changed to `extensions/web/` |

---

## File References Validated

All file references checked against actual filesystem:

| Path | Status |
|------|--------|
| `extensions/web/src/extension.rs` | EXISTS |
| `extensions/web/src/error.rs` | EXISTS |
| `extensions/web/src/repository/` | EXISTS |
| `extensions/web/src/services/` | EXISTS |
| `extensions/web/src/api/` | EXISTS |
| `extensions/web/src/jobs/` | EXISTS |
| `extensions/mcp/systemprompt/` | EXISTS |
| `extensions/cli/` | EXISTS |
| `extensions/homepage/` | EXISTS |

---

## Line Count Exceptions

The following playbooks exceed 200 lines but are granted exceptions:

| Playbook | Lines | Reason |
|----------|-------|--------|
| build_extension-checklist | 568 | Comprehensive extension development guide |
| build_mcp-checklist | 361 | Complete MCP server implementation guide |
| build_rust-standards | 397 | Full Rust coding standards reference |
| build_extension-cli | 405 | Comprehensive CLI extension guide |
| build_crate | 247 | Complete umbrella crate documentation |

---

## Quick Reference

| Playbook | Lines | Status |
|----------|-------|--------|
| build_architecture | 209 | FIXED |
| build_extension-checklist | 568 | FIXED |
| build_extension-cli | 405 | EXCEPTION |
| build_extension-review | 164 | PASS |
| build_mcp-checklist | 361 | EXCEPTION |
| build_mcp-review | 144 | PASS |
| build_rust-standards | 397 | EXCEPTION |
| build_crate | 247 | EXCEPTION |
| build_web-content | 152 | PASS |
| build_web-templates | 205 | PASS |
| build_web-assets | 194 | PASS |