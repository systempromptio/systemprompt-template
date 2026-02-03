---
title: "Image Management Playbook"
description: "Generate AI images and associate them with blog content. Complete guide covering generation, linking, and the publish workflow."
keywords:
  - images
  - ai-generation
  - featured-images
  - content-files
  - gemini
  - openai
category: content
---

# Image Management Playbook

Generate AI images and associate them with blog content. This playbook covers the complete workflow from generation to display.

## Prerequisites

**Load the [Session Playbook](../cli/session.md) first.**

```bash
systemprompt admin session show
```

Verify image providers are configured:
```bash
systemprompt admin config show
```

Look for `ai.providers.gemini.enabled: true` or `ai.providers.openai.enabled: true`.

---

## CRITICAL: Two-Step Image Association

**Images require TWO operations to display on pages:**

1. **Link file to content** - Creates the content_files association (for file management)
2. **Set image field on content** - Sets the `image` URL in content metadata (for display)

Missing either step will result in placeholder images showing on the page.

---

## Quick Reference

| Task | Command |
|------|---------|
| Generate image | `plugins mcp call content-manager generate_featured_image -a '{...}'` |
| List files | `core files list` |
| Link to content | `core content files link <file_id> --content <content_id> --role featured` |
| Set image field | `core content edit <content_id> --set image="<public_url>"` |
| Republish | `infra jobs run publish_pipeline` |

---

# Complete Workflow: Add Image to Blog Post

Follow these steps in order. Do not skip any step.

## Step 1: Generate the Image

```bash
systemprompt plugins mcp call content-manager generate_featured_image -a '{
  "skill_id": "blog_image_generation",
  "topic": "Your Topic Here",
  "title": "Your Blog Title",
  "summary": "Brief description for image generation"
}' --timeout 120
```

**Save from the response:**
- `Image ID` (e.g., `27e47153-3acb-4685-88d9-b690d5200ba5`)
- `Public URL` (e.g., `/files/images/generated/2026/02/02/fece2027-b1d7.png`)

## Step 2: Find the Content ID

```bash
systemprompt core content list --source blog
```

Find your blog post and note its `id`.

## Step 3: Link File to Content

```bash
systemprompt core content files link <file_id> --content <content_id> --role featured
```

Example:
```bash
systemprompt core content files link 27e47153-3acb-4685-88d9-b690d5200ba5 \
  --content 7ed8c2cc-e4c5-41df-9ec5-334e3bbe8c6c \
  --role featured
```

## Step 4: Set Image Field on Content (REQUIRED FOR DISPLAY)

```bash
systemprompt core content edit <content_id> --set image="<public_url>"
```

Example:
```bash
systemprompt core content edit 7ed8c2cc-e4c5-41df-9ec5-334e3bbe8c6c \
  --set image="/files/images/generated/2026/02/02/fece2027-b1d7.png"
```

**This step is required.** The blog template uses the `image` field from content metadata, not the content_files association.

## Step 5: Republish

```bash
systemprompt infra jobs run publish_pipeline
```

## Step 6: Verify

Check the page in your browser. The image should now display.

```bash
systemprompt core content show <content_id>
```

Verify the `image` field is set in the response.

---

# File-Based vs Database-Only Content

| Content Type | Where to Set Image |
|--------------|-------------------|
| **File-based** (on disk) | Edit frontmatter `image:` field in markdown file |
| **Database-only** (AI-generated) | Use `core content edit --set image="..."` |

## File-Based Content (On Disk)

For blog posts stored in `services/content/blog/<slug>/index.md`:

1. Edit the markdown file's frontmatter:
   ```yaml
   ---
   title: "Your Blog Title"
   image: "/files/images/generated/2026/02/02/your-image.png"
   ---
   ```

2. Re-run publish pipeline:
   ```bash
   systemprompt infra jobs run publish_pipeline
   ```

## Database-Only Content (AI-Generated)

For blog posts created via MCP tools (no file on disk):

1. Use the CLI to set the image field:
   ```bash
   systemprompt core content edit <content_id> --set image="/files/images/generated/..."
   ```

2. Re-run publish pipeline:
   ```bash
   systemprompt infra jobs run publish_pipeline
   ```

---

# Image Generation Details

## MCP Tool Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `skill_id` | Image generation skill | `blog_image_generation` |
| `topic` | Main subject | `"Discord Bot Security"` |
| `title` | Image title context | `"Building a Secure Discord Bot"` |
| `summary` | Description for prompt | `"Enterprise security with bot automation"` |

## Response Fields

| Field | Description |
|-------|-------------|
| `Image ID` | File UUID for linking |
| `Public URL` | URL path for the image (use this for `--set image=`) |
| `Resolution` | Image resolution (typically 2K) |
| `Aspect Ratio` | Typically 16:9 |

## Supported Providers

| Provider | Models | Resolution | Aspect Ratios |
|----------|--------|------------|---------------|
| Gemini | `gemini-2.5-flash-image`, `gemini-3-pro-image-preview` | 1K, 2K, 4K | Square, 16:9, 9:16, 4:3, 3:4, UltraWide |
| OpenAI | `dall-e-3`, `dall-e-2` | 1K | Square, 16:9, 9:16 |

---

# File Roles

When linking files to content, use these roles:

| Role | Purpose |
|------|---------|
| `featured` | Main featured image for content |
| `og-image` | Open Graph social sharing image |
| `thumbnail` | Thumbnail/preview image |
| `inline` | Image embedded in content body |
| `attachment` | Downloadable attachment |

---

# Image Storage

Generated images are stored in:
```
storage/files/images/generated/YYYY/MM/DD/<uuid>_<timestamp>.png
```

Accessible via URL:
```
/files/images/generated/YYYY/MM/DD/<uuid>_<timestamp>.png
```

## Image Optimization (WebP)

For production, convert PNG images to WebP for 90%+ size reduction:

```bash
# Using Python PIL (if available)
python3 << 'EOF'
from PIL import Image
import os

img_dir = "storage/files/images/generated/2026/02/02"
for f in os.listdir(img_dir):
    if f.endswith('.png'):
        img = Image.open(f"{img_dir}/{f}")
        img.save(f"{img_dir}/{f.replace('.png', '.webp')}", 'WEBP', quality=85)
        print(f"Converted: {f}")
EOF
```

Then update image paths in content to use `.webp` extension.

---

# Syncing Content to Disk

After generating images and associating them with database content, export to disk for version control:

```bash
systemprompt infra jobs run content_sync -p direction=to-disk -p source=blog
```

This creates markdown files in `services/content/blog/` with proper frontmatter including image paths.

See [Sync Playbook](../cli/sync.md) for more sync options.

---

# Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Placeholder image showing | `image` field not set on content | Run `core content edit <id> --set image="..."` |
| "No image providers available" | AI provider not configured | Check `admin config show` for enabled providers |
| "Failed to connect to MCP server" | Services not running | Run `just start` |
| Image generated but not linked | Missing link step | Run `core content files link ...` |
| Changes not appearing | Publish pipeline not run | Run `infra jobs run publish_pipeline` |

---

# Full Example: New Blog Post with Image

```bash
# 1. Generate the image
systemprompt plugins mcp call content-manager generate_featured_image -a '{
  "skill_id": "blog_image_generation",
  "topic": "AI Agent Architecture",
  "title": "Building Multi-Agent Systems",
  "summary": "Complex orchestration of AI agents working together"
}' --timeout 120

# Response:
# Image ID: abc12345-def6-7890-abcd-ef1234567890
# Public URL: /files/images/generated/2026/02/02/abc12345-def6.png

# 2. Find content ID
systemprompt core content list --source blog
# Found: id = xyz98765-4321-...

# 3. Link file to content
systemprompt core content files link abc12345-def6-7890-abcd-ef1234567890 \
  --content xyz98765-4321-... \
  --role featured

# 4. Set image field (CRITICAL!)
systemprompt core content edit xyz98765-4321-... \
  --set image="/files/images/generated/2026/02/02/abc12345-def6.png"

# 5. Republish
systemprompt infra jobs run publish_pipeline

# 6. Verify
systemprompt core content show xyz98765-4321-...
# Check that "image" field is set
```

---

## Related Playbooks

- [Blog Playbook](./blog.md) - Blog content creation
- [Session Playbook](../cli/session.md) - Authentication
- [Jobs Playbook](../cli/jobs.md) - Job management
