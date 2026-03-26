---
title: "Dashboard"
description: "Monitor AI usage across your organization with the admin dashboard. View real-time activity, metrics, top users, popular skills, model usage, and department breakdowns."
author: "systemprompt.io"
slug: "dashboard"
keywords: "dashboard, analytics, metrics, usage, monitoring, activity, events, sessions"
kind: "guide"
public: true
tags: ["dashboard", "analytics", "admin"]
published_at: "2026-02-18"
updated_at: "2026-03-19"
after_reading_this:
  - "Understand the metrics displayed on the admin dashboard"
  - "Use time-range controls to analyze AI usage trends"
  - "Identify top users, popular skills, and department activity"
  - "Monitor error rates and system health"
related_docs:
  - title: "Events"
    url: "/documentation/events"
  - title: "Users"
    url: "/documentation/users"
  - title: "Jobs"
    url: "/documentation/jobs"
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "Presentation"
    url: "/documentation/presentation"
---

# Dashboard

**TL;DR:** The dashboard is the admin home page. It shows a metric ribbon with today's key numbers, an AI usage chart with selectable time ranges, live activity feed, system health indicators, top users leaderboard, popular skills, event breakdown, model usage, hourly activity, and department activity. Admin access is required — non-admin users are redirected to their profile page.

> **See this in the presentation:** [Slide 16: Cost Visibility & Analytics](/documentation/presentation#slide-16)

## Access Control

The dashboard is **admin-only**. When a non-admin user navigates to `/admin/`, they are automatically redirected to their own profile page at `/admin/user?id={user_id}`. Admin status is determined by whether the user's roles include `admin`.

## What You'll See

### Metric Ribbon

A horizontal strip across the top of the page shows key statistics at a glance:

| Metric | Description |
|--------|-------------|
| **Events Today** | Total events recorded in the current day |
| **Tool Uses** | Number of tool invocations this week |
| **Prompts** | Number of user prompts submitted this week |
| **Sessions** | Total sessions this week |
| **Active Users** | Distinct users active in the last 24 hours |
| **Errors** | Error count this week (only shown when errors exist, highlighted in red) |
| **Plugins** | Total number of plugins available to your roles |
| **Skills** | Total number of skills across all accessible plugins |

### AI Usage Chart

A multi-series area chart visualizes usage over time. The chart plots five data series:

- **Sessions** (purple) — Number of sessions started
- **Active Users** (green) — Distinct users per time bucket
- **Prompts** (blue) — User prompts submitted
- **Tool Uses** (indigo) — Tool invocations
- **Errors** (red) — Error events

Use the time-range tabs to switch between views:

- **24h** — Last 24 hours, bucketed hourly
- **7d** — Last 7 days (default), bucketed daily
- **14d** — Last 14 days, bucketed daily

The chart also displays a peak value indicator and Y-axis labels that auto-scale to the data.

### Live Activity Feed

The left column shows a **Live Activity** timeline of recent events. Each entry includes:

- The user's display name
- A description of the action
- A relative timestamp (e.g., "2 minutes ago")

Events are color-coded by category: blue for logins, purple for marketplace connections, green for marketplace edits, cyan for sessions, and orange for other events. A link at the bottom takes you to the full [Events](/documentation/events) page.

Real-time updates are delivered via Server-Sent Events (SSE), indicated by a live dot next to the section title.

### System Overview

The right column shows three health cards:

- **Error Rate** — Percentage of events that are errors this week, with a progress bar that turns red when errors exist
- **Active Users (24h)** — Count of distinct users in the last 24 hours
- **Avg Session** — Average session duration, formatted as minutes and seconds

### Models

Below the system overview, a bar chart shows which AI models are being used and how frequently. Model names are shortened for display (e.g., `claude-3-5-sonnet` becomes `sonnet`).

### Top Users

A leaderboard table ranks users by activity. Columns include:

| Column | Description |
|--------|-------------|
| **#** | Rank position (top 3 get special styling) |
| **User** | Display name with avatar initials |
| **Logins** | Number of login events |
| **Prompts** | Number of prompts submitted |
| **Plugins** | Number of plugins accessed |
| **Tokens** | Token usage count |

### Popular Skills

A horizontal bar chart shows the most-used skills (tools) ranked by invocation count.

### Activity (Last 24h)

A mini bar chart shows hourly event counts for the last 24 hours. Each bar is labeled with the hour and shows a tooltip with the exact count.

### Event Breakdown

A horizontal bar chart shows the distribution of event types. Event types include Tool Use, Tool Failure, Session Start, Session End, Turn Complete, Subagent Start, Subagent Stop, and User Prompt. Error-related events are highlighted in red.

### Department Activity

When department data is available, a bar chart shows event counts by department. This section only appears if there is department activity to display.

## Navigating from the Dashboard

The dashboard links to other admin pages:

- Click **View all events** in the live activity feed to go to the [Events](/documentation/events) page
- Use the sidebar to navigate to [Users](/documentation/users), [Jobs](/documentation/jobs), and other management pages

## Troubleshooting

**Dashboard shows all zeros** — This typically means no events have been recorded yet. Verify that hooks are configured and tracking events. Check `systemprompt infra logs view --level error` for issues.

**AI Usage chart shows "No activity data for this time range"** — Try a different time range. If all ranges are empty, confirm that usage events are being recorded in the database.

**Redirected to profile page** — You do not have admin access. Contact an administrator to add the `admin` role to your account.

**Live activity not updating** — The SSE connection may have dropped. Refresh the page to reconnect. Check browser developer tools for SSE connection errors.

## Enterprise Cost & Usage Visibility

The dashboard provides enterprise-scale visibility designed for large organizational deployments. At Foodles scale, cost and usage data must be actionable across organizational boundaries — from individual agents to entire departments.

### Per-Agent Token Consumption

The platform tracks token consumption for each agent independently:

- **developer_agent** — Token usage from internal developer workflows (code assistance, documentation, API integration)
- **associate_agent** — Token usage from domain-specific interactions (scheduling, task management)

Each agent's consumption is broken down by input tokens, output tokens, and total cost, allowing precise attribution of AI spend to business functions.

### Department-Level Cost Tracking

For enterprise chargeback, the dashboard aggregates costs at the department level. Each department sees its own AI spend based on the agents and users within its scope. This enables finance teams to allocate AI costs to the correct cost centers without manual reconciliation.

### Model Usage Distribution

The model usage chart (described above) extends to show distribution across agents. Track which models each agent uses most frequently — for example, whether `associate_agent` is consuming more expensive models than necessary, or whether `developer_agent` could be shifted to a more cost-effective model for routine tasks.

### CLI Commands for Cost Analytics

Use the CLI for programmatic access to cost data:

```bash
# Summary of total costs across all agents and departments
systemprompt analytics costs summary

# Break down costs by individual agent
systemprompt analytics costs breakdown --by agent

# Break down costs by AI model
systemprompt analytics costs breakdown --by model
```

These commands output structured data suitable for scripting, reporting, and integration with existing BI tools.

### RBAC-Governed Cost Visibility

Cost data visibility is governed by the same RBAC system that controls all platform resources. Analysts with department-scoped roles see only their own department's cost data. Organization-wide cost views require the `admin` or `finance` role. This ensures sensitive spend data is compartmentalized appropriately across enterprise-scale deployments.

### Export to CSV

Finance teams can export cost and usage data to CSV for integration with existing accounting and budgeting systems. Exports include per-agent breakdowns, department rollups, model-level costs, and daily/weekly/monthly aggregations. Use the dashboard export button or the CLI to generate reports on demand.
