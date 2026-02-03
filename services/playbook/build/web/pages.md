---
title: "Web Pages Architecture"
description: "Content collections vs. configured pages."
keywords:
  - pages
  - content
  - homepage
  - architecture
category: build
---

# Web Pages Architecture

Two patterns for generating web pages.

> **Help**: `{ "command": "core playbooks show build_web-pages" }`

---

## Content Collections (Markdown)

**Use for**: Blog posts, documentation, articles — many items with similar structure.

| Aspect | Description |
|--------|-------------|
| Data source | Markdown files with YAML frontmatter |
| Storage | Database (for search, analytics) |
| Fields | Fixed set: title, description, author, date, keywords, image |
| Template vars | `{{TITLE}}`, `{{DESCRIPTION}}`, `{{CONTENT}}` |
| Example | Blog posts, docs, legal pages |

```yaml
---
title: "My Article"
description: "Article description"
kind: "article"
published_at: "2025-01-15"
---
```

---

## Configured Pages (YAML)

**Use for**: Homepage, feature pages, landing pages — singleton pages with custom structure.

| Aspect | Description |
|--------|-------------|
| Data source | `services/web/config.yaml` |
| Storage | None (config file is source of truth) |
| Fields | Unlimited — any structure you define |
| Template vars | `{{site.homepage.*}}`, `{{feature.*}}` |
| Example | Homepage, feature pages, pricing page |

```yaml
homepage:
  hero:
    title: "Build AI Agents"
    subtitle: "Production infrastructure"
  features:
    - title: "OAuth2 Auth"
      description: "Enterprise-ready"
```

---

## When to Use Which

| Scenario | Pattern |
|----------|---------|
| 100+ blog posts | Content (markdown) |
| 5 feature pages | Configured (YAML) |
| Legal pages | Content (markdown) |
| Homepage | Configured (YAML) |
| Pricing page | Configured (YAML) |
| Documentation | Content (markdown) |

**Rule of thumb**: Collections → Content. Singletons → Configured.

---

## Extension Traits

| Trait | When to Implement |
|-------|-------------------|
| `PagePrerenderer` | To define new page types with output paths |
| `PageDataProvider` | To inject data into existing page types |

### Example: Homepage

- `DefaultHomepagePrerenderer` → defines `/index.html` exists
- `HomepagePageDataProvider` → provides `site.homepage.*` data

### Example: Feature Pages

```rust
impl PagePrerenderer for FeaturePagePrerenderer {
    fn page_type(&self) -> &str { "feature" }

    async fn prepare(&self, ctx: &PagePrepareContext<'_>) -> Result<Option<PageRenderSpec>> {
        // Return one spec per feature page
        Ok(Some(PageRenderSpec::new(
            "feature",
            serde_json::json!({ "feature": feature_data }),
            format!("features/{}/index.html", slug),
        )))
    }
}
```

---

## Creating a Configured Page

1. **Add to config.yaml**:
   ```yaml
   features:
     pages:
       - slug: "mcp-native"
         headline: "MCP Native"
         highlights: ["Point 1", "Point 2"]
   ```

2. **Create extension with PagePrerenderer**:
   -> See [Extension Checklist](extension-checklist.md)

3. **Create template**: `services/web/templates/feature.html`

4. **Map template** (`templates.yaml`):
   ```yaml
   templates:
     feature:
       content_types:
         - feature
   ```

---

## Quick Reference

| Pattern | Data | Fields | Use Case |
|---------|------|--------|----------|
| Content | Markdown | Fixed | Collections |
| Configured | YAML | Custom | Singletons |

-> See [Web Content](web-content.md) for markdown content.
-> See [Web Templates](web-templates.md) for template configuration.
-> See [Homepage Extension](https://github.com/systempromptio/systemprompt-web/tree/main/extensions/homepage) for reference implementation.
