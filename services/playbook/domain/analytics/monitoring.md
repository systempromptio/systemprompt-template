---
title: "Analytics Monitoring"
description: "Monitor usage metrics, costs, sessions, and bot detection."
author: "SystemPrompt"
slug: "domain-analytics-monitoring"
keywords: "analytics, monitoring, metrics, costs, sessions"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Analytics Monitoring

Analytics and monitoring workflows. Config: `services/config/analytics.yaml`

> **Help**: `{ "command": "core playbooks show domain_analytics-monitoring" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session](../cli/session.md)

---

## Usage Overview

{ "command": "analytics summary" }
{ "command": "analytics summary --period hour" }
{ "command": "analytics summary --period day" }
{ "command": "analytics summary --period week" }
{ "command": "analytics summary --period month" }

---

## AI Provider Usage

{ "command": "analytics ai --period day" }
{ "command": "analytics ai --provider anthropic --period week" }
{ "command": "analytics ai --provider openai --period week" }
{ "command": "analytics ai --detail model --period day" }

---

## Agent Activity

{ "command": "analytics agents --period day" }
{ "command": "analytics agents --agent welcome --period week" }

---

## Sessions

{ "command": "analytics sessions --period day" }
{ "command": "analytics sessions --active" }

---

## Content Performance

{ "command": "analytics content --period week" }
{ "command": "analytics content --source blog --period week" }
{ "command": "analytics content --source documentation --period week" }

---

## Bot Detection

{ "command": "analytics bots --period day" }
{ "command": "analytics bots --suspicious" }
{ "command": "admin users banned" }
{ "command": "admin users unban 192.168.1.100" }

---

## Export Data

{ "command": "analytics export --format json --period week > analytics.json" }
{ "command": "analytics export --format csv --period week > analytics.csv" }

---

## Alerts

Configure in profile:

```yaml
analytics:
  alerts:
    daily_cost_limit: 50.00
    hourly_request_limit: 1000
```

{ "command": "analytics alerts --period week" }

---

## Configuration

`services/config/analytics.yaml`:

```yaml
analytics:
  enabled: true
  retention_days: 90
  tracking:
    sessions: true
    requests: true
    ai_usage: true
    content: true
    bots: true
  bot_detection:
    enabled: true
    block_threshold: 10
    monitoring_window: 3600
  aggregation:
    interval: 3600
```

---

## Time Periods

| Flag | Period |
|------|--------|
| `--period hour` | Last hour |
| `--period day` | Last 24 hours |
| `--period week` | Last 7 days |
| `--period month` | Last 30 days |

---

## Quick Reference

| Task | Command |
|------|---------|
| Summary | `analytics summary` |
| AI usage | `analytics ai --period day` |
| Agent stats | `analytics agents --period day` |
| Sessions | `analytics sessions --period day` |
| Content | `analytics content --period week` |
| Bots | `analytics bots --period day` |
| Export | `analytics export --format json` |

---

## Related

-> See [Analytics Service](/documentation/services/analytics)