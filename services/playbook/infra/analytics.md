---
title: "Analytics Infrastructure Playbook"
description: "Understanding the analytics system, querying data, diagnosing issues, and maintaining accurate metrics."
keywords:
  - analytics
  - metrics
  - sessions
  - bots
  - visitors
---

# Analytics Infrastructure Playbook

This document explains the analytics infrastructure, how to query production data, diagnose issues, and ensure accurate visitor metrics.

> **Help**: `{ "command": "analytics" }` via `systemprompt_help`
> **Related**: [Analytics CLI Playbook](../cli/analytics.md) | [Tracking Playbook](../analytics/tracking.md)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Web Request                                 │
└─────────────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     Session Middleware                               │
│  1. Extract analytics (IP, user-agent, fingerprint, geo)            │
│  2. Check if bot (keywords, IP ranges, datacenter, risk country)    │
│  3. If bot: skip tracking (session_id = "bot_xxx", tracked=false)   │
│  4. If human: find/create session, set JWT cookie                   │
└─────────────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     user_sessions table                              │
│  - session_id, user_id, fingerprint_hash                            │
│  - is_bot, is_scanner, is_behavioral_bot                            │
│  - device_type, browser, os, country                                │
│  - referrer_source, utm_source, landing_page                        │
└─────────────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     Analytics Queries                                │
│  - Homepage stats: COUNT(DISTINCT fingerprint_hash) for visitors    │
│  - Bot filtering: WHERE is_bot = false AND is_behavioral_bot = false│
│  - Unique visitors: By fingerprint, not user_id                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Key Concepts

### Fingerprint Hash

The fingerprint uniquely identifies a visitor device/browser combination:
- Computed from: user-agent + accept-language (server-side)
- Can be enhanced with client-side fingerprinting (x-fingerprint header)
- Used to deduplicate sessions from the same visitor

### Session vs Visitor

| Concept | Definition | How Counted |
|---------|------------|-------------|
| **Session** | Single browsing session (JWT cookie lifetime) | `COUNT(session_id)` |
| **Visitor** | Unique device/browser | `COUNT(DISTINCT fingerprint_hash)` |
| **User** | Registered account | `COUNT(DISTINCT user_id) WHERE user_id IS NOT NULL` |

### Bot Detection Layers

| Layer | Flag | Detection Method |
|-------|------|------------------|
| User-Agent | `is_bot` | Known bot keywords (googlebot, python-httpx, etc.) |
| Scanner | `is_scanner` | Suspicious paths (.env, wp-admin, etc.) |
| Behavioral | `is_behavioral_bot` | Pattern analysis (high velocity, page coverage) |

---

## Querying Production Data

### Switch to Production Profile

```bash
just cli admin session switch production
```

### Basic Analytics Commands

```bash
# Overview stats
just cli analytics overview

# Session statistics
just cli analytics sessions stats

# Bot traffic analysis
just cli analytics traffic bots

# Content performance
just cli analytics content stats
```

### Direct Database Queries

For complex analysis, query the database directly:

```bash
# Unique visitors in last 24h (correct method)
just cli infra db query "SELECT COUNT(DISTINCT fingerprint_hash) FROM user_sessions WHERE is_bot = false AND is_behavioral_bot = false AND is_scanner = false AND started_at > NOW() - INTERVAL '24 hours'"

# Session breakdown by bot flags
just cli infra db query "SELECT is_bot, is_scanner, is_behavioral_bot, COUNT(*) FROM user_sessions WHERE started_at > NOW() - INTERVAL '24 hours' GROUP BY 1,2,3"

# Top user agents (check for undetected bots)
just cli infra db query "SELECT user_agent, COUNT(*) as cnt FROM user_sessions WHERE is_bot = false AND started_at > NOW() - INTERVAL '24 hours' GROUP BY user_agent ORDER BY cnt DESC LIMIT 20"
```

---

## Diagnosing Issues

### Inflated Visitor Counts

**Symptoms**: Visitor count much higher than expected

**Check 1: User-agent analysis**
```bash
just cli infra db query "SELECT user_agent, COUNT(*) FROM user_sessions WHERE is_bot = false AND started_at > NOW() - INTERVAL '24 hours' GROUP BY user_agent ORDER BY 2 DESC LIMIT 20"
```

Look for:
- `python-*` libraries (should be flagged)
- Empty user-agents
- Old browser versions (Firefox 47, Chrome 80)
- Generic/template user-agents

**Check 2: Request count distribution**
```bash
just cli infra db query "SELECT request_count, COUNT(*) FROM user_sessions WHERE is_bot = false AND started_at > NOW() - INTERVAL '24 hours' GROUP BY request_count ORDER BY 1"
```

Suspicious pattern: Most sessions with 0 requests = likely bots doing single-page hits

**Check 3: Fingerprint vs user_id count**
```bash
just cli infra db query "SELECT COUNT(DISTINCT fingerprint_hash) as unique_visitors, COUNT(DISTINCT user_id) as distinct_users, COUNT(*) as total_sessions FROM user_sessions WHERE is_bot = false AND started_at > NOW() - INTERVAL '24 hours'"
```

If `distinct_users >> unique_visitors`, the display is counting sessions not visitors

### Missing Bot Detection

**Add new patterns** to `crates/domain/analytics/src/services/bot_keywords.rs`:

```rust
pub const BOT_KEYWORDS: &[&str] = &[
    // ... existing patterns
    "new-bot-keyword",
];
```

**Verify detection**:
```bash
# Check if pattern would be matched
grep -i "suspicious-agent" bot_keywords.rs
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

## Key Files

| File | Purpose |
|------|---------|
| `crates/domain/analytics/src/services/bot_keywords.rs` | Bot detection patterns |
| `crates/domain/analytics/src/services/extractor.rs` | Analytics extraction and bot checks |
| `crates/domain/analytics/src/repository/core_stats/overview.rs` | Homepage stats queries |
| `crates/entry/api/src/services/middleware/session.rs` | Session creation middleware |
| `crates/entry/api/src/services/middleware/bot_detector.rs` | Request-level bot detection |

---

## Homepage Stats Query

The homepage displays visitor counts from `get_user_metrics_with_trends()`:

```sql
SELECT
    COUNT(DISTINCT fingerprint_hash) FILTER (WHERE last_activity_at > cutoff_24h) as "count_24h",
    COUNT(DISTINCT fingerprint_hash) FILTER (WHERE last_activity_at > cutoff_7d) as "count_7d",
    COUNT(DISTINCT fingerprint_hash) FILTER (WHERE last_activity_at > cutoff_30d) as "count_30d"
FROM user_sessions
WHERE is_bot = false AND is_behavioral_bot = false AND is_scanner = false
```

**Key points**:
- Uses `fingerprint_hash` not `user_id` for unique visitors
- Filters all three bot flags
- Time windows based on `last_activity_at`

---

## Common Issues

### "99% human visitors" but counts too high

The bot percentage is calculated correctly, but undetected bots still slip through. Check:
1. User-agent patterns not in bot_keywords.rs
2. Empty/missing user-agents
3. Datacenter IPs not in detection list

### Sessions with 0 requests

Normal for:
- Single page views (no AJAX calls counted)
- Bots that hit and leave immediately

Suspicious if majority of sessions have 0 requests.

### 7d = 30d visitor count

Indicates either:
- Site launched less than 30 days ago
- Data retention/cleanup issue
- Query bug in time window

---

## Monitoring Checklist

Daily checks:
1. `analytics traffic bots` - Bot percentage should be reasonable
2. `analytics sessions stats` - Check avg requests per session
3. Review top user-agents for new bot patterns

Weekly checks:
1. Compare fingerprint count vs user_id count
2. Review behavioral bot detections
3. Check for new datacenter IP ranges

---

## Quick Reference

| Task | Command |
|------|---------|
| Switch to production | `just cli admin session switch production` |
| Overview stats | `just cli analytics overview` |
| Bot analysis | `just cli analytics traffic bots` |
| Session stats | `just cli analytics sessions stats` |
| Top user-agents | `just cli infra db query "SELECT user_agent, COUNT(*) FROM user_sessions WHERE is_bot=false GROUP BY 1 ORDER BY 2 DESC LIMIT 20"` |
| Unique visitors | `just cli infra db query "SELECT COUNT(DISTINCT fingerprint_hash) FROM user_sessions WHERE is_bot=false AND started_at > NOW() - INTERVAL '24h'"` |
