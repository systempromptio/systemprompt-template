---
title: "Content Publishing Playbook"
description: "Publish and manage web content via CLI."
author: "SystemPrompt"
slug: "cli-content-publish"
keywords: "content, publishing, blog, documentation"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Content Publishing

Publish and manage web content via CLI.

---

## Publishing Pipeline

### Full Publish

```json
{ "command": "core content publish" }
```

Runs: ingest -> assets -> prerender -> homepage -> sitemap

### Individual Steps

```json
{ "command": "core content publish --step ingest" }
{ "command": "core content publish --step assets" }
{ "command": "core content publish --step prerender" }
{ "command": "core content publish --step pages" }
{ "command": "core content publish --step sitemap" }
```

| Step | Action |
|------|--------|
| `ingest` | Parse markdown files into database |
| `assets` | Copy CSS/JS from storage to web/dist |
| `prerender` | Generate static HTML from database |
| `pages` | Generate static pages (homepage, etc.) |
| `sitemap` | Generate sitemap.xml and feed.xml |

---

## Content Management

### List Content

```json
{ "command": "core content list --source blog" }
{ "command": "core content list --source documentation" }
{ "command": "core content list --source legal" }
{ "command": "core content list --source blog --limit 20 --offset 0" }
```

### Show Content

```json
{ "command": "core content show <slug> --source <source>" }
```

### Search Content

```json
{ "command": "core content search \"<query>\"" }
```

### Verify Content

```json
{ "command": "core content verify <slug> --source <source>" }
{ "command": "core content verify <slug> --source <source> --base-url https://example.com" }
```

### Content Status

```json
{ "command": "core content status --source <source>" }
```

### Re-ingest Content

```json
{ "command": "core content ingest services/content/blog --source blog --override" }
```

### Delete Content

```json
{ "command": "core content delete <content-id> --yes" }
```

---

## File Management

### Upload Files

```json
{ "command": "core files upload ./my-image.png --context <context-id>" }
```

### List Files

```json
{ "command": "core files list" }
{ "command": "core files list --limit 50" }
{ "command": "core files list --mime image/png" }
```

### Show/Search Files

```json
{ "command": "core files show <file-id>" }
{ "command": "core files search \"blog\"" }
```

### Delete Files

```json
{ "command": "core files delete <file-id> -y" }
```

### Storage Stats

```json
{ "command": "core files stats" }
{ "command": "core files config" }
```

---

## Sync Content

### Export to Disk

```json
{ "command": "cloud sync local content --direction to-disk" }
{ "command": "cloud sync local content --direction to-disk --source blog" }
```

### Import to Database

```json
{ "command": "cloud sync local content --direction to-db" }
```

### Dry Run

```json
{ "command": "cloud sync local content --dry-run" }
```

---

## Background Jobs

### Content Jobs

```json
{ "command": "infra jobs run publish_content" }
{ "command": "infra jobs run blog_image_optimization" }
{ "command": "infra jobs run copy_extension_assets" }
```

---

## Troubleshooting

**Content not appearing** -- Verify published with `core content verify --slug <slug> --source <source>`.

**Missing images** -- Check file upload and run `core content publish --step assets`.

**Stale content** -- Re-ingest with `core content ingest <path> --source <source> --override`.

---

## Quick Reference

| Task | Command |
|------|---------|
| Full publish | `core content publish` |
| Ingest only | `core content publish --step ingest` |
| Assets only | `core content publish --step assets` |
| Prerender only | `core content publish --step prerender` |
| Pages only | `core content publish --step pages` |
| Sitemap only | `core content publish --step sitemap` |
| List content | `core content list --source <source>` |
| Show content | `core content show <slug> --source <source>` |
| Search | `core content search "<query>"` |
| Verify | `core content verify <slug> --source <source>` |
| Upload file | `core files upload <path>` |
| List files | `core files list` |
| Export to disk | `cloud sync local content --direction to-disk` |
| Import to DB | `cloud sync local content --direction to-db` |