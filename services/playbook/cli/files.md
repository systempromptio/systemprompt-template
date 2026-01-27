---
title: "Files Management Playbook"
description: "Upload, manage, and search files in the storage system."
keywords:
  - files
  - upload
  - storage
  - images
---

# Files Management Playbook

Upload, manage, and search files in the storage system.

> **Help**: `{ "command": "core files" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session Playbook](session.md)

---

## List Files

```json
// MCP: systemprompt
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
// MCP: systemprompt
{ "command": "core files show file_abc123" }
```

---

## Upload Files

```json
// MCP: systemprompt
{ "command": "core files upload /path/to/image.png" }
{ "command": "core files upload /path/to/image.png --path images/blog/featured.png" }
{ "command": "core files validate /path/to/image.png" }
```

---

## Search Files

```json
// MCP: systemprompt
{ "command": "core files search \"blog\"" }
{ "command": "core files search \"*.png\"" }
{ "command": "core files search \"images/2026\"" }
```

---

## Delete Files

```json
// MCP: systemprompt
{ "command": "core files delete file_abc123" }
{ "command": "core files delete file_abc123 -y" }
```

---

## Storage Statistics

```json
// MCP: systemprompt
{ "command": "core files stats" }
```

Shows total files, storage used, files by type, and recent uploads.

---

## File Configuration

```json
// MCP: systemprompt
{ "command": "core files config" }
```

Shows max file size, allowed types, and storage path.

---

## AI-Generated Images

```json
// MCP: systemprompt
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
| Upload file | `core files upload /path/to/file` |
| Upload with path | `core files upload /path --path storage/path` |
| Validate file | `core files validate /path/to/file` |
| Search files | `core files search "pattern"` |
| Delete file | `core files delete <id> -y` |
| Storage stats | `core files stats` |
| File config | `core files config` |
| List AI images | `core files ai list` |
