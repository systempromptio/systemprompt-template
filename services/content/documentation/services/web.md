---
title: "Web Services"
description: "Configure the web interface including branding, navigation, templates, and theme. Control how content is rendered and presented to users."
author: "SystemPrompt Team"
slug: "services/web"
keywords: "web, branding, navigation, theme, templates, rendering"
image: "/files/images/docs/services-web.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Web Services

**TL;DR:** The web service configures everything users see - branding, navigation, templates, and theme. It renders content from the content service using Handlebars templates and applies consistent styling across all pages.

## The Problem

A SystemPrompt application needs a web interface. Blog posts need templates, documentation needs navigation sidebars, and the homepage needs custom sections. Each content type requires different presentation while maintaining consistent branding.

The web service solves this by providing configurable templates and styling. Define your brand colors once, and they apply everywhere. Configure navigation in YAML, and it appears on every page. Templates handle the rendering while you focus on content.

## How Web Works

The web service has three main components:

1. **Configuration** - YAML files defining branding, navigation, and theme
2. **Templates** - Handlebars HTML files that render content
3. **Assets** - CSS, JavaScript, and images served to browsers

When a user requests a page, the web service:
1. Loads the content from the database
2. Selects the appropriate template
3. Renders the template with content and configuration
4. Serves the resulting HTML

## Directory Structure

```
services/web/
├── config.yaml           # Main configuration (branding, server)
├── config/
│   ├── homepage.yaml     # Homepage sections
│   ├── navigation.yaml   # Header, footer, docs sidebar
│   ├── theme.yaml        # Colors, typography, spacing
│   └── features/         # Feature page configurations
└── templates/
    ├── blog-post.html    # Blog article template
    ├── blog-list.html    # Blog listing template
    ├── docs-page.html    # Documentation page template
    ├── docs-list.html    # Documentation index template
    ├── homepage.html     # Homepage template
    └── partials/         # Reusable template components
```

## Main Configuration

Configure basic settings in `services/web/config.yaml`:

```yaml
branding:
  site_name: "SystemPrompt"
  tagline: "Production infrastructure for AI agents"
  logo: "/files/images/logo.svg"
  favicon: "/files/images/favicon.ico"

server:
  host: "0.0.0.0"
  port: 3000
```

## Navigation

Configure navigation in `config/navigation.yaml`:

<details>
<summary>Navigation configuration</summary>

```yaml
header:
  items:
    - id: documentation
      label: "Documentation"
      href: "/documentation"
      dropdown: true
      sections:
        - title: "Getting Started"
          links:
            - label: "Installation"
              href: "/documentation/installation"

footer:
  legal:
    - path: "/legal/privacy-policy"
      label: "Privacy Policy"

docs_sidebar:
  - title: "Getting Started"
    links:
      - label: "Installation"
        href: "/documentation/installation"
```

</details>

The `docs_sidebar` controls the left sidebar on documentation pages. Add sections and links to organize your documentation hierarchy.

## Theme Configuration

Customize visual styling in `config/theme.yaml`:

```yaml
colors:
  primary: "#0d9488"
  secondary: "#6366f1"
  background: "#0f0f0f"
  text: "#e5e7eb"

typography:
  font_family: "Inter, sans-serif"
  heading_font: "Space Grotesk, sans-serif"
  base_size: "16px"

spacing:
  container_max_width: "1280px"
  section_padding: "4rem"
```

Colors are used throughout templates via CSS variables. Change them here, and the entire site updates.

## Homepage Configuration

Configure homepage sections in `config/homepage.yaml`:

<details>
<summary>Homepage configuration</summary>

```yaml
hero:
  title: "The production runtime"
  title_highlight: "for agentic products."
  subtitle: "An embeddable Rust library..."
  cta: "Get Started"
  cta_url: "/documentation/installation"

value_props:
  - id: "platform"
    title: "EMBEDDED"
    title_highlight: "AI PLATFORM"
    features:
      - "Complete stack in one binary"
      - "Production auth built in"
```

</details>

Each section (hero, value props, features, pricing, FAQ) can be customized or disabled.

## Templates

Templates use Handlebars syntax for dynamic content:

```html
<!DOCTYPE html>
<html>
<head>
  <title>{{TITLE}}</title>
  {{> head-assets}}
</head>
<body>
  {{> header}}
  <main>{{{CONTENT}}}</main>
  {{> footer}}
</body>
</html>
```

Template variables come from content frontmatter and configuration:
- `{{TITLE}}` - Page title from frontmatter
- `{{DESCRIPTION}}` - Meta description
- `{{{CONTENT}}}` - Rendered markdown (triple braces for unescaped HTML)
- `{{SLUG}}` - URL slug
- `{{> partial}}` - Include a partial template

## Template Types

| Template | Purpose |
|----------|---------|
| `docs-page.html` | Individual documentation pages |
| `docs-list.html` | Documentation section indexes |
| `blog-post.html` | Individual blog articles |
| `blog-list.html` | Blog listing pages |
| `homepage.html` | The main landing page |
| `playbook-post.html` | Playbook pages |

## Partials

Partials are reusable template components in `templates/partials/`:

| Partial | Purpose |
|---------|---------|
| `header.html` | Site header with navigation |
| `footer.html` | Site footer |
| `head-assets.html` | CSS and meta tags |
| `scripts.html` | JavaScript includes |

Include partials with `{{> partial-name}}`.

## Service Relationships

The web service connects to:

- **Content service** - Provides content to render
- **Config service** - Included through the aggregation pattern
- **Assets** - CSS and JavaScript files in `storage/files/css/`

The web service is the final presentation layer. It takes content from the content service and configuration from the config service to produce the user interface.

## Validation

Validate web configuration with the CLI:

```bash
# Validate all configuration
systemprompt web validate

# Validate templates only
systemprompt web validate --templates

# Validate assets only
systemprompt web validate --assets
```

## CLI Reference

| Command | Description |
|---------|-------------|
| `systemprompt web content-types` | Manage content types |
| `systemprompt web templates` | Manage templates |
| `systemprompt web assets` | List and inspect assets |
| `systemprompt web sitemap` | Sitemap operations |
| `systemprompt web validate` | Validate web configuration |

See `systemprompt web <command> --help` for detailed options.

## Troubleshooting

**Styles not applying** -- Check that CSS files are in `storage/files/css/` and registered in the web extension. Rebuild with `just build`.

**Navigation not updating** -- Verify the YAML syntax in `navigation.yaml`. Changes may require an application restart.

**Template errors** -- Check for missing Handlebars variables or unclosed tags. The error log shows the template and line number.