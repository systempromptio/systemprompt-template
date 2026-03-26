
# LinkedIn Content Manager

## Overview

This skill manages the LinkedIn content lifecycle for Foodles:

| Feature | Description | Template |
|---------|-------------|----------|
| Content Calendar | Visual month/week view of scheduled posts | `templates/calendar.html` |
| Analytics Dashboard | Post performance metrics and insights | `templates/analytics.html` |
| Post Editor | Create and edit LinkedIn posts | `templates/editor.html` |
| Content Library | Browse and manage all content | `templates/library.html` |

All templates use the Foodles brand system with glassmorphism design and bento-box layout.

---

## Quick Start

### View Content Calendar
Open `templates/calendar.html` in Canvas to display the content calendar.

### View Analytics
Open `templates/analytics.html` to see performance metrics.

### Create New Post
Use `templates/editor.html` for drafting new content.

---

## Data Structure

### posts.json

```json
{
  "id": "post_001",
  "title": "Post title for internal reference",
  "content": "The actual LinkedIn post text...",
  "status": "draft|scheduled|published|archived",
  "scheduled_date": "2026-02-15T10:00:00Z",
  "published_date": null,
  "hashtags": ["#Odoo", "#ERP", "#Foodles"],
  "content_type": "text|image|carousel|video|article",
  "media_urls": [],
  "metrics": {
    "impressions": 0,
    "reactions": 0,
    "comments": 0,
    "shares": 0,
    "clicks": 0
  },
  "notes": "Internal notes about this post"
}
```

### campaigns.json

```json
{
  "id": "campaign_001",
  "name": "Q1 Product Launch",
  "start_date": "2026-01-01",
  "end_date": "2026-03-31",
  "goals": ["Brand awareness", "Lead generation"],
  "post_ids": ["post_001", "post_002"],
  "status": "active|completed|paused"
}
```

---

## Content Guidelines

### LinkedIn Best Practices (Foodles Voice)

1. **Hook in first line**: Capture attention immediately
2. **Value-first approach**: Share expertise, not sales pitches
3. **Conversational "tu"**: Professional but approachable
4. **No emojis**: Following Foodles brand guidelines strictly
5. **Strategic hashtags**: 3-5 relevant hashtags maximum
6. **Call to action**: End with engagement prompt when appropriate

### Optimal Posting Times

| Day | Best Times (CET) |
|-----|------------------|
| Tuesday | 10:00, 12:00 |
| Wednesday | 09:00, 10:00 |
| Thursday | 10:00, 14:00 |

### Content Mix (Monthly)

| Type | Percentage | Examples |
|------|------------|----------|
| Educational | 40% | How-to guides, tips, insights |
| Case Studies | 25% | Client success stories |
| Industry News | 20% | Odoo updates, tech trends |
| Company Culture | 15% | Team, events, behind-scenes |

---

## Integration

### With content-collection

Reference Foodles case studies and videos:
```markdown
1. Read `skills/content-collection/references/cases-index.md`
2. Select relevant case for LinkedIn post
3. Adapt content following brand voice
```

### With foodles-brand

All generated content must comply with:
- Professional "tu" address
- Zero emojis
- Value vocabulary (no "problema", "coste", "gasto")
- Long-form explanatory style when needed

---

## Templates

### Design System

All templates follow:
- **Colors**: Warm Yellow, Blue Lilac, Blue Space, Light Sky
- **Typography**: Dosis (Google Fonts CDN)
- **Framework**: Bootstrap 5.3
- **Style**: Glassmorphism + Bento box layout
- **Themes**: Light/Dark toggle (user preference saved)
- **Animations**: Subtle CSS transitions

### Template Files

| File | Purpose |
|------|---------|
| `templates/calendar.html` | Content calendar with month/week views |
| `templates/analytics.html` | Performance dashboard with charts |
| `templates/editor.html` | Post creation and editing |
| `templates/library.html` | Content library browser |

### CSS Variables

```css
:root {
  --foodles-warm-yellow: #E5B92B;
  --foodles-blue-lilac: #6B68FA;
  --foodles-blue-space: #1C265D;
  --foodles-light-sky: #8AC2DB;
}
```

---

## Scripts

| Script | Purpose |
|--------|---------|
| `scripts/fetch_metrics.py` | Fetch LinkedIn post metrics via API |
| `scripts/schedule_post.py` | Schedule a post for publishing |
| `scripts/generate_report.py` | Generate weekly/monthly reports |

---

## Workflow

### Content Creation

1. **Ideation**: Review content-collection for inspiration
2. **Draft**: Create post in editor template
3. **Review**: Apply brand voice checklist
4. **Schedule**: Set optimal date/time
5. **Publish**: Manual or automated posting
6. **Analyze**: Review metrics after 48-72h

### Weekly Tasks

- Monday: Review previous week metrics
- Tuesday: Schedule 2-3 posts for the week
- Wednesday: Engage with comments/reactions
- Thursday: Content ideation for next week
- Friday: Update content calendar

---

## Notes

- Templates are standalone HTML files viewable in Canvas
- Data files are JSON for easy editing
- Scripts require LinkedIn API credentials (not included)
- All content must pass Foodles brand compliance before publishing
