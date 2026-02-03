---
title: "Web Content Editing"
description: "Edit web templates, homepage, and static content."
type: build
keywords:
  - web
  - templates
  - homepage
  - editing
category: build
code_references:
  - path: "services/web/templates/"
    description: "HTML templates"
  - path: "services/web/config/"
    description: "Page configuration (YAML)"
---

# Web Content Editing

Edit web templates, homepage, and static content.

> **Help**: `{ "command": "core playbooks show build_web-edit" }`

---

## What to Edit

| Content Type | Location | Job to Publish |
|--------------|----------|----------------|
| Homepage template | `services/web/templates/homepage.html` | `page_prerender` |
| Homepage config | `services/web/config/homepage.yaml` | `page_prerender` |
| Feature pages | `services/web/config/features/*.yaml` | `page_prerender` |
| Blog templates | `services/web/templates/blog-*.html` | `content_prerender` |
| Blog/docs content | `services/content/blog/*.md` | `content_prerender` |
| CSS/JS | `storage/files/css/`, `storage/files/js/` | `copy_extension_assets` |
| Theme/branding | `services/web/config/theme.yaml` | `page_prerender` |

---

## Edit Workflow

### Step 1: Edit the file

Templates use Handlebars syntax. Config uses YAML.

```
services/web/templates/homepage.html    # HTML + {{variables}}
services/web/config/homepage.yaml       # Data for templates
```

### Step 2: Build

```bash
just build
```

### Step 3: Publish

**Homepage or feature page changes:**

```bash
systemprompt infra jobs run page_prerender
```

**Blog/docs template or content changes:**

```bash
systemprompt infra jobs run content_prerender
```

**CSS/JS asset changes:**

```bash
systemprompt infra jobs run copy_extension_assets
```

**Full publish (everything):**

```bash
systemprompt infra jobs run publish_pipeline
```

-> See [Content Publishing](../cli/content-publish.md) for full details.

---

## Template Syntax

Handlebars variables from config:

```html
{{site.homepage.hero.title}}
{{#each site.homepage.features}}
  <div>{{this.title}}</div>
{{/each}}
```

-> See [Web Templates](web-templates.md) for full syntax.

---

## Quick Reference

| Task | Action |
|------|--------|
| Edit homepage content | `services/web/config/homepage.yaml` |
| Edit homepage layout | `services/web/templates/homepage.html` |
| Edit feature page | `services/web/config/features/<name>.yaml` |
| Build after edits | `just build` |
| Publish homepage/features | `systemprompt infra jobs run page_prerender` |
| Publish blog/docs | `systemprompt infra jobs run content_prerender` |
| Publish CSS/JS | `systemprompt infra jobs run copy_extension_assets` |
| Publish everything | `systemprompt infra jobs run publish_pipeline` |

---

## Related

-> See [Web Templates](web-templates.md) for template syntax
-> See [Web Assets](web-assets.md) for CSS/JS
-> See [Content Publishing](../cli/content-publish.md) for publishing workflow
