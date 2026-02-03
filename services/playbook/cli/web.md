---
title: "Web Configuration Playbook"
description: "Configure templates, content types, and web settings."
author: "SystemPrompt"
slug: "cli-web"
keywords: "web, templates, config, content-types"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Web Configuration

Configure templates, content types, and web settings.

---

## Validate Configuration

```json
{ "command": "web validate" }
```

---

## Content Types

```json
{ "command": "web content-types list" }
{ "command": "web content-types show <name>" }
{ "command": "web content-types create <name>" }
{ "command": "web content-types edit <name>" }
{ "command": "web content-types delete <name> -y" }
```

---

## Templates

```json
{ "command": "web templates list" }
{ "command": "web templates show <name>" }
{ "command": "web templates create <name>" }
{ "command": "web templates edit <name>" }
{ "command": "web templates delete <name> -y" }
```

---

## Assets

```json
{ "command": "web assets list" }
{ "command": "web assets show <asset-id>" }
```

---

## Sitemap

```json
{ "command": "web sitemap show" }
{ "command": "web sitemap generate" }
{ "command": "web sitemap generate --output sitemap.xml" }
```

---

## Troubleshooting

**Template not found** -- Check template name with `web templates list`.

**Invalid config** -- Run `web validate` to check configuration.

**Asset not loading** -- Verify asset exists with `web assets list`, then run `core content publish --step assets`.

---

## Quick Reference

| Task | Command |
|------|---------|
| Validate config | `web validate` |
| List content types | `web content-types list` |
| Show content type | `web content-types show <name>` |
| Edit content type | `web content-types edit <name>` |
| List templates | `web templates list` |
| Show template | `web templates show <name>` |
| Edit template | `web templates edit <name>` |
| List assets | `web assets list` |
| Generate sitemap | `web sitemap generate` |

---

## Related Playbooks

- [Content Publish](content-publish.md) - Publish and manage content
- [Recipes](../guide/recipes.md) - Complete workflow examples
- [Web Content](../build/web-content.md) - Markdown content structure
- [Web Templates](../build/web-templates.md) - Handlebars templates
- [Web Assets](../build/web-assets.md) - CSS, JS, fonts