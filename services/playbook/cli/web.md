---
title: "Web Configuration Playbook"
description: "Manage content types, templates, assets, and sitemap."
keywords:
  - web
  - templates
  - content-types
  - sitemap
  - assets
---

# Web Configuration Playbook

Manage content types, templates, assets, and sitemap.

> **Help**: `{ "command": "web" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session Playbook](session.md)

---

## Validate Configuration

```json
// MCP: systemprompt
{ "command": "web validate" }
```

---

## Content Types

```json
// MCP: systemprompt
{ "command": "web content-types list" }
{ "command": "web content-types show blog" }
{ "command": "web content-types create my-type" }
{ "command": "web content-types edit blog" }
{ "command": "web content-types delete my-type -y" }
```

---

## Templates

```json
// MCP: systemprompt
{ "command": "web templates list" }
{ "command": "web templates show base" }
{ "command": "web templates create my-template" }
{ "command": "web templates edit blog-post" }
{ "command": "web templates delete my-template -y" }
```

---

## Assets

```json
// MCP: systemprompt
{ "command": "web assets list" }
{ "command": "web assets show <asset-id>" }
```

---

## Sitemap

```json
// MCP: systemprompt
{ "command": "web sitemap show" }
{ "command": "web sitemap generate" }
{ "command": "web sitemap generate --output sitemap.xml" }
```

---

## Troubleshooting

**Configuration errors**: Run `web validate` to check all web configuration files.

**Missing template**: Run `web templates list` to verify available templates.

---

## Quick Reference

| Task | Command |
|------|---------|
| Validate config | `web validate` |
| List content types | `web content-types list` |
| Show content type | `web content-types show <name>` |
| Create content type | `web content-types create <name>` |
| Edit content type | `web content-types edit <name>` |
| Delete content type | `web content-types delete <name> -y` |
| List templates | `web templates list` |
| Show template | `web templates show <name>` |
| Create template | `web templates create <name>` |
| Edit template | `web templates edit <name>` |
| Delete template | `web templates delete <name> -y` |
| List assets | `web assets list` |
| Show sitemap | `web sitemap show` |
| Generate sitemap | `web sitemap generate` |
