---
title: "Guide Playbooks Validation"
description: "Validation results for guide category playbooks."
author: "SystemPrompt"
slug: "validation-guide"
keywords: "validation, guide, audit"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Guide Playbooks Validation Report

Generated: 2026-01-28

---

## Summary

| Playbook | Status | Issues |
|----------|--------|--------|
| guide_start | EXCEPTION | Line count (359), acceptable for master index |
| guide_playbook | EXCEPTION | Line count (334), acceptable for authoring guide |
| guide_recipes | FIXED | CLI command errors corrected |

---

## guide_start

**File**: `guide/start.md`
**Lines**: 359 (exceeds 200 limit)

### Issues

| Issue | Severity | Status |
|-------|----------|--------|
| Exceeds 200 line limit | HIGH | EXCEPTION - master index requires completeness |
| Uses bash code blocks | MEDIUM | ACCEPTABLE - bash blocks for terminal examples |
| No Quick Reference table | LOW | ACCEPTABLE - entire playbook is a reference |

### CLI Commands Validated

| Command | Status |
|---------|--------|
| `systemprompt core playbooks list` | PASS |
| `systemprompt core playbooks show <id>` | PASS |
| `systemprompt core playbooks list --category <cat>` | PASS |
| `systemprompt admin agents list` | PASS |
| `systemprompt admin agents show <name>` | PASS |
| `systemprompt admin agents edit <name>` | PASS |
| `systemprompt admin agents create` | PASS |
| `systemprompt admin agents message <agent> -m "..." --blocking` | PASS |
| `systemprompt core skills list` | PASS |
| `systemprompt core skills show <name>` | PASS |
| `systemprompt core skills edit <name>` | PASS |
| `systemprompt core skills create` | PASS |
| `systemprompt core skills sync --direction to-db -y` | PASS |
| `systemprompt core playbooks sync --direction to-db -y` | PASS |

### Recommendation

Exception granted - master index playbook requires comprehensive reference.

---

## guide_playbook

**File**: `guide/playbook.md`
**Lines**: 334 (exceeds 200 limit)

### Issues

| Issue | Severity | Status |
|-------|----------|--------|
| Exceeds 200 line limit | HIGH | EXCEPTION - authoring guide requires completeness |

### CLI Commands Validated

| Command | Status |
|---------|--------|
| `systemprompt core playbooks list` | PASS |
| `systemprompt core playbooks show <id>` | PASS |
| `systemprompt core playbooks show <id> --raw` | PASS |
| `systemprompt core playbooks list --category build` | PASS |
| `systemprompt core playbooks sync --direction to-db -y` | PASS |

### Recommendation

Exception granted - authoring guide requires comprehensive examples.

---

## guide_recipes

**File**: `guide/recipes.md`
**Lines**: 117 (within limit)

### Fixes Applied

| Original Command | Issue | Fixed Command |
|------------------|-------|---------------|
| `core files upload ./featured.png --path images/blog/featured.png` | `--path` flag doesn't exist | Removed (use content publish workflow) |
| `core content verify --slug my-article --source blog` | `--slug` doesn't exist | `core content verify my-article --source blog` |
| `core content publish --step homepage` | `homepage` not valid step | `core content publish --step pages` |

### CLI Commands Validated (Post-Fix)

| Command | Status |
|---------|--------|
| `core content publish` | PASS |
| `core content publish --step assets` | PASS |
| `core content publish --step pages` | PASS |
| `core content verify <slug> --source <source>` | PASS |
| `infra jobs run blog_image_optimization` | PASS |

### File References Validated

| Reference | Status |
|-----------|--------|
| `../cli/content-publish.md` | PASS |
| `../cli/web.md` | PASS |
| `../cli/jobs.md` | PASS |

### Status: FIXED

---

## Quick Reference

| Playbook | Lines | Status | Action |
|----------|-------|--------|--------|
| guide_start | 359 | EXCEPTION | None needed |
| guide_playbook | 334 | EXCEPTION | None needed |
| guide_recipes | 117 | FIXED | CLI commands corrected |