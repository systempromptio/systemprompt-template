---
title: "Web Development Playbook"
description: "Guide for content, themes, templates, and static site generation."
keywords:
  - web
  - content
  - themes
  - templates
---

# Web Development Guide

This guide covers how to add content, customize themes, manage assets, and build the static website within the SystemPrompt platform.

> **Help**: `{ "command": "playbook build" }` via `systemprompt_help`

---

## Architecture Overview

The web system uses a **static site generation** approach:

1. **Content** is written in Markdown with YAML frontmatter
2. **Templates** render content using Handlebars syntax
3. **Themes** are configured via YAML and generate CSS variables
4. **Assets** (images, fonts, JS) are served from `/storage/files/`
5. **Prerendering** generates static HTML from database content

```
services/content/        → Markdown files
services/web/           → Theme config + templates
extensions/blog/assets/ → CSS/JS source files
storage/files/          → Runtime assets (images, fonts, css, js)
```

---

## Adding Content

### Content Location

All content lives in `services/content/`:

```
services/content/
├── config.yaml          # Content source definitions
├── blog/               # Blog articles (*.md)
├── legal/              # Legal pages (*.md)
└── skills/             # Agent skills (*.md)
```

### Markdown Frontmatter

Every markdown file requires YAML frontmatter:

```yaml
---
title: "Article Title"
description: "Brief description for SEO (150-160 chars)"
author: "Author Name"
slug: "url-friendly-slug"
keywords: "comma, separated, keywords"
kind: "article"          # article | paper | guide | tutorial | page
image: "/files/images/blog/featured-image.webp"
public: true
tags: []
published_at: "2025-12-11"
updated_at: "2026-01-13"
links:                   # Optional reference links
  - title: "Reference Name"
    url: "https://example.com"
---

# Article Title

Your markdown content here...
```

### Content Types (kind)

| Kind | Description | Use For |
|------|-------------|---------|
| `article` | Standard blog post | Most blog content |
| `paper` | Technical deep-dive | Research, whitepapers |
| `guide` | Step-by-step tutorial | How-to guides |
| `tutorial` | Educational content | Learning materials |
| `page` | Static page | Legal, about, contact |
| `legal` | Legal document | Use for legal content source |

> **Important**: The `kind` value must match what's configured in `templates.yaml`. Use `legal` for content in the `legal/` source.

### Required Frontmatter Fields

All fields below are **required** for content to publish successfully:

| Field | Required | Notes |
|-------|----------|-------|
| `title` | Yes | Page/article title |
| `description` | Yes | SEO description (150-160 chars) |
| `author` | Yes | Author name |
| `slug` | Yes | URL-friendly identifier |
| `keywords` | Yes | Can be empty string |
| `image` | **Yes** | Must be set, use placeholder if no image |
| `kind` | Yes | Must match templates.yaml mapping |
| `public` | Yes | Set to `true` to publish |
| `published_at` | Yes | ISO date format |
| `updated_at` | Yes | ISO date format |

> **Common Issue**: Empty `image: ""` will cause publish to fail. Always use a placeholder: `image: "/files/images/blog/placeholder.svg"`

### Creating a New Blog Post

1. Create file: `services/content/blog/my-article.md`
2. Add frontmatter with required fields
3. Write markdown content
4. Publish: `systemprompt core content publish`

### URL Structure

URLs are determined by the content source configuration:

| Source | URL Pattern | Example |
|--------|-------------|---------|
| blog | `/blog/{slug}` | `/blog/my-article` |
| legal | `/legal/{slug}` | `/legal/privacy-policy` |

---

## Theme Customization

### Configuration File

Theme settings live in `services/web/config.yaml`. This YAML generates CSS variables at build time.

### Branding

```yaml
branding:
  name: "tyingshoelaces"
  title: "tyingshoelaces | AI Agent Platform"
  description: "Open-source agent orchestration platform"
  themeColor: "#404040"
  display_sitename: false
  twitter_handle: "@tyingshoelaces_"

  logo:
    primary:
      svg: "/assets/logos/logo.svg"
      webp: "/assets/logos/logo.webp"
      png: "/assets/logos/logo.png"
    dark:
      png: "/assets/logos/logo-dark.png"
    small:
      png: "/assets/logos/logo-512.png"

  favicon: "/assets/logos/logo.svg"
```

### Color Palette

```yaml
colors:
  light:
    primary:
      hsl: "hsl(0, 0%, 35%)"
    text:
      primary: "#111111"
      secondary: "#666666"
    background:
      default: "hsl(0, 0%, 100%)"

  dark:
    primary:
      hsl: "hsl(0, 0%, 60%)"
    text:
      primary: "#FFFFFF"
      secondary: "#CCCCCC"
    background:
      default: "#1A1A1A"
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

## Templates

### Template Location

```
services/web/templates/
├── templates.yaml      # Template-to-content-type mapping
├── blog-post.html      # Individual blog posts
├── blog-list.html      # Blog listing page
├── homepage.html       # Homepage
└── legal-post.html     # Legal pages
```

### templates.yaml Configuration

The `templates.yaml` file maps content `kind` values to templates:

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
  blog-list:
    content_types:
      - blog-list
```

> **Important**: If a content `kind` isn't mapped here, prerender will fail silently.

### Handlebars Syntax

```handlebars
<!-- Variable substitution -->
<title>{{TITLE}} | {{ORG_NAME}}</title>

<!-- Raw HTML (no escaping) -->
{{{CONTENT}}}

<!-- Conditionals -->
{{#if FEATURED_IMAGE}}
<img src="{{FEATURED_IMAGE}}" alt="{{TITLE}}" />
{{/if}}

<!-- Loops -->
{{#each items}}
  <li>{{this.title}}</li>
{{/each}}
```

### Available Template Variables

**Site/Organization:**
- `ORG_NAME` - Site name
- `ORG_URL` - Base URL
- `ORG_LOGO` - Logo URL

**Content:**
- `TITLE` - Article title
- `DESCRIPTION` - Meta description
- `AUTHOR` - Author name
- `DATE` - Formatted publish date
- `SLUG` - URL slug
- `READ_TIME` - Estimated read time

**Rendered Content:**
- `CONTENT` - Rendered markdown HTML
- `TOC_HTML` - Table of contents
- `RELATED_CONTENT` - Related articles HTML

---

## CLI Content Workflow

### Content Management

```bash
# List content
systemprompt core content list --source blog --limit 20

# Show content details
systemprompt core content show my-article-slug --source blog

# Search content
systemprompt core content search "MCP server"

# Delete content
systemprompt core content delete <id> --yes
```

### Publishing Pipeline

```bash
# Run full publishing pipeline (all steps)
systemprompt core content publish

# Or run individual steps
systemprompt core content publish --step ingest     # Parse markdown to DB
systemprompt core content publish --step assets     # Copy CSS/JS to web/dist
systemprompt core content publish --step prerender  # Generate HTML pages
systemprompt core content publish --step homepage   # Generate index.html
systemprompt core content publish --step sitemap    # Generate sitemap.xml
```

---

## Complete Publishing Workflow

### 1. Create Content

```bash
cat > services/content/blog/my-new-post.md << 'EOF'
---
title: "My New Post"
description: "A brief description"
author: "Author Name"
slug: "my-new-post"
keywords: "keyword1, keyword2"
kind: "article"
image: "/files/images/blog/my-image.webp"
public: true
published_at: "2026-01-15"
---

# My New Post

Content goes here...
EOF
```

### 2. Add Images

```bash
systemprompt files upload ./my-image.png
```

### 3. Publish

```bash
systemprompt core content publish
```

### 4. Verify

```bash
# Check generated files
ls -la web/dist/
ls -la web/dist/blog/my-new-post/

# Start server and browse
just start
# Visit http://localhost:8080/blog/my-new-post
```

---

## Quick Reference

| Task | Command |
|------|---------|
| **Full publish** | `systemprompt core content publish` |
| Homepage only | `systemprompt core content publish --step homepage` |
| Ingest only | `systemprompt core content publish --step ingest` |
| Prerender only | `systemprompt core content publish --step prerender` |
| List content | `systemprompt core content list --source <source>` |
| Search | `systemprompt core content search "<query>"` |
| Upload image | `systemprompt core files upload <path>` |

## Output Structure

After publishing, files are generated in `web/dist/`:

```
web/dist/
├── index.html              # Homepage
├── sitemap.xml             # Sitemap
├── feed.xml                # RSS feed
├── css/                    # Stylesheets
├── js/                     # Scripts
├── blog/
│   ├── index.html          # Blog list
│   └── {slug}/index.html   # Individual posts
└── legal/
    └── {slug}/index.html   # Legal pages
```
