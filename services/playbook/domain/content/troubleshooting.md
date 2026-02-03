---
title: "Content Troubleshooting"
description: "Diagnose and fix content issues: sync failures, missing content, search problems, rendering errors."
keywords:
  - content
  - troubleshooting
  - debug
  - sync
  - search
category: domain
---

# Content Troubleshooting

Diagnose and fix content issues. Config: `services/content/config.yaml`

> **Help**: `{ "command": "core playbooks show domain_content-troubleshooting" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session](../cli/session.md)

---

## Diagnostic Checklist

{ "command": "core content list" }
{ "command": "core content validate" }
{ "command": "cloud sync local content --direction to-db --dry-run" }
{ "command": "infra logs --limit 50" }

---

## Issue: Content Not Appearing

Symptoms: File exists but not in list, 404 errors

Step 1: Check synced

{ "command": "core content list --source blog" }

Step 2: Verify file exists

```bash
ls -la services/content/blog/
```

Step 3: Check frontmatter

```bash
head -50 services/content/blog/my-post.md
```

Step 4: Validate

{ "command": "core content validate" }

Solutions:

Not synced:

{ "command": "cloud sync local content --direction to-db -y" }
{ "command": "infra jobs run publish_pipeline" }

Frontmatter invalid: Add required fields:

```markdown
---
title: "My Post"
description: "Description"
slug: "my-post"
kind: "blog"
public: true
published_at: "2026-02-01"
---
```

Not public:

```yaml
public: true
```

---

## Issue: Content Sync Fails

Symptoms: Sync errors, changes not reflected

Step 1: Dry run

{ "command": "cloud sync local content --direction to-db --dry-run" }

Step 2: Validate

{ "command": "core content validate" }

Step 3: Check file

```bash
cat services/content/blog/my-post.md
```

Solutions:

YAML frontmatter error:

```markdown
---
title: "Title with: colon"
description: "Description"
slug: "valid-slug"
---
```

Database error:

{ "command": "infra db status" }
{ "command": "infra services restart db" }

---

## Issue: Search Not Finding Content

Symptoms: Content exists but search returns empty

Step 1: Verify indexed

{ "command": "core content search \"known text\"" }

Step 2: Check publish ran

{ "command": "infra jobs status publish_pipeline" }

Step 3: View logs

{ "command": "infra logs --context content --limit 50" }

Solutions:

Re-index:

{ "command": "infra jobs run publish_pipeline" }
{ "command": "core content search \"text\"" }

Content not public: Check `public: true` in frontmatter

---

## Issue: Sitemap Not Updating

Symptoms: New content not in sitemap

Step 1: Check config

```bash
cat services/content/config.yaml | grep -A5 sitemap
```

Step 2: Run publish

{ "command": "infra jobs run publish_pipeline" }

Step 3: Check sitemap

```bash
curl http://localhost:8080/sitemap.xml
```

Solutions:

Enable sitemap:

```yaml
sources:
  - name: blog
    sitemap:
      enabled: true
      changefreq: weekly
      priority: 0.8
```

Regenerate:

{ "command": "infra jobs run publish_pipeline" }

---

## Issue: RSS Feed Not Generating

Symptoms: Feed returns 404, empty, wrong content

Step 1: Check config

```bash
cat services/content/config.yaml | grep -A5 rss
```

Step 2: Run publish

{ "command": "infra jobs run publish_pipeline" }

Step 3: Check feed

```bash
curl http://localhost:8080/blog/feed.xml
```

Solutions:

Enable RSS:

```yaml
sources:
  - name: blog
    rss:
      enabled: true
      title: "Blog Feed"
      description: "Latest posts"
      max_items: 20
```

---

## Issue: Template Rendering Errors

Symptoms: Page shows error, partial rendering

Step 1: View error logs

{ "command": "infra logs --context web --level error" }

Step 2: Check template exists

```bash
ls services/web/templates/
```

Step 3: Validate content

{ "command": "core content validate" }

Solutions:

Template not found: Check config template name:

```yaml
sources:
  - name: blog
    template: blog-post
```

Template variable missing: Add required frontmatter fields

---

## Issue: Categories Not Working

Symptoms: Category pages empty, wrong content

Step 1: List by category

{ "command": "core content list --source blog --category tutorials" }

Step 2: Check category config

```bash
cat services/content/config.yaml | grep -A10 categories
```

Step 3: Check frontmatter

```bash
grep "category:" services/content/blog/*.md
```

Solutions:

Define categories:

```yaml
sources:
  - name: blog
    categories:
      - name: tutorials
        slug: tutorials
```

Assign content:

```markdown
---
title: "My Tutorial"
category: tutorials
---
```

---

## Validation Errors

{ "command": "core content validate" }

| Error | Cause | Fix |
|-------|-------|-----|
| Missing field | Frontmatter incomplete | Add field |
| Invalid date | Wrong format | Use "YYYY-MM-DD" |
| Invalid slug | Spaces/special chars | Use lowercase-with-dashes |
| YAML syntax | Formatting issue | Fix indentation |

---

## Quick Reference

| Problem | First Command |
|---------|---------------|
| Missing | `core content list` |
| Sync fails | `core content validate` |
| Search broken | `infra jobs run publish_pipeline` |
| Sitemap empty | `infra jobs run publish_pipeline` |
| RSS missing | Check `rss.enabled: true` |
| Template error | `infra logs --context web --level error` |
| Any issue | `core content validate` |

---

## Related

-> See [Content Management](content-management.md)
-> See [Content Service](/documentation/services/content)
