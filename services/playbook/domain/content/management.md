---
title: "Content Management"
description: "Manage content sources, categories, sitemap, RSS feeds, and publishing workflows."
author: "SystemPrompt"
slug: "domain-content-management"
keywords: "content, management, sources, categories, sitemap, rss"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Content Management

Content lifecycle management. Config: `services/content/config.yaml`

> **Help**: `{ "command": "core playbooks show domain_content-management" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session](../cli/session.md)

---

## Configure Content Source

Edit `services/content/config.yaml`:

```yaml
content:
  sources:
    - name: blog
      path: services/content/blog
      url_pattern: /blog/{slug}
      template: blog-post
      enabled: true
      categories:
        - name: tutorials
          slug: tutorials
          description: "Step-by-step guides"
        - name: announcements
          slug: announcements
          description: "Product updates"
      sitemap:
        enabled: true
        changefreq: weekly
        priority: 0.8
      rss:
        enabled: true
        title: "Blog RSS Feed"
        description: "Latest posts"
        max_items: 20
    - name: documentation
      path: services/content/documentation
      url_pattern: /documentation/{category}/{slug}
      template: documentation
      enabled: true
      sitemap:
        enabled: true
        changefreq: monthly
        priority: 0.9
```

---

## Create Content

Step 1: Create markdown file with frontmatter:

```markdown
---
title: "My First Blog Post"
description: "An introduction to SystemPrompt"
slug: "my-first-post"
kind: "blog"
public: true
tags: ["introduction", "tutorial"]
published_at: "2026-02-01"
---

# My First Blog Post

Welcome to SystemPrompt!

## What You'll Learn

- How to create content
- How to publish it
```

Required frontmatter:

| Field | Type | Required |
|-------|------|----------|
| `title` | string | Yes |
| `description` | string | Yes |
| `slug` | string | Yes |
| `kind` | string | Yes |
| `public` | boolean | Yes |
| `published_at` | date | Yes |
| `author` | string | No |
| `tags` | array | No |

---

## Publish Content

Step 1: Sync to database

{ "command": "cloud sync local content --direction to-db -y" }

Step 2: Run publish job

{ "command": "infra jobs run publish_pipeline" }

Step 3: Verify

{ "command": "core content list --source blog" }
{ "command": "core content show blog/my-first-post" }

---

## Configure Categories

```yaml
sources:
  - name: blog
    categories:
      - name: tutorials
        slug: tutorials
        description: "Step-by-step guides"
        order: 1
      - name: announcements
        slug: announcements
        description: "Product updates"
        order: 2
```

Assign content:

```markdown
---
title: "My Tutorial"
category: tutorials
---
```

{ "command": "core content list --source blog --category tutorials" }

---

## Configure Sitemap

```yaml
sources:
  - name: blog
    sitemap:
      enabled: true
      changefreq: weekly
      priority: 0.8
```

Changefreq options: `always`, `hourly`, `daily`, `weekly`, `monthly`, `yearly`, `never`

{ "command": "infra jobs run publish_pipeline" }

---

## Configure RSS

```yaml
sources:
  - name: blog
    rss:
      enabled: true
      title: "Blog Feed"
      description: "Latest posts"
      max_items: 20
      language: "en"
```

{ "command": "infra jobs run publish_pipeline" }

---

## Search Content

{ "command": "core content search \"getting started\"" }
{ "command": "core content search \"tutorial\" --source blog" }

---

## Validate Content

{ "command": "core content validate" }

---

## Troubleshooting

- Content missing: `{ "command": "cloud sync local content --direction to-db -y" }`, `{ "command": "infra jobs run publish_pipeline" }`
- Sitemap not updating: `{ "command": "infra jobs run publish_pipeline" }`
- Search broken: `{ "command": "infra jobs run publish_pipeline" }`

---

## Quick Reference

| Task | Command |
|------|---------|
| List | `core content list` |
| Show | `core content show <source>/<slug>` |
| Search | `core content search "query"` |
| Sync | `cloud sync local content --direction to-db -y` |
| Publish | `infra jobs run publish_pipeline` |
| Validate | `core content validate` |

---

## Related

-> See [Content Troubleshooting](content-troubleshooting.md)
-> See [Content Service](/documentation/services/content)