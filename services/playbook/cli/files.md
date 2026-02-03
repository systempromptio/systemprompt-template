---
title: "Files Management Playbook"
description: "Upload, manage, and search files in the storage system."
author: "SystemPrompt"
slug: "cli-files"
keywords: "files, upload, storage, images"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Files Management Playbook

Upload, manage, and search files in the storage system.

---

## List Files

```json
{ "command": "core files list" }
{ "command": "core files list --limit 50" }
{ "command": "core files list --limit 20 --offset 40" }
```

Filter by MIME type (use `--mime` with full MIME type, not `--type`):
```json
{ "command": "core files list --mime image/png" }
{ "command": "core files list --mime image/webp" }
{ "command": "core files list --mime application/pdf" }
```

---

## Show File Details

```json
{ "command": "core files show file_abc123" }
```

---

## Upload Files

```json
{ "command": "core files upload /path/to/image.png --context <context-id>" }
{ "command": "core files upload /path/to/image.png --context <context-id> --ai" }
{ "command": "core files validate /path/to/image.png" }
```

| Flag | Purpose |
|------|---------|
| `--context` | Required context ID for the upload |
| `--ai` | Mark as AI-generated content |
| `--user` | Associate with user ID |
| `--session` | Associate with session ID |

---

## Search Files

```json
{ "command": "core files search \"blog\"" }
{ "command": "core files search \"*.png\"" }
{ "command": "core files search \"images/2026\"" }
```

---

## Delete Files

```json
{ "command": "core files delete file_abc123" }
{ "command": "core files delete file_abc123 -y" }
```

---

## Storage Statistics

```json
{ "command": "core files stats" }
```

Shows total files, storage used, files by type, and recent uploads.

---

## File Configuration

```json
{ "command": "core files config" }
```

Shows max file size, allowed types, and storage path.

---

## AI-Generated Images

```json
{ "command": "core files ai list" }
{ "command": "core files ai show <image-id>" }
```

---

## Troubleshooting

**Upload failed (file too large or invalid type)** -- check allowed limits with `core files config`.

**File not found** -- search by partial name with `core files search "partial-name"`.

---

## Quick Reference

| Task | Command |
|------|---------|
| List files | `core files list` |
| List by MIME | `core files list --mime image/png` |
| Show file | `core files show <id>` |
| Upload file | `core files upload /path --context <id>` |
| Validate file | `core files validate /path/to/file` |
| Search files | `core files search "pattern"` |
| Delete file | `core files delete <id> -y` |
| Storage stats | `core files stats` |
| File config | `core files config` |
| List AI images | `core files ai list` |