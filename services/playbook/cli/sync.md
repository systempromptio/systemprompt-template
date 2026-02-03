---
title: "Sync Playbook"
description: "Sync content and data between local and cloud environments."
keywords:
  - sync
  - cloud
  - content
  - upload
  - download
category: cli
---

# Sync Playbook

Sync content and data between local and cloud environments.

> **Help**: `{ "command": "cloud sync" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session Playbook](session.md)

---

## Check Cloud Status

```json
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
{ "command": "cloud sync pull" }
{ "command": "cloud sync pull --dry-run" }
```

---

## Sync Up to Cloud

```json
{ "command": "cloud sync push" }
{ "command": "cloud sync push --dry-run" }
```

---

## Local Content Sync

Content can sync in BOTH directions using different commands.

### How Content Sync Works

```
Disk (services/content/)
    │                    ▲
    │ publish_pipeline   │ content_sync (to-disk)
    ▼                    │
Database (markdown_content)
    │
    ▼ publish_pipeline prerenders
Static HTML (web/dist/)
```

### Sync Disk → Database (Publish)

Run the publish pipeline to ingest content from disk:

```json
{ "command": "infra jobs run publish_pipeline" }
```

This:
1. Reads markdown files from `services/content/`
2. Upserts to `markdown_content` table in database
3. Prerenders to static HTML

### Sync Database → Disk (Export)

Export content from database to disk files (e.g., AI-generated blog posts):

```bash
systemprompt infra jobs run content_sync -p direction=to-disk
```

Or for a specific source:
```bash
systemprompt infra jobs run content_sync -p direction=to-disk -p source=blog
```

This:
1. Reads content from database
2. Creates/updates markdown files in `services/content/`
3. Preserves frontmatter format

**Use this when:** AI agents create content in the database and you want to save it to disk for version control.

### Edit Content

To edit content, modify the markdown files on disk, then run `publish_pipeline`:

```bash
# Edit the file
vim services/content/blog/my-post/index.md

# Re-ingest to database
systemprompt infra jobs run publish_pipeline
```

**CLI `edit` commands only update the database temporarily.** Changes will be overwritten on next `publish_pipeline` run.

---

## Skills Sync

### View Diff

```json
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

**No content synced (0 items)**: `cloud sync push` only syncs files from `storage/`, not database content. Content lives on disk in `services/content/` and is ingested via `publish_pipeline`.

**Cloud DB connection failed**: Check profile with `cloud profile show <profile-name>` and ensure `external_db_access: true` in tenant config.

**CLI edit changes disappeared**: CLI `edit` commands only update the database. The next `publish_pipeline` run overwrites DB with disk content. Edit the markdown files on disk instead.

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
| Pull files | `cloud sync pull` |
| **Content: Disk → DB** | `infra jobs run publish_pipeline` |
| **Content: DB → Disk** | `infra jobs run content_sync -p direction=to-disk` |
| Sync skills | `cloud sync local skills` |
| Cloud DB query | `cloud db query --profile <name> "<SQL>"` |
| Local DB query | `infra db query "<SQL>"` |
