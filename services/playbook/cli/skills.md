---
title: "Skills Playbook"
description: "Configure and sync skills between disk and database."
author: "SystemPrompt"
slug: "cli-skills"
keywords: "skills, agents, yaml, sync"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Skills Playbook

Configure and sync skills between disk and database.

---

## Understanding Skills

Skills are reusable capabilities that agents can use. Each skill defines:

| Field | Purpose |
|-------|---------|
| `name` | Unique identifier |
| `description` | What the skill does |
| `instructions` | Detailed guidance for the agent |
| `tools` | MCP tools the skill can access |

Skills are stored in YAML files on disk and synced to the database.

---

## Skills Location

Skills are defined in YAML files:
```
services/skills/
├── research.yaml
├── blog_writing.yaml
└── social_media.yaml
```

---

## List Skills

```json
{ "command": "core skills list" }
```

---

## Check Sync Status

Compare disk files with database:
```json
{ "command": "core skills status" }
```

Shows skills on disk only, in DB only, or in sync.

---

## Sync Skills

### Dry Run (Preview)

```json
{ "command": "core skills sync --dry-run" }
```

### Execute Sync

```json
{ "command": "core skills sync" }
```

---

## Create New Skill

```json
{ "command": "core skills create my_skill" }
{ "command": "core skills create my_skill --description \"Does something useful\"" }
```

Creates a new skill YAML file in `services/skills/`.

---

## Edit Skill

```json
{ "command": "core skills edit my_skill" }
```

---

## Delete Skill

```json
{ "command": "core skills delete my_skill" }
{ "command": "core skills delete my_skill -y" }
```

---

## Troubleshooting

**Skill not showing in agent**: Check `core skills status`, run `core skills sync`, then verify with `admin agents show <agent-name>`.

**Sync conflicts**: Preview changes with `core skills sync --dry-run` before syncing.

---

## Quick Reference

| Task | Command |
|------|---------|
| List skills | `core skills list` |
| Check sync status | `core skills status` |
| Sync skills | `core skills sync` |
| Preview sync | `core skills sync --dry-run` |
| Create skill | `core skills create <name>` |
| Edit skill | `core skills edit <name>` |
| Delete skill | `core skills delete <name> -y` |