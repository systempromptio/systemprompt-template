---
title: "Web Content"
description: "Creating and publishing markdown content."
keywords:
  - content
  - markdown
  - frontmatter
  - publishing
category: build
---

# Web Content

Creating and publishing markdown content.

> **Help**: `{ "command": "core playbooks show build_web-content" }`

---

## Scope

This playbook covers **content collections** — blog posts, docs, articles stored as markdown files with fixed frontmatter fields.

For **custom pages** with flexible fields (homepage, feature pages), see [Web Pages Architecture](web-pages.md).

---

## Content Location

```
services/content/
├── config.yaml          # Content source definitions
├── blog/               # Blog articles (*.md)
├── documentation/      # Documentation (*.md)
├── legal/              # Legal pages (*.md)
└── skills/             # Agent skills (*.md)
```

---

## Frontmatter Template

```yaml
---
title: "Article Title"
description: "Brief description for SEO (150-160 chars)"
author: "Author Name"
slug: "url-friendly-slug"
keywords: "comma, separated, keywords"
kind: "article"
image: "/files/images/blog/featured-image.webp"
public: true
tags: []
published_at: "2025-12-11"
updated_at: "2026-01-13"
links:
  - title: "Reference Name"
    url: "https://example.com"
---
```

---

## Required Fields

| Field | Required | Notes |
|-------|----------|-------|
| `title` | Yes | Page/article title |
| `description` | Yes | SEO description (150-160 chars) |
| `author` | Yes | Author name |
| `slug` | Yes | URL-friendly identifier |
| `keywords` | Yes | Can be empty string |
| `image` | Yes | Defaults to placeholder if missing |
| `kind` | Yes | Must match templates.yaml |
| `public` | Yes | Set to `true` to publish |
| `published_at` | Yes | ISO date format |
| `updated_at` | Yes | ISO date format |

---

## Content Types (kind)

| Kind | Use For |
|------|---------|
| `article` | Blog posts, news |
| `paper` | Research, whitepapers |
| `guide` | How-to guides, walkthroughs |
| `tutorial` | Learning materials, step-by-step |
| `reference` | API docs, CLI reference |
| `docs-index` | Documentation index pages |
| `docs` | Generic documentation |
| `docs-list` | Documentation list pages |
| `feature` | Feature descriptions |

> **Source of truth**: `extensions/web/src/models/content.rs` (`ContentKind` enum)

---

## Adding New Content Kinds

Adding a new content kind requires updating **three places**:

| Location | File | What to Update |
|----------|------|----------------|
| 1. Rust enum | `extensions/web/src/models/content.rs` | Add variant to `ContentKind` enum |
| 2. Serialization | Same file | Add to `as_str()` and `FromStr` impl |
| 3. Config | `services/content/config.yaml` | Add to `allowed_content_types` for source |

**Checklist:**

1. Add enum variant (e.g., `MyKind`)
2. Add `Self::MyKind => "my-kind"` to `as_str()`
3. Add `"my-kind" => Ok(Self::MyKind)` to `from_str()`
4. Add `"my-kind"` to source's `allowed_content_types` in config
5. Run `just build` to verify
6. Update this playbook's Content Types table

> **Warning**: Adding kinds only to config without updating the Rust enum will cause validation errors.

---

## URL Mapping

| Source Directory | URL Pattern |
|------------------|-------------|
| `services/content/blog/` | `/blog/{slug}` |
| `services/content/documentation/` | `/documentation/{slug}` |
| `services/content/legal/` | `/legal/{slug}` |

### Nested Slugs

```yaml
slug: "config/profiles"
```

Generates: `/documentation/config/profiles/`

---

## Publishing Workflow

```bash
systemprompt infra jobs run publish_pipeline
```

### Individual Steps

| Step | Command |
|------|---------|
| Ingest | `systemprompt infra jobs run blog_content_ingestion` |
| Assets | `systemprompt infra jobs run copy_extension_assets` |
| Prerender | `systemprompt infra jobs run publish_pipeline` |
| Homepage | `systemprompt infra jobs run publish_pipeline` |
| Sitemap | `systemprompt infra jobs run publish_pipeline` |

---

## Content Management

```bash
systemprompt core content list --source blog --limit 20
systemprompt core content show my-article-slug --source blog
systemprompt core content search "MCP server"
systemprompt core content delete <id> --yes
```

---

## Troubleshooting

| Error | Solution |
|-------|----------|
| No template for content type | Add mapping to `templates.yaml` |
| Missing field 'X' | Add field to frontmatter |
| Page data provider failed | Run `--step ingest` first |

---

## Quick Reference

| Task | Command |
|------|---------|
| Full publish | `infra jobs run publish_pipeline` |
| List content | `core content list --source <source>` |
| Search | `core content search "<query>"` |
| Upload image | `core files upload <path>` |

-> See [Web Templates](web-templates.md) for template configuration.
-> See [Web Assets](web-assets.md) for CSS/JS/images.
