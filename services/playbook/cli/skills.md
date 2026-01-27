---
title: "Skills Playbook"
description: "Configure and sync skills between disk and database."
keywords:
  - skills
  - agents
  - yaml
  - sync
---

# Skills Playbook

Configure and sync skills between disk and database.

> **Help**: `{ "command": "core skills" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session Playbook](session.md)

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
// MCP: systemprompt
{ "command": "core skills list" }
```

---

## Check Sync Status

Compare disk files with database:
```json
// MCP: systemprompt
{ "command": "core skills status" }
```

Shows skills on disk only, in DB only, or in sync.

---

## Sync Skills

### Dry Run (Preview)

```json
// MCP: systemprompt
{ "command": "core skills sync --dry-run" }
```

### Execute Sync

```json
{ "command": "core skills sync" }
```

---

## Create New Skill

```json
// MCP: systemprompt
{ "command": "core skills create my_skill" }
{ "command": "core skills create my_skill --description \"Does something useful\"" }
```

Creates a new skill YAML file in `services/skills/`.

---

## Edit Skill

```json
// MCP: systemprompt
{ "command": "core skills edit my_skill" }
```

---

## Delete Skill

```json
// MCP: systemprompt
{ "command": "core skills delete my_skill" }
{ "command": "core skills delete my_skill -y" }
```

---

## Troubleshooting

**Skill not showing in agent**: Check `core skills status`, run `core skills sync`, then verify with `admin agents show <agent-name>`.

**Sync conflicts**: Preview changes with `core skills sync --dry-run` before syncing.

-> See [Sync Playbook](sync.md) for deploying skills to cloud.
-> See [Agents Playbook](agents.md) for assigning skills to agents.

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
