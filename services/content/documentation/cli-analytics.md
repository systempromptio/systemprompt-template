---
title: "CLI Analytics"
description: "Complete reference for the systemprompt analytics CLI. Query metrics, costs, sessions, agents, tools, traffic, and requests from the terminal."
author: "systemprompt.io"
slug: "cli-analytics"
keywords: "CLI, analytics, command line, systemprompt analytics, CSV export, reporting"
kind: "guide"
public: true
tags: ["analytics", "cli", "reference"]
published_at: "2026-03-25"
updated_at: "2026-03-25"
after_reading_this:
  - "Use the systemprompt analytics CLI to query all platform metrics"
  - "Filter analytics by time range, agent, model, and department"
  - "Export any analytics dataset to CSV for reporting"
related_docs:
  - title: "Dashboard"
    url: "/documentation/dashboard"
  - title: "Metrics Reference"
    url: "/documentation/metrics-reference"
  - title: "Cost Tracking"
    url: "/documentation/cost-tracking"
  - title: "Activity Tracking"
    url: "/documentation/activity-tracking"
---

# CLI Analytics

The `systemprompt analytics` CLI provides programmatic access to all platform metrics. Every analytics view available in the dashboard is also accessible from the terminal with filtering, sorting, and CSV export.

## Standard Flags

All analytics commands support these flags:

| Flag | Description | Example |
|------|-------------|---------|
| `--since` | Start of time range | `--since 24h`, `--since 7d`, `--since 30d` |
| `--until` | End of time range | `--until 2026-03-20` |
| `--export` | Export results to CSV file | `--export report.csv` |
| `--help` | Command-specific help | `systemprompt analytics costs --help` |

## Dashboard & Overview

```bash
# High-level metrics dashboard
systemprompt analytics overview

# Dashboard for a specific time range
systemprompt analytics overview --since 7d
```

## Cost Analytics

```bash
# Cost summary across all providers
systemprompt analytics costs summary

# Breakdown by model, agent, or provider
systemprompt analytics costs breakdown --by model
systemprompt analytics costs breakdown --by agent
systemprompt analytics costs breakdown --by provider

# Cost trends over time
systemprompt analytics costs trends --since 7d
```

## Session Analytics

```bash
# Session metrics and duration stats
systemprompt analytics sessions stats

# Session trends over time
systemprompt analytics sessions trends --since 7d

# Real-time active sessions
systemprompt analytics sessions live
```

## Agent Analytics

```bash
# Aggregate agent statistics
systemprompt analytics agents stats

# List agents with metrics
systemprompt analytics agents list

# Sort by specific metric
systemprompt analytics agents list --sort-by cost
systemprompt analytics agents list --sort-by success-rate
systemprompt analytics agents list --sort-by task-count
systemprompt analytics agents list --sort-by last-active

# Deep dive on a specific agent
systemprompt analytics agents show <agent-name>

# Agent usage trends
systemprompt analytics agents trends --since 7d
```

## Tool Analytics

```bash
# Aggregate tool statistics
systemprompt analytics tools stats

# List tools with metrics
systemprompt analytics tools list

# Sort by specific metric
systemprompt analytics tools list --sort-by execution-count
systemprompt analytics tools list --sort-by success-rate
systemprompt analytics tools list --sort-by avg-time

# Deep dive on a specific tool
systemprompt analytics tools show <tool-name>

# Tool usage trends
systemprompt analytics tools trends --since 7d
```

## Conversation Analytics

```bash
# Conversation statistics
systemprompt analytics conversations stats

# Conversation trends
systemprompt analytics conversations trends --since 7d

# List recent conversations
systemprompt analytics conversations list --limit 20
```

## Content Analytics

```bash
# Content engagement statistics
systemprompt analytics content stats

# Top performing content
systemprompt analytics content top --limit 10

# Content trends
systemprompt analytics content trends --since 7d
```

## AI Request Analytics

```bash
# AI request volume and latency
systemprompt analytics requests stats

# List recent AI requests
systemprompt analytics requests list --since 24h

# Request trends
systemprompt analytics requests trends --since 7d

# Model usage breakdown
systemprompt analytics requests models
```

## Traffic Analytics

```bash
# Traffic source breakdown
systemprompt analytics traffic sources

# Geographic distribution
systemprompt analytics traffic geo

# Device and browser breakdown
systemprompt analytics traffic devices

# Bot traffic analysis
systemprompt analytics traffic bots
```

## Common Workflows

### Daily Health Check

```bash
systemprompt analytics overview
systemprompt analytics costs summary
systemprompt analytics traffic bots
```

### Investigate High Costs

```bash
systemprompt analytics costs breakdown --by model
systemprompt analytics costs breakdown --by agent
systemprompt analytics costs trends --since 7d
```

### Check Agent Performance

```bash
systemprompt analytics agents stats
systemprompt analytics agents list --sort-by success-rate
systemprompt analytics tools stats
```

### Generate Leadership Report

```bash
systemprompt analytics overview --since 30d --export monthly-overview.csv
systemprompt analytics costs summary --since 30d --export monthly-costs.csv
systemprompt analytics agents list --since 30d --export monthly-agents.csv
```

## Notes

- Time ranges use shorthand: `1h`, `24h`, `7d`, `30d`
- Cost data may lag behind real-time by a few minutes
- Bot detection results are heuristic-based — review before taking action
- Use `--help` on any subcommand for the full flag reference
