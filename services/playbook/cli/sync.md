---
title: "Sync Playbook"
description: "Sync content and data between local and cloud environments."
keywords:
  - sync
  - cloud
  - content
  - upload
  - download
---

# Sync Playbook

Sync content and data between local and cloud environments.

> **Help**: `{ "command": "cloud sync" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session Playbook](session.md)

---

## Check Cloud Status

```json
// MCP: systemprompt
{ "command": "cloud status" }
```

Check authentication:
```json
{ "command": "cloud auth whoami" }
```

---

## Sync Files to Cloud

### Dry Run First

```json
// MCP: systemprompt
{ "command": "cloud sync push --dry-run --verbose" }
```

### Execute Sync

```json
{ "command": "cloud sync push --verbose" }
```

This syncs files from `storage/`, not database content.

---

## Sync Down from Cloud

```json
// MCP: systemprompt
{ "command": "cloud sync down" }
```

---

## Sync Up to Cloud

```json
// MCP: systemprompt
{ "command": "cloud sync up" }
```

---

## Local Content Sync (DB <-> Disk)

### View Diff

```json
// MCP: systemprompt
{ "command": "cloud sync local content --dry-run" }
```

### Export DB to Disk

```json
{ "command": "cloud sync local content --direction to-disk" }
```

### Import Disk to DB

```json
{ "command": "cloud sync local content --direction to-db" }
```

### Sync Specific Source

```json
{ "command": "cloud sync local content --source blog --direction to-disk" }
```

---

## Skills Sync

### View Diff

```json
// MCP: systemprompt
{ "command": "cloud sync local skills --dry-run" }
```

### Execute Sync

```json
{ "command": "cloud sync local skills" }
```

---

## Database Comparison

### Count Local Content

```json
// MCP: systemprompt
{ "command": "infra db query \"SELECT COUNT(*) FROM markdown_content\"" }
```

### Count Cloud Content

```json
{ "command": "cloud db query --profile <profile-name> \"SELECT COUNT(*) FROM markdown_content\"" }
```

### Compare Sources

```json
{ "command": "infra db query \"SELECT source, COUNT(*) FROM markdown_content GROUP BY source\"" }
{ "command": "cloud db query --profile <profile-name> \"SELECT source, COUNT(*) FROM markdown_content GROUP BY source\"" }
```

---

## Troubleshooting

**UUID parsing error**: Local profile has non-UUID tenant ID. Switch to a cloud profile first with `admin session switch <profile-name>`.

**No content synced (0 items)**: `cloud sync push` only syncs files, not database content. Export content to disk first, then sync files.

**Cloud DB connection failed**: Check profile with `cloud profile show <profile-name>` and ensure `external_db_access: true` in tenant config.

-> See [Cloud Playbook](cloud.md) for cloud authentication and profiles.
-> See [Deploy Playbook](deploy.md) for deployment workflows.
-> See [Session Playbook](session.md) for switching profiles.
-> See [Database Playbook](database.md) for cloud database operations.

---

## Quick Reference

| Task | Command |
|------|---------|
| Cloud status | `cloud status` |
| Auth check | `cloud auth whoami` |
| Push files | `cloud sync push` |
| Sync down | `cloud sync down` |
| Sync up | `cloud sync up` |
| Export to disk | `cloud sync local content --direction to-disk` |
| Import to DB | `cloud sync local content --direction to-db` |
| Sync skills | `cloud sync local skills` |
| Cloud DB query | `cloud db query --profile <name> "<SQL>"` |
| Local DB query | `infra db query "<SQL>"` |
