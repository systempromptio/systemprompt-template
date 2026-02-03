---
title: "Analytics Playbook"
description: "View metrics, traffic analysis, bot detection, and cost tracking."
author: "SystemPrompt"
slug: "cli-analytics"
keywords: "analytics, metrics, costs, performance, traffic, tracking, bots, sessions"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Analytics Playbook

View metrics, traffic analysis, bot detection, and cost tracking.

---

## Dashboard Overview

```json
{ "command": "analytics overview" }
{ "command": "analytics overview --since 24h" }
```

---

## Traffic Analytics

```json
{ "command": "analytics traffic sources" }
{ "command": "analytics traffic geo --limit 10" }
{ "command": "analytics traffic devices" }
{ "command": "analytics traffic bots" }
```

---

## Session Analytics

```json
{ "command": "analytics sessions stats" }
{ "command": "analytics sessions trends" }
{ "command": "analytics sessions live --limit 20" }
```

---

## Content Performance

```json
{ "command": "analytics content stats" }
{ "command": "analytics content top --limit 10" }
{ "command": "analytics content trends" }
{ "command": "analytics content popular --limit 20" }
```

---

## Cost Analytics

```json
{ "command": "analytics costs summary" }
{ "command": "analytics costs summary --days 30" }
{ "command": "analytics costs trends --group-by day" }
{ "command": "analytics costs breakdown --by model" }
{ "command": "analytics costs breakdown --by agent" }
```

---

## Agent Analytics

```json
{ "command": "analytics agents stats" }
{ "command": "analytics agents trends --days 7" }
{ "command": "analytics agents show <name>" }
{ "command": "analytics agents list" }
```

---

## Tool Usage Analytics

```json
{ "command": "analytics tools stats" }
{ "command": "analytics tools trends" }
{ "command": "analytics tools show <tool-name>" }
```

---

## AI Request Analytics

```json
{ "command": "analytics requests stats" }
{ "command": "analytics requests list" }
{ "command": "analytics requests list --limit 50 --model claude" }
{ "command": "analytics requests trends" }
{ "command": "analytics requests models" }
```

---

## Conversation Analytics

```json
{ "command": "analytics conversations stats" }
{ "command": "analytics conversations trends" }
{ "command": "analytics conversations list" }
```

---

## Common Flags

| Flag | Description |
|------|-------------|
| `--since` | Time range: `1h`, `24h`, `7d`, `30d`, or ISO datetime |
| `--until` | End time for range |
| `--export` | Export to CSV file |
| `--json` | Output as JSON |
| `--group-by` | Group by: `hour`, `day`, `week` |

---

## Tracking System Architecture

### Data Flow

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

## Session Tracking

Session tracking happens automatically via `AnalyticsMiddleware`. No client-side code required.

Every visitor session captures:

| Field | Description |
|-------|-------------|
| `referrer_source` | Parsed source type: `direct`, `organic_search`, `referral`, `social` |
| `referrer_url` | Full referrer URL |
| `landing_page` | First page visited |
| `entry_url` | Entry URL with query params |

### UTM Parameter Tracking

| Parameter | Field | Example |
|-----------|-------|---------|
| `utm_source` | `utm_source` | `google`, `newsletter`, `twitter` |
| `utm_medium` | `utm_medium` | `cpc`, `email`, `social` |
| `utm_campaign` | `utm_campaign` | `spring_sale`, `product_launch` |

### Geographic & Device Data

Automatically captured per session: country, region, city, device_type, browser, os, ip_address, user_agent.

---

## Engagement Tracking

### API Endpoints

| Endpoint | Purpose |
|----------|---------|
| `POST /api/v1/engagement` | Single engagement event |
| `POST /api/v1/engagement/batch` | Batch engagement events |

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

**Filtered Views:** All analytics views exclude bots:
```sql
WHERE is_bot = false
  AND is_behavioral_bot = false
```

---

## Session vs Visitor

| Concept | Definition | How Counted |
|---------|------------|-------------|
| **Session** | Single browsing session (JWT cookie lifetime) | `COUNT(session_id)` |
| **Visitor** | Unique device/browser | `COUNT(DISTINCT fingerprint_hash)` |
| **User** | Registered account | `COUNT(DISTINCT user_id) WHERE user_id IS NOT NULL` |

The fingerprint uniquely identifies a visitor device/browser combination (computed from user-agent + accept-language).

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

| View | Purpose |
|------|---------|
| `v_top_referrer_sources` | Sessions grouped by referrer |
| `v_utm_campaign_performance` | UTM parameter analysis |
| `v_traffic_source_quality` | Quality scoring by source |
| `v_link_performance` | Link performance metrics |
| `v_campaign_performance` | Campaign-level analytics |
| `v_content_journey` | User flow between content |
| `v_bot_traffic_summary` | Bot activity by date |
| `v_behavioral_bot_analysis` | Behavioral detection results |
| `v_clean_human_traffic` | Filtered human-only data |

---

## Diagnosing Issues

### Inflated Visitor Counts

**Check 1: User-agent analysis**
```sql
SELECT user_agent, COUNT(*)
FROM user_sessions
WHERE is_bot = false AND started_at > NOW() - INTERVAL '24 hours'
GROUP BY user_agent
ORDER BY 2 DESC
LIMIT 20
```

**Check 2: Request count distribution**
```sql
SELECT request_count, COUNT(*)
FROM user_sessions
WHERE is_bot = false AND started_at > NOW() - INTERVAL '24 hours'
GROUP BY request_count
ORDER BY 1
```

**Check 3: Fingerprint vs user_id count**
```sql
SELECT
    COUNT(DISTINCT fingerprint_hash) as unique_visitors,
    COUNT(DISTINCT user_id) as distinct_users,
    COUNT(*) as total_sessions
FROM user_sessions
WHERE is_bot = false AND started_at > NOW() - INTERVAL '24 hours'
```

### Retroactive Bot Flagging

Flag existing sessions as bots:
```sql
UPDATE user_sessions
SET is_bot = true
WHERE user_agent LIKE '%python-httpx%'
  AND is_bot = false;
```

---

## Troubleshooting

**Events not recording** -- check browser console for JS errors, verify API endpoint accessible, check for ad-blockers.

**Bot traffic in reports** -- run `analytics traffic bots`, check bot flags, use `v_clean_human_traffic` view.

**Missing UTM data** -- verify URL parameters are properly formatted, check landing page middleware.

**"99% human visitors" but counts too high** -- undetected bots slipping through. Check user-agent patterns in bot detection configuration.

**Sessions with 0 requests** -- normal for single page views, suspicious if majority of sessions have 0.

**No data showing** -- check date range with `--since` flag. Data may not exist for the period.

**Permission denied** -- check profile permissions with `admin session show`.

---

## Quick Reference

| Task | Command |
|------|---------|
| Overview | `analytics overview` |
| Traffic sources | `analytics traffic sources` |
| Bot traffic | `analytics traffic bots` |
| Session stats | `analytics sessions stats` |
| Live sessions | `analytics sessions live` |
| Top content | `analytics content top` |
| Content trends | `analytics content trends` |
| Cost summary | `analytics costs summary` |
| Cost breakdown | `analytics costs breakdown` |
| Agent stats | `analytics agents stats` |
| Tool usage | `analytics tools stats` |
| Request stats | `analytics requests stats` |
| Export data | `analytics overview --export file.csv` |