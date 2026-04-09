---
title: "Metrics Reference"
description: "Complete reference of all dashboard metrics, data structures, and aggregations captured by the systemprompt.io AI governance platform."
author: "systemprompt.io"
slug: "metrics-reference"
keywords: "metrics, dashboard data, analytics, structs, aggregations, time series"
kind: "guide"
public: true
tags: ["analytics", "metrics", "reference"]
published_at: "2026-03-25"
updated_at: "2026-03-25"
after_reading_this:
  - "Know every metric captured by the admin dashboard"
  - "Understand the data structures behind each dashboard component"
  - "Query and filter metrics by time range, department, and user"
related_docs:
  - title: "Dashboard"
    url: "/documentation/dashboard"
  - title: "Activity Tracking"
    url: "/documentation/activity-tracking"
  - title: "Cost Tracking"
    url: "/documentation/cost-tracking"
  - title: "CLI Analytics"
    url: "/documentation/cli-analytics"
---

# Metrics Reference

This page documents every metric captured by the platform dashboard, the data structures that power them, and how they are aggregated.

## Stats Ribbon

The stats ribbon displays 11 key metrics at the top of the dashboard:

| Metric | Field | Type | Description |
|--------|-------|------|-------------|
| Events Today | `events_today` | i64 | Total activity events recorded today |
| Events This Week | `events_this_week` | i64 | Total events in the current week |
| Total Sessions | `total_sessions` | i64 | AI sessions in the selected time range |
| Error Count | `error_count` | i64 | Tool failures and errors |
| Tool Uses | `tool_uses` | i64 | Total tool invocations |
| Prompts | `prompts` | i64 | User prompt submissions |
| Subagents Spawned | `subagents_spawned` | i64 | Subagent processes started |
| Input Tokens | `total_input_tokens` | i64 | Total input tokens consumed |
| Output Tokens | `total_output_tokens` | i64 | Total output tokens generated |
| Total Cost (USD) | `total_cost_usd` | f64 | Total AI cost in US dollars |
| Failure Count | `failure_count` | i64 | Failed operations count |

These metrics are computed by `get_activity_stats()` from the `plugin_usage_events` table.

## Usage Time Series

The area chart shows 5 metrics over time in hourly buckets:

| Metric | Field | Description |
|--------|-------|-------------|
| Sessions | `sessions` | AI sessions started per hour |
| Active Users | `active_users` | Distinct users per hour |
| Prompts | `prompts` | Prompts submitted per hour |
| Tool Uses | `tool_uses` | Tool invocations per hour |
| Errors | `errors` | Errors per hour |

Each bucket is a `TimeSeriesBucket` with a `bucket` timestamp (DateTime) and the five metric counts. The dashboard supports three time ranges: 24 hours, 7 days (default), and 14 days.

## Top Users

The top users table ranks the 10 most active users:

| Column | Field | Type | Description |
|--------|-------|------|-------------|
| User ID | `user_id` | String | Unique user identifier |
| Display Name | `display_name` | String | User's display name |
| Logins | `logins` | i64 | Login count in the time range |
| Prompts | `prompts` | i64 | Prompts submitted |
| Plugins | `plugins` | i64 | Active plugins |
| Tokens | `tokens` | i64 | Total tokens consumed |
| Last Active | `last_active` | DateTime | Most recent activity timestamp |

## Popular Skills

Skills ranked by invocation count:

| Field | Type | Description |
|-------|------|-------------|
| `tool_name` | String | Skill or tool name |
| `count` | i64 | Total invocations |

The dashboard displays the top 10 skills as a horizontal bar chart.

## Tool Success Rates

Per-tool reliability metrics:

| Field | Type | Description |
|-------|------|-------------|
| `tool_name` | String | Tool identifier |
| `total` | i64 | Total invocations |
| `successes` | i64 | Successful completions |
| `failures` | i64 | Failed invocations |
| `success_pct` | f64 | Success percentage (0-100) |

## Model Usage

Distribution of AI model usage across the platform:

| Field | Type | Description |
|-------|------|-------------|
| `model` | String | Model identifier (e.g., "claude-opus-4-6") |
| `count` | i64 | Number of requests using this model |

## Department Activity

Activity counts aggregated by department:

| Field | Type | Description |
|-------|------|-------------|
| `department` | String | Department name |
| `count` | i64 | Total events from this department |

## Hourly Activity

24-hour activity distribution showing usage patterns:

| Field | Type | Description |
|-------|------|-------------|
| `hour` | i32 | Hour of day (0-23) |
| `count` | i64 | Events in that hour |

## Event Type Breakdown

Distribution of events across Claude Code event types:

| Field | Type | Description |
|-------|------|-------------|
| `event_type` | String | Event type (e.g., `claude_code_PostToolUse`) |
| `count` | i64 | Number of events of this type |

## Project Activity

Activity grouped by project/repository:

| Field | Type | Description |
|-------|------|-------------|
| `project_path` | String | Full project path |
| `project_name` | String | Project directory name |
| `event_count` | i64 | Total events for this project |
| `session_count` | i64 | Sessions opened in this project |

## Daily Aggregations

The platform automatically rolls up raw events into daily aggregation tables for fast querying:

**`plugin_usage_daily`** — One row per (date, user, plugin, event_type, tool_name):
- `event_count` — Number of events
- `total_duration_ms` — Total duration in milliseconds
- `total_input_tokens` — Input tokens consumed
- `total_output_tokens` — Output tokens generated
- `error_count` — Errors encountered

**`plugin_session_summaries`** — One row per session:
- `started_at` / `ended_at` — Session boundaries
- `total_events` — Events in the session
- `tool_uses` / `prompts` / `errors` — Categorised counts
- `total_input_tokens` / `total_output_tokens` — Token consumption
- `model` — AI model used

## Query Parameters

Dashboard metrics support the following query parameters:

| Parameter | Values | Default | Description |
|-----------|--------|---------|-------------|
| `range` | `24h`, `7d`, `14d` | `7d` | Time range for all metrics |
| `dept` | Department name | *(all)* | Filter by department |
