---
title: "Web Templates"
description: "Handlebars templates and theme configuration."
keywords:
  - templates
  - handlebars
  - themes
  - branding
category: build
---

# Web Templates

Handlebars templates and theme configuration.

> **Help**: `{ "command": "core playbooks show build_web-templates" }`

---

## Page Types

Two patterns exist for web pages:

| Pattern | Template Variables |
|---------|-------------------|
| Content (markdown) | `{{TITLE}}`, `{{DESCRIPTION}}`, `{{CONTENT}}` |
| Configured (YAML) | `{{site.homepage.*}}`, `{{feature.*}}` |

-> See [Web Pages Architecture](web-pages.md) for when to use each.

---

## Template Location

```
services/web/templates/
├── templates.yaml      # Template-to-content-type mapping
├── homepage.html       # Homepage
├── blog-post.html      # Individual blog posts
├── blog-list.html      # Blog listing page
├── docs-page.html      # Documentation pages
├── docs-list.html      # Documentation index
└── legal-post.html     # Legal pages
```

> **Never** create templates in `web/templates/`. Only `services/web/templates/`.

---

## templates.yaml

Maps content `kind` values to templates:

```yaml
templates:
  homepage:
    content_types:
      - homepage
  blog-post:
    content_types:
      - blog
      - article
      - paper
      - guide
      - tutorial
  legal-post:
    content_types:
      - legal
      - page
```

---

## Handlebars Syntax

| Syntax | Purpose |
|--------|---------|
| `{{VAR}}` | Escaped output |
| `{{{VAR}}}` | Raw HTML output |
| `{{#if VAR}}...{{/if}}` | Conditional |
| `{{#each items}}...{{/each}}` | Loop |
| `{{../name}}` | Parent context in nested loop |
| `{{this.field}}` | Current item in loop |

---

## Template Variables

### Frontmatter Mapping

| Frontmatter | Variable |
|-------------|----------|
| `title` | `TITLE` |
| `description` | `DESCRIPTION` |
| `author` | `AUTHOR` |
| `slug` | `SLUG` |
| `keywords` | `KEYWORDS` |
| `image` | `IMAGE` |
| `published_at` | `DATE`, `DATE_ISO`, `DATE_PUBLISHED` |
| `updated_at` | `DATE_MODIFIED`, `DATE_MODIFIED_ISO` |

### Site Variables (from config.yaml)

| Variable | Source |
|----------|--------|
| `ORG_NAME` | `branding.name` |
| `ORG_URL` | `branding.url` |
| `ORG_LOGO` | `branding.logo.primary.svg` |
| `TWITTER_HANDLE` | `branding.twitter_handle` |
| `DISPLAY_SITENAME` | `branding.display_sitename` |

### Rendered Content

| Variable | Description |
|----------|-------------|
| `CONTENT` | Markdown → HTML |
| `TOC_HTML` | Table of contents |
| `RELATED_CONTENT` | Related articles |
| `FOOTER_NAV` | Footer navigation |

---

## Theme Configuration

`services/web/config.yaml`:

### Required Branding Fields

| Field | Example |
|-------|---------|
| `branding.copyright` | `"© 2024 Company"` |
| `branding.twitter_handle` | `"@handle"` |
| `branding.display_sitename` | `true` |
| `branding.favicon` | `"/favicon.ico"` |
| `branding.logo.primary.svg` | `"/logo.svg"` |

### Colors

```yaml
colors:
  light:
    primary:
      hsl: "hsl(0, 0%, 35%)"
    text:
      primary: "#111111"
      secondary: "#666666"
  dark:
    primary:
      hsl: "hsl(0, 0%, 60%)"
    text:
      primary: "#FFFFFF"
```

### Typography

```yaml
typography:
  sizes:
    xs: "12px"
    sm: "14px"
    md: "15px"
    lg: "18px"
  weights:
    regular: 400
    medium: 500
    bold: 700
```

---

## Adding a New Content Type

1. **Define kind in frontmatter**:
   ```yaml
   kind: case-study
   ```

2. **Add to content source** (`services/content/config.yaml`):
   ```yaml
   content_sources:
     case-studies:
       path: services/content/case-studies
       allowed_content_types:
         - case-study
   ```

3. **Map to template** (`templates.yaml`):
   ```yaml
   templates:
     case-study-page:
       content_types:
         - case-study
   ```

4. **Create template file**: `services/web/templates/case-study-page.html`

---

## Troubleshooting

| Error | Solution |
|-------|----------|
| No template for content type | Add to `templates.yaml` |
| Missing branding config | Add to `services/web/config.yaml` |
| Variable not rendering | Check frontmatter field name |

---

## Quick Reference

| Task | Location |
|------|----------|
| Templates | `services/web/templates/*.html` |
| Template mapping | `services/web/templates/templates.yaml` |
| Theme config | `services/web/config.yaml` |
| SEO metadata | `services/web/metadata.yaml` |

-> See [Web Content](web-content.md) for creating content.
-> See [Web Assets](web-assets.md) for CSS/JS/images.
