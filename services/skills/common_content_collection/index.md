
# Content Collection

## Overview

This skill provides a pre-packaged catalog of Enterprise Demo content collected from three sources:

| Source | Content | Data File |
|--------|---------|-----------|
| YouTube Channel | @enterprise-demo-odoo videos + thumbnails | `data/videos.json` |
| Odoo Blog | Client case studies / references | `data/cases.json` |
| Odoo Products | Enterprise Demo Core module specifications | `data/modules.json` |
| Website Sitemap | All public URLs categorized by type | `data/sitemap.json` |

Data is collected by running `scripts/collect_all.py` and stored in the skill package.

---

## Content Routing

| Need | Read This |
|------|-----------|
| Find a case study | [references/cases-index.md](references/cases-index.md) |
| Find a module spec | [references/modules-index.md](references/modules-index.md) |
| Find a video | [references/videos-index.md](references/videos-index.md) |
| Find a page for interlinking | [references/sitemap-index.md](references/sitemap-index.md) |
| Get full record details | Load specific item from `data/*.json` |

---

## Search Patterns

### By Keyword

Search across all content for a keyword:
```bash
grep -i "inventory" data/*.json
grep -i "manufacturing" data/cases.json
```

### By Module

Find all content related to a specific module:
```bash
# Cases using the module
grep "enterprise-demo_inventory" data/cases.json

# Videos about the topic
grep -i "inventory" data/videos.json
```

### By Client

Find case study for a specific client:
```bash
grep -i "client_name" data/cases.json
```

---

## Data Schemas

### videos.json

```json
{
  "id": "youtube_video_id",
  "title": "Video Title",
  "description": "Full description...",
  "publish_date": "2024-01-15",
  "duration": "PT12M30S",
  "url": "https://youtube.com/watch?v=...",
  "thumbnail": "assets/thumbnails/VIDEO_ID.jpg",
  "topics": ["odoo", "inventory", "manufacturing"]
}
```

### cases.json

```json
{
  "id": 123,
  "title": "Client Name - Project Title",
  "client": "Client Name",
  "industry": "Manufacturing",
  "url": "https://enterprise-demo.es/blog/...",
  "publish_date": "2024-02-20",
  "summary": "Brief summary...",
  "content": "Full blog post content...",
  "modules_used": ["enterprise-demo_inventory", "enterprise-demo_mrp"]
}
```

### modules.json

```json
{
  "id": 456,
  "name": "Enterprise Demo Inventory",
  "technical_name": "enterprise-demo_inventory",
  "code": "IND-INV",
  "description": "Extended inventory management...",
  "features": ["Feature 1", "Feature 2"],
  "price": 1500.00,
  "url": "https://enterprise-demo.es/shop/...",
  "related_cases": [123, 125],
  "related_videos": ["VIDEO_ID_1", "VIDEO_ID_2"]
}
```

### sitemap.json

```json
{
  "url": "https://www.enterprise-demo.es/blog/blog-enterprise-demo-1/articulo-ejemplo-123",
  "category": "blog",
  "title_hint": "Articulo Ejemplo",
  "lastmod": "2026-01-15",
  "priority": 0.5,
  "changefreq": ""
}
```

Categories: `blog`, `case_study`, `blog_other`, `page`, `service`, `slide`, `shop`, `job`, `event`, `forum`, `helpdesk`

---

## Cross-References

The `build_indexes.py` script automatically creates cross-references:

- **Modules** have `related_cases` and `related_videos` arrays
- Links are built by matching module names/keywords in case content and video topics
- Use these to find all content about a specific module

Example: To find all content about Enterprise Demo Inventory:
1. Read module from `data/modules.json` where `technical_name = "enterprise-demo_inventory"`
2. Use `related_cases` array to fetch case studies
3. Use `related_videos` array to fetch video details

---

## Refresh Workflow

To update the content collection:

```bash
# Set environment variables
export YOUTUBE_API_KEY="your_youtube_api_key"
export ODOO_URL="https://enterprise-demo.es"
export ODOO_DB="enterprise-demo_production"
export ODOO_KEY="your_odoo_api_key"

# Run collection (from skill directory)
cd skills/content-collection
python scripts/collect_all.py

# Or run individual collectors
python scripts/collect_youtube.py
python scripts/collect_cases.py
python scripts/collect_modules.py
python scripts/build_indexes.py

# Re-package the skill
python /path/to/skill-creator/scripts/package_skill.py .
```

### Collection Scripts

| Script | Source | Requirements |
|--------|--------|--------------|
| `collect_youtube.py` | YouTube Data API v3 | `YOUTUBE_API_KEY`, `google-api-python-client` |
| `collect_cases.py` | Odoo Blog | `ODOO_URL`, `ODOO_DB`, `ODOO_KEY`, odoo-pilot |
| `collect_modules.py` | Odoo Products | `ODOO_URL`, `ODOO_DB`, `ODOO_KEY`, odoo-pilot |
| `collect_sitemap.py` | Website sitemap.xml | None (public URL) |
| `build_indexes.py` | Local JSON files | None |
| `collect_all.py` | All sources | All above |

---

## Integration with Other Skills

### For Consumer Skills (e.g., blog-creator)

Reference this skill in your SKILL.md:

```markdown
## Required Skills

- **content-collection**: For case studies, module specs, and videos.
  - Read `references/cases-index.md` to browse case studies
  - Read `references/modules-index.md` to browse modules
  - Load full records from `data/*.json` as needed
```

### With enterprise-demo-brand

When generating content that references this collection:
1. Find relevant content using indexes
2. Load full records from JSON
3. Apply enterprise-demo-brand voice and visual guidelines

---

## Assets

### Thumbnails (`assets/thumbnails/`)

YouTube video thumbnails stored as JPG files named by video ID:
- `VIDEO_ID.jpg` - High quality thumbnail (typically 480x360)

Use in HTML/markdown:
```html
<img src="assets/thumbnails/VIDEO_ID.jpg" alt="Video title">
```

---

## Notes

- Data is pre-packaged in the skill (not fetched at runtime)
- Run `collect_all.py` periodically to refresh content
- Index files are auto-generated; do not edit manually
- Cross-references are rebuilt each time indexes are generated
