---
title: "Analytics Tracking Playbook"
description: "How the analytics system tracks external traffic and on-site behavior."
keywords:
  - analytics
  - tracking
  - sessions
  - engagement
---

# Analytics Tracking System

This document explains how the analytics system tracks external traffic and on-site behavior.

> **Help**: `{ "command": "playbook analytics" }` via `systemprompt_help`

---

## Overview

The analytics system provides comprehensive tracking through two layers:

1. **Session-level tracking** - Automatic HTTP request tracking via middleware (traffic sources, UTM parameters, device info)
2. **Engagement tracking** - Client-side engagement metrics (scroll depth, time on page, reading patterns)

**Note:** Page views are tracked automatically by server middleware. No client-side page view events needed.

---

## External Traffic Tracking

### Session Attribution (Automatic)

Session tracking happens automatically via `AnalyticsMiddleware`. No client-side code required.

Every visitor session captures traffic source data in the `user_sessions` table:

| Field | Description |
|-------|-------------|
| `referrer_source` | Parsed source type: `direct`, `organic_search`, `referral`, `social` |
| `referrer_url` | Full referrer URL |
| `landing_page` | First page visited |
| `entry_url` | Entry URL with query params |

### UTM Parameter Tracking

Marketing campaign parameters are automatically extracted:

| Parameter | Field | Example |
|-----------|-------|---------|
| `utm_source` | `utm_source` | `google`, `newsletter`, `twitter` |
| `utm_medium` | `utm_medium` | `cpc`, `email`, `social` |
| `utm_campaign` | `utm_campaign` | `spring_sale`, `product_launch` |

**Example URL:**
```
https://example.com/blog/post?utm_source=twitter&utm_medium=social&utm_campaign=launch
```

### Geographic & Device Data

Automatically captured per session:

```
Geographic: country, region, city
Device: device_type, browser, os
Network: ip_address, user_agent
```

---

## On-Site Behavior Tracking

### API Endpoints

| Endpoint | Purpose |
|----------|---------|
| `POST /api/v1/engagement` | Single engagement event |
| `POST /api/v1/engagement/batch` | Batch engagement events |

### Engagement Payload

```javascript
{
  page_url: "/blog/post-slug",
  time_on_page_ms: 45000,
  max_scroll_depth: 72,
  click_count: 5,
  time_to_first_interaction_ms: 2300,
  time_to_first_scroll_ms: 1500,
  scroll_velocity_avg: 125,
  reading_pattern: "engaged"
}
```

### Reading Pattern Classification

| Pattern | Criteria |
|---------|----------|
| `bounce` | <10s on page AND <25% scroll |
| `skimmer` | Default fallback |
| `scanner` | >30% scroll AND <20s on page |
| `reader` | >50% scroll AND >15s on page |
| `engaged` | >75% scroll AND >30s on page |

---

## Bot Detection

Three-tier bot detection prevents skewed analytics:

### 1. User-Agent Detection (`is_bot`)

Identifies known bots by user-agent string:
- Search engines: Googlebot, Bingbot, DuckDuckBot
- AI crawlers: ChatGPT-User, Claude-Web, Perplexity
- SEO tools: Ahrefs, SEMrush, Moz

### 2. Scanner Detection (`is_scanner`)

Detects security threats and malicious scanners:
- Requests for `.env`, `.php`, `/wp-admin`, etc.
- High-velocity request patterns
- Known scanner user-agents

### 3. Behavioral Detection (7-Signal System)

| Signal | Description | Threshold |
|--------|-------------|-----------|
| Request Velocity | Requests per minute | >60/min |
| Page Coverage | Unique pages per session | >50 pages |
| Time Between Requests | Consistency analysis | <100ms avg |
| Session Duration | Abnormally long sessions | >4 hours |

Threshold: `behavioral_bot_score >= 50`

**Filtered Views:**

All analytics views exclude bots:
```sql
WHERE is_bot = false
  AND is_behavioral_bot = false
```

---

## CLI Analytics Commands

### Overview

```json
// MCP: systemprompt
{ "command": "analytics overview --since 24h" }
```

### Traffic Analysis

```json
// MCP: systemprompt
{ "command": "analytics traffic sources" }
{ "command": "analytics traffic geo --limit 10" }
{ "command": "analytics traffic devices" }
{ "command": "analytics traffic bots" }
```

### Content Performance

```json
// MCP: systemprompt
{ "command": "analytics content stats" }
{ "command": "analytics content top --limit 10" }
{ "command": "analytics content trends" }
```

### Session Analysis

```json
// MCP: systemprompt
{ "command": "analytics sessions stats" }
{ "command": "analytics sessions trends" }
{ "command": "analytics sessions live --limit 20" }
```

### Cost Tracking

```json
// MCP: systemprompt
{ "command": "analytics costs summary" }
{ "command": "analytics costs trends --group-by day" }
{ "command": "analytics costs breakdown --by model" }
```

### Common Flags

| Flag | Description |
|------|-------------|
| `--since` | Time range: `1h`, `24h`, `7d`, `30d`, or ISO datetime |
| `--until` | End time for range |
| `--export` | Export to CSV file |
| `--json` | Output as JSON |
| `--group-by` | Group by: `hour`, `day`, `week` |

---

## Database Schema

### Core Tables

| Table | Purpose |
|-------|---------|
| `user_sessions` | Session tracking |
| `engagement_events` | Client-side engagement |
| `fingerprint_reputation` | Fingerprint tracking |
| `analytics_events` | HTTP request logging |

### Key Analytics Views

**Traffic Source Analysis:**
- `v_top_referrer_sources` - Sessions grouped by referrer
- `v_utm_campaign_performance` - UTM parameter analysis
- `v_traffic_source_quality` - Quality scoring by source

**Engagement Analysis:**
- `v_link_performance` - Link performance metrics
- `v_campaign_performance` - Campaign-level analytics
- `v_content_journey` - User flow between content

**Bot & Security:**
- `v_bot_traffic_summary` - Bot activity by date
- `v_behavioral_bot_analysis` - Behavioral detection results

---

## Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Frontend/Client                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │ Page Views   │  │ Clicks       │  │ Engagement Metrics       │  │
│  │ (automatic)  │  │ (tracked)    │  │ (scroll, time, focus)    │  │
│  └──────────────┘  └──────────────┘  └──────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
          │                                           │
          ▼                                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                          API Gateway                                 │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                   Analytics Middleware                        │  │
│  │  - Session creation/update                                    │  │
│  │  - Bot detection (known patterns)                             │  │
│  │  - Request logging                                            │  │
│  └──────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
          │                                           │
          ▼                                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        PostgreSQL                                    │
│  user_sessions | engagement_events | fingerprint_reputation          │
└─────────────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        CLI Reporting                                 │
│  systemprompt analytics overview | sessions | traffic | costs        │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Troubleshooting

### Events Not Recording

1. Check browser console for JavaScript errors
2. Verify API endpoint is accessible
3. Check for ad-blockers blocking analytics requests
4. Verify session cookie is set

### Bot Traffic in Reports

1. Run `analytics traffic bots` to see bot activity
2. Check `is_bot`, `is_scanner`, `is_behavioral_bot` flags
3. Use `v_clean_human_traffic` view for filtered data

### Missing UTM Data

1. Verify URL parameters are properly formatted
2. Check that landing page middleware is running
3. Query `user_sessions` directly to see raw data

---

## Quick Reference

| Task | Command |
|------|---------|
| Overview | `analytics overview` |
| Traffic sources | `analytics traffic sources` |
| Top content | `analytics content top` |
| Session stats | `analytics sessions stats` |
| Bot traffic | `analytics traffic bots` |
| Cost summary | `analytics costs summary` |
| Live sessions | `analytics sessions live` |
| Export data | `analytics overview --export file.csv` |
