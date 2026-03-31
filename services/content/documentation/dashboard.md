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
  - title: "Integration: Claude Code"
    url: "/documentation/integration-claude-code"
  - title: "Integration: Claude Cowork"
    url: "/documentation/integration-claude-cowork"
  - title: "Presentation"
    url: "/documentation/presentation"
---

# Dashboard

**TL;DR:** The dashboard is the admin home page. It shows a metric ribbon with today's key numbers, an AI usage chart with selectable time ranges, live activity feed, system health indicators, top users leaderboard, popular skills, event breakdown, model usage, hourly activity, and department activity. Admin access is required — non-admin users are redirected to their profile page.

> **See this in the presentation:** [Slide 8: The Admin Dashboard](/documentation/presentation#slide-8)

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
- The **Share & Install** menu (share icon in the header) provides install commands for [Claude Code](/documentation/integration-claude-code) and export options for [Claude Cowork](/documentation/integration-claude-cowork)
- New users see an onboarding panel with step-by-step installation guides for all three Claude surfaces (Claude Code, Cowork, and claude.ai)

## Enterprise Cost & Usage Visibility

The dashboard provides enterprise-scale visibility designed for large organizational deployments. At Enterprise Demo scale, cost and usage data must be actionable across organizational boundaries — from individual agents to entire departments.

### Per-Agent Token Consumption

The platform tracks token consumption for each agent independently:

- **Platform agents** — Token usage from internal developer workflows (code assistance, documentation, API integration)
- **Domain agents** — Token usage from department-specific interactions (revenue management, operations, scheduling)

Each agent's consumption is broken down by input tokens, output tokens, and total cost, allowing precise attribution of AI spend to business functions.

### Department-Level Cost Tracking

For enterprise chargeback, the dashboard aggregates costs at the department level. Each department sees its own AI spend based on the agents and users within its scope. This enables finance teams to allocate AI costs to the correct cost centers without manual reconciliation.

### Model Usage Distribution

The model usage chart (described above) extends to show distribution across agents. Track which models each agent uses most frequently — for example, whether domain agents are consuming more expensive models than necessary, or whether platform agents could be shifted to a more cost-effective model for routine tasks.

### CLI Commands for Cost Analytics

Use the CLI for programmatic access to cost data:

```bash
# Summary overview of analytics
systemprompt analytics overview

# Analytics for the last 7 days
systemprompt analytics overview --since 7d

# Export analytics data
systemprompt analytics overview --export
```

These commands output structured data suitable for scripting, reporting, and integration with existing BI tools.

### RBAC-Governed Cost Visibility

Cost data visibility is governed by the same RBAC system that controls all platform resources. Analysts with department-scoped roles see only their own department's cost data. Organization-wide cost views require the `admin` or `finance` role. This ensures sensitive spend data is compartmentalized appropriately across enterprise-scale deployments.

### Export to CSV

Finance teams can export cost and usage data to CSV for integration with existing accounting and budgeting systems. Exports include per-agent breakdowns, department rollups, model-level costs, and daily/weekly/monthly aggregations. Use the dashboard export button or the CLI to generate reports on demand.

---

## Enterprise Analytics: Internal Data Integration

The dashboard analytics extend to enterprise-scale reporting with internal data mapping. Capabilities available for scoping in the Phase 1 PRD include:

- **Internal data source mapping** — correlate platform analytics with Enterprise Demo's existing data structures (CRM, ERP, workforce management) for unified reporting
- **Custom analytics dashboards** — define which metrics, charts, and drill-downs appear in the Enterprise Demo Control Center based on operational priorities
- **User analytics with role context** — map platform usage data to Enterprise Demo's internal user hierarchy, teams, and reporting lines
- **Cross-system engagement metrics** — combine AI usage data with business outcomes to measure the impact of AI adoption on operational KPIs
- **Scheduled reporting** — automated report generation and distribution to stakeholders on configurable schedules
