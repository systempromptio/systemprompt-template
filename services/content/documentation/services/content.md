---
title: "Content Services"
description: "Configure content sources for blog, documentation, playbooks, and legal pages. Set up indexing, categories, and automatic sitemap generation."
author: "SystemPrompt Team"
slug: "services/content"
keywords: "content, sources, categories, indexing, sitemap, blog, documentation"
image: "/files/images/docs/services-content.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Content Services

**TL;DR:** The content service manages all your site's content - blog posts, documentation, playbooks, and legal pages. Each content source is defined with its path, category, branding, and sitemap settings. The content service indexes files and makes them available through the web interface and API.

## The Problem

A SystemPrompt application typically has multiple types of content: blog posts for announcements, documentation for guides, playbooks for agent instructions, and legal pages for compliance. Each type needs different handling - different URL patterns, different templates, different SEO settings.

The content service solves this by defining content sources. Each source specifies where content lives, how it should be categorized, and how it appears in sitemaps. The indexing system reads markdown files, parses frontmatter, and stores content in the database for fast retrieval.

## How Content Works

Content flows through several stages:

1. **Authoring** - Create markdown files with YAML frontmatter in the source directory
2. **Indexing** - The content service reads files and parses metadata
3. **Storage** - Content is stored in the database for fast querying
4. **Rendering** - The web service renders content using templates
5. **Sitemap** - URLs are included in the generated sitemap

Each content source can have different settings for all these stages.

## Configuration

Configure content sources in `services/content/config.yaml`:

<details>
<summary>Content source configuration</summary>

```yaml
# services/content/config.yaml
content_sources:
  blog:
    path: "content/blog"
    source_id: "blog"
    category_id: "blog"
    enabled: true
    description: "Blog articles and announcements"
    allowed_content_types: ["article", "tutorial", "announcement"]
    branding:
      name: "Blog"
      description: "Articles and updates"
      image: "/images/blog/og-default.png"
      keywords: "blog, articles, tutorials"
    indexing:
      clear_before: false
      recursive: true
    sitemap:
      enabled: true
      url_pattern: "/blog/{slug}"
      priority: 0.8
      changefreq: "weekly"
```

</details>

## Content Sources

Each content source defines a type of content:

| Source | Path | Content Types | URL Pattern |
|--------|------|---------------|-------------|
| `blog` | content/blog | article, tutorial, announcement | `/blog/{slug}` |
| `documentation` | content/documentation | guide, reference, docs | `/documentation/{slug}` |
| `legal` | content/legal | page | `/legal/{slug}` |
| `playbooks` | content/playbooks | playbook | `/playbooks/{slug}` |
| `skills` | skills | skill | `/skills/{slug}` |

Add new sources to handle custom content types with their own branding and URL patterns.

## Content Frontmatter

Each markdown file requires frontmatter that describes the content:

```yaml
---
title: "My Post Title"
description: "Post description for SEO"
slug: "my-post"
kind: "blog"
public: true
tags: ["tag1", "tag2"]
published_at: "2026-01-30"
---
```

The `kind` field must match one of the `allowed_content_types` for the source. Set `public: false` to draft content without publishing.

## Branding Configuration

Each source can have custom branding for SEO and social sharing:

```yaml
branding:
  name: "Documentation"
  description: "Guides and reference documentation"
  image: "/files/images/logo.png"
  keywords: "docs, documentation, guides"
```

This branding appears in meta tags and Open Graph tags when content from this source is shared.

## Indexing Configuration

Control how the content service reads and stores content:

```yaml
indexing:
  clear_before: false    # Clear existing before re-indexing
  recursive: true        # Index subdirectories
  override_existing: true # Update existing content
```

Set `clear_before: true` to remove old content when re-indexing. This is useful during development but be careful in production.

## Sitemap Configuration

Configure how content appears in the generated sitemap:

```yaml
sitemap:
  enabled: true
  url_pattern: "/blog/{slug}"
  priority: 0.8
  changefreq: "weekly"
  fetch_from: "database"
  parent_route:
    enabled: true
    url: "/blog"
    priority: 0.9
    changefreq: "daily"
```

The `changefreq` values are: `always`, `hourly`, `daily`, `weekly`, `monthly`, `yearly`, `never`.

## Publishing Content

Sync content to the database with the CLI:

```bash
# Publish all content
systemprompt core content publish

# Publish specific source
systemprompt core content publish --source blog

# List indexed content
systemprompt core content list --source documentation
```

The scheduler can also run content publishing automatically on a schedule.

## Service Relationships

The content service connects to:

- **Web service** - Renders content using templates
- **Playbooks service** - Stores playbook content
- **Scheduler** - Publishes content on a schedule
- **Config service** - Included through the aggregation pattern

## Categories

Define categories to organize content:

```yaml
categories:
  blog:
    name: "Blog"
    slug: "blog"
  documentation:
    name: "Documentation"
    slug: "documentation"
  playbooks:
    name: "Playbooks"
    slug: "playbooks"
    description: "Step-by-step guides"
```

Categories help organize the content library and can be used for filtering.

## Content-File Associations

Link files to content with specific roles. This is essential for featured images, attachments, and inline media.

**CRITICAL:** For featured images to display on pages, you must do BOTH:
1. Link the file to content (`content files link`)
2. Set the `image` field on the content record (`content edit --set image=...`)

The `content files link` command creates the association for file management. The `image` field is what templates use for display.

### Linking Files to Content

```bash
# Link a file as featured image
systemprompt core content files link <file_id> --content <content_id> --role featured

# ALSO set the image field for display
systemprompt core content edit <content_id> --set image="<public_url>"

# Link as attachment (no image field needed)
systemprompt core content files link <file_id> --content <content_id> --role attachment
```

### Available Roles

| Role | Description |
|------|-------------|
| `featured` | Main featured image for the content |
| `og-image` | Open Graph image for social sharing |
| `thumbnail` | Preview/thumbnail image |
| `inline` | Embedded in content body |
| `attachment` | Downloadable attachment |

### Managing Featured Images

```bash
# Get current featured image
systemprompt core content files featured <content_id>

# Set featured image (file must be linked first)
systemprompt core content files featured <content_id> --set <file_id>

# List all files attached to content
systemprompt core content files list --content <content_id>

# Unlink a file
systemprompt core content files unlink <file_id> --content <content_id>
```

### AI-Generated Images

Generate featured images using the content-manager MCP server:

```bash
systemprompt plugins mcp call content-manager generate_featured_image -a '{
  "skill_id": "blog_image_generation",
  "topic": "Your Topic",
  "title": "Content Title",
  "summary": "Brief description for image generation"
}' --timeout 120
```

The generated image is automatically stored in the files system. Link it to content using the returned `image_id`.

See the [Images Playbook](/playbooks/content-images) for complete image management workflows.

## CLI Reference

| Command | Description |
|---------|-------------|
| `systemprompt core content list` | List content with pagination |
| `systemprompt core content show <id>` | Show content details |
| `systemprompt core content search <query>` | Search content |
| `systemprompt core content edit <id>` | Edit content fields |
| `systemprompt core content delete <id>` | Delete content by ID |
| `systemprompt core content delete-source <source>` | Delete all content from a source |
| `systemprompt core content popular` | Get popular content |
| `systemprompt core content verify <id>` | Verify content is published and accessible |
| `systemprompt core content status <source>` | Show content health status for a source |
| `systemprompt core content link` | Link generation and management |
| `systemprompt core content analytics` | Content analytics |
| `systemprompt core content files` | Content-file operations (link, unlink, featured) |

See `systemprompt core content <command> --help` for detailed options.

## Troubleshooting

**Content not appearing** -- Run `core content publish` and check that `public: true` is set in frontmatter. Verify the source is enabled.

**Wrong URL pattern** -- Check the `url_pattern` in sitemap configuration. The `{slug}` placeholder is replaced with the content slug.

**Indexing errors** -- Check for YAML syntax errors in frontmatter. Ensure required fields are present.