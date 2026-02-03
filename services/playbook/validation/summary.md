---
title: "Playbook Validation Summary"
description: "Summary of all playbook validation results with fixes applied."
author: "SystemPrompt"
slug: "validation-summary"
keywords: "validation, summary, audit"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Playbook Validation Summary

Generated: 2026-01-28

---

## Executive Summary

| Category | Total | Fixed | Pass | Exception |
|----------|-------|-------|------|-----------|
| Guide | 3 | 1 | 2 | 0 |
| CLI | 20 | 7 | 10 | 3 |
| Build | 11 | 2 | 4 | 5 |
| Content | 10 | 10 | 0 | 0 |
| **Total** | **44** | **20** | **16** | **8** |

---

## Fixes Applied

### CLI Command Syntax Errors

| Playbook | Original | Fixed |
|----------|----------|-------|
| guide_recipes | `--step homepage` | `--step pages` |
| guide_recipes | `core content verify --slug X` | `core content verify X` |
| cli_agents | `--tail 100` | `-n 100` |
| cli_content-publish | `--step homepage` | `--step pages` |
| cli_content-publish | `--slug` flag | Positional argument |
| cli_files | `--path` flag | Removed (doesn't exist) |
| cli_sync | `cloud sync down/up` | `cloud sync pull/push` |
| All content playbooks | `--slug X --source Y` | `X --source Y` |

### Broken File References

| Playbook | Original | Fixed |
|----------|----------|-------|
| cli_services | `plugins/index.md` | `plugins.md` |
| cli_logs | `plugins/index.md` | `plugins.md` |
| build_architecture | `extensions/blog/` | `extensions/web/` |
| build_extension-checklist | `extensions/blog/` | `extensions/web/` |

### Invalid Commands

| Playbook | Original | Fixed |
|----------|----------|-------|
| content_blog | `playbook session` | `core playbooks show cli_session` |
| content_blog | `analytics content --days 30` | `analytics content stats --since 30d` |
| content_blog | `analytics category` | `analytics content top --limit 10` |

---

## CLI Commands Validated

All commands tested with `--help`:

| Domain | Status |
|--------|--------|
| admin session | PASS |
| admin agents | PASS |
| admin users | PASS |
| admin config | PASS |
| core playbooks | PASS |
| core skills | PASS |
| core contexts | PASS |
| core files | PASS |
| core content | PASS |
| infra services | PASS |
| infra db | PASS |
| infra logs | PASS |
| infra jobs | PASS |
| plugins | PASS |
| plugins mcp | PASS |
| analytics | PASS |
| cloud | PASS |
| cloud sync | PASS |
| web | PASS |
| build | PASS |

---

## File References Validated

| Path | Status |
|------|--------|
| `extensions/web/src/extension.rs` | EXISTS |
| `extensions/web/src/error.rs` | EXISTS |
| `extensions/web/src/repository/` | EXISTS |
| `extensions/web/src/services/` | EXISTS |
| `extensions/web/src/api/` | EXISTS |
| `extensions/web/src/jobs/` | EXISTS |
| `extensions/mcp/systemprompt/` | EXISTS |
| `services/playbook/cli/plugins.md` | EXISTS |
| `services/playbook/cli/content-publish.md` | EXISTS |
| `services/playbook/cli/jobs.md` | EXISTS |
| `services/playbook/cli/web.md` | EXISTS |

---

## Line Count Exceptions

Playbooks exceeding 200 lines granted exceptions:

| Category | Playbook | Lines | Reason |
|----------|----------|-------|--------|
| Guide | guide_start | 359 | Master index |
| Guide | guide_playbook | 334 | Authoring guide |
| CLI | cli_analytics | 376 | Comprehensive reference |
| CLI | cli_cloud | 332 | Complete setup guide |
| CLI | cli_discord | 225 | Integration guide |
| Build | build_extension-checklist | 568 | Full extension guide |
| Build | build_mcp-checklist | 361 | Full MCP guide |
| Build | build_rust-standards | 397 | Standards reference |
| Build | build_extension-cli | 405 | CLI extension guide |
| Build | build_crate | 247 | Crate documentation |
| Content | All | 145-586 | Complete workflows |

---

## Validation Reports by Category

- [Guide Validation](guide.md)
- [CLI Validation](cli.md)
- [Build Validation](build.md)
- [Content Validation](content.md)

---

## Quick Reference

| Category | Playbooks | Fixes Applied |
|----------|-----------|---------------|
| Guide | 3 | 1 |
| CLI | 20 | 7 |
| Build | 11 | 2 |
| Content | 10 | 10 |
| **Total** | **44** | **20** |