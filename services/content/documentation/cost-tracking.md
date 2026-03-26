---
title: "Cost Tracking & Model Usage"
description: "Track AI costs by model, department, user, and agent. Monitor token consumption, model distribution, and cost allocation across the organization."
author: "systemprompt.io"
slug: "cost-tracking"
keywords: "cost tracking, analytics, token usage, model costs, budget"
kind: "guide"
public: true
tags: ["analytics", "costs", "enterprise"]
published_at: "2026-03-19"
updated_at: "2026-03-19"
after_reading_this:
  - "Use the CLI to query cost data by model, agent, provider, and department"
  - "Navigate the dashboard cost metrics and usage charts"
  - "Set up department-level chargeback reporting"
  - "Understand RBAC controls on cost data visibility"
related_docs:
  - title: "Platform Architecture"
    url: "/documentation/architecture"
  - title: "Scaling Architecture"
    url: "/documentation/scaling"
  - title: "Rate Limiting & Compliance"
    url: "/documentation/rate-limiting"
  - title: "Access Control"
    url: "/documentation/access-control"
  - title: "Agents"
    url: "/documentation/agents"
  - title: "Presentation"
    url: "/documentation/presentation"
---

# Cost Tracking & Model Usage

**TL;DR:** The platform tracks every AI request by model, agent, provider, department, and user. Cost data is available through the CLI (`systemprompt analytics costs`) and the admin dashboard. Department-level chargeback, model usage distribution, and CSV export give finance teams the data they need. RBAC controls ensure analysts see only their own department's costs.

> **See this in the presentation:** [Slide 16: Cost Visibility & Analytics](/documentation/presentation#slide-16)

## Why Cost Visibility Matters at Enterprise Scale

At enterprise scale across multiple departments, AI costs can grow unpredictably. Without visibility:

- Finance cannot allocate costs to cost centers
- Departments have no incentive to optimize usage
- Expensive model usage goes undetected until the monthly bill arrives
- Agent misconfigurations (retry loops, verbose prompts) burn tokens silently

The platform solves this by tracking cost data at every dimension that matters for enterprise budgeting.

## Cost Breakdown Dimensions

Every AI request records the following cost-relevant data:

| Dimension | What It Tracks | Why It Matters |
|-----------|---------------|----------------|
| **Model** | Which AI model processed the request (e.g., Claude Sonnet, GPT-4) | Different models have different per-token costs |
| **Agent** | Which agent initiated the request (e.g., developer_agent) | Identifies which workflows consume the most AI |
| **Provider** | Which AI provider served the request (e.g., Anthropic, OpenAI) | Tracks vendor spend for contract negotiations |
| **Department** | Which department the requesting user belongs to | Enables chargeback to cost centers |
| **User** | Which user triggered the request | Identifies individual usage patterns |
| **Token count** | Input tokens + output tokens for each request | The basis for cost calculation |
| **Timestamp** | When the request was made | Enables trend analysis and anomaly detection |

## CLI Commands

The analytics CLI provides full access to cost data from the command line.

### Cost Summary

```bash
# Overall cost summary
systemprompt analytics costs summary
```

Returns total token consumption, total cost, and breakdown by the top dimensions. Use this as your starting point to understand overall spend.

### Cost Breakdown

Break down costs by any dimension:

```bash
# By model — which models cost the most?
systemprompt analytics costs breakdown --by model

# By agent — which agents consume the most tokens?
systemprompt analytics costs breakdown --by agent

# By provider — how much are you spending with each AI vendor?
systemprompt analytics costs breakdown --by provider
```

Each breakdown shows:
- Total tokens (input + output)
- Estimated cost
- Percentage of total spend
- Request count

### Cost Trends

Track cost changes over time:

```bash
# Last 7 days
systemprompt analytics costs trends --since 7d

# Last 30 days
systemprompt analytics costs trends --since 30d

# Last 24 hours (for anomaly detection)
systemprompt analytics costs trends --since 24h
```

Trend data shows daily aggregates with token counts, costs, and percentage change from the previous period. Use this to detect unexpected cost increases early.

### Additional Analytics Commands

```bash
# Overview of all analytics
systemprompt analytics overview

# Conversation analytics
systemprompt analytics conversations

# Agent-specific analytics
systemprompt analytics agents

# Tool usage analytics
systemprompt analytics tools

# Session analytics
systemprompt analytics sessions

# Traffic analytics
systemprompt analytics traffic
```

## Dashboard Cost Metrics

The admin dashboard (`/admin/`) displays cost data in several components.

### Metric Ribbon

The metric ribbon at the top of the dashboard shows real-time counts:

| Metric | What It Shows |
|--------|--------------|
| **Events** | Total platform events in the selected time period |
| **Tool Uses** | Number of MCP tool invocations |
| **Prompts** | Number of AI prompts sent |
| **Sessions** | Active user sessions |
| **Active Users** | Unique users who made requests |
| **Errors** | Error count for the period |

### AI Usage Chart

The main dashboard includes an AI usage chart showing token consumption over time. This chart breaks down by:

- **Input tokens** — tokens sent to AI models (prompts, context, system instructions)
- **Output tokens** — tokens generated by AI models (responses, tool calls)
- **Total cost** — estimated cost based on per-model pricing

### Department Activity Breakdown

A dedicated section shows AI activity by department:

| Column | Description |
|--------|-------------|
| **Department** | The department name |
| **Users** | Number of active users in the department |
| **Requests** | Total AI requests from the department |
| **Tokens** | Total token consumption |
| **Cost** | Estimated cost for the department |
| **% of Total** | Department's share of total AI spend |

This is the primary data source for chargeback reporting.

### Model Usage Distribution

A breakdown of which AI models are being used:

| Column | Description |
|--------|-------------|
| **Model** | Model name and version |
| **Provider** | AI provider (Anthropic, OpenAI, etc.) |
| **Requests** | Number of requests using this model |
| **Tokens** | Total token consumption |
| **Cost** | Estimated cost |
| **% of Total** | Model's share of total spend |

Use this to identify opportunities for cost optimization — replacing expensive models with cheaper alternatives for non-critical tasks.

### Popular Skills

The dashboard shows which skills generate the most AI activity:

| Column | Description |
|--------|-------------|
| **Skill** | Skill name |
| **Usage Count** | Number of times the skill was invoked |
| **Agents** | Which agents use this skill |
| **Avg Tokens** | Average token consumption per invocation |

## Department-Level Chargeback

For organizations that charge AI costs back to departments, the platform provides the data needed for monthly chargeback reports.

### How Chargeback Works

1. Every AI request is tagged with the requesting user's department (from their user record)
2. Token consumption and estimated cost are calculated per request
3. Costs aggregate to the department level
4. Finance exports the department breakdown as CSV

### Setting Up Departments

Departments are configured on user records. When a user makes an AI request, their department is recorded with the request.

```bash
# List users with departments
systemprompt admin users list

# Update a user's department
systemprompt admin users update <user-id> --department "Commerce"
```

### Chargeback Report Example

| Department | Users | Requests | Input Tokens | Output Tokens | Total Cost | % of Total |
|-----------|------:|--------:|-----------:|-----------:|----------:|---------:|
| Commerce | 45 | 12,340 | 8,200,000 | 3,100,000 | $1,240.00 | 38% |
| Technology | 32 | 9,870 | 6,500,000 | 2,800,000 | $980.00 | 30% |
| Supply Chain | 28 | 5,430 | 3,200,000 | 1,400,000 | $520.00 | 16% |
| Corporate | 18 | 3,210 | 2,100,000 | 900,000 | $340.00 | 10% |
| Other | 12 | 1,890 | 1,100,000 | 500,000 | $190.00 | 6% |

## Agent Cost Attribution

Each agent's AI consumption is tracked independently. This answers the question: "Which agent costs the most to operate?"

| Agent | Role | Typical Cost Profile |
|-------|------|---------------------|
| **developer_agent** | Code assistance and tooling | Variable — spikes during development sprints |
| **associate_agent** | Domain-specific workflows | Medium volume — queries during business hours |

### Identifying Cost Anomalies

```bash
# Check which agents have the highest cost
systemprompt analytics costs breakdown --by agent

# Look at trends for a specific period
systemprompt analytics costs trends --since 7d
```

Common cost anomalies to watch for:

| Anomaly | Cause | Fix |
|---------|-------|-----|
| **Sudden cost spike** | Agent retry loop, verbose system prompt, or model upgrade | Check agent logs: `systemprompt infra logs trace list --agent <name> --status failed` |
| **Steady cost increase** | Growing user base or expanding skill set | Expected — validate against user growth |
| **One agent dominates** | Agent handling too many use cases | Consider splitting into specialized sub-agents |
| **High input token ratio** | System prompts or context windows too large | Review and trim system prompts and skill content |

## Model Usage Distribution

Track which AI models are used across the platform and optimize for cost vs quality.

### Model Cost Comparison

Different models have different cost profiles. The platform tracks usage by model so you can make informed decisions:

| Decision | Data Point |
|----------|-----------|
| **Use cheaper models for simple tasks** | Model breakdown shows which models handle which request types |
| **Reserve expensive models for complex tasks** | Agent-to-model mapping shows where premium models are needed |
| **Negotiate volume discounts** | Provider breakdown shows total spend per vendor |
| **Detect model drift** | Trend data shows if agents are switching to more expensive models |

### Tool Model Overrides

Agents support `toolModelOverrides` — allowing specific tools to use different models than the agent's default. This is a key cost optimization lever:

- Default agent model: Claude Sonnet (balanced cost/quality)
- Complex reasoning tool: Claude Opus (higher quality, higher cost)
- Simple lookup tool: Claude Haiku (lower quality, lower cost)

## Export for Finance Teams

Cost data can be exported as CSV for integration with finance systems.

### What the Export Includes

| Field | Description |
|-------|-------------|
| **Date** | Date of the request |
| **Department** | User's department |
| **User** | Username or user ID |
| **Agent** | Agent that processed the request |
| **Model** | AI model used |
| **Provider** | AI provider |
| **Input Tokens** | Tokens sent to the model |
| **Output Tokens** | Tokens generated by the model |
| **Estimated Cost** | Cost based on per-model pricing |

### Export Workflow

1. Navigate to the analytics dashboard
2. Set the date range for the export period
3. Apply any filters (department, agent, model)
4. Click export to download CSV
5. Import into your finance system for chargeback processing

## RBAC on Cost Data

Not everyone should see all cost data. The platform applies RBAC to analytics:

| Role | What They See |
|------|--------------|
| **Admin** | All cost data across all departments, all agents, all users |
| **Analyst** | Cost data for their own department only |
| **Developer** | Cost data for agents they manage |
| **Viewer** | No cost data — analytics pages are hidden |

### Department-Scoped Analytics

When an analyst logs into the dashboard, the analytics page automatically filters to their department. They see:

- Their department's total cost
- Their department's agent usage
- Their department's model distribution
- Their department's user activity

They do not see other departments' data, total platform cost, or cross-department comparisons. This is enforced at the API level — not just the UI.

### Admin Override

Admins see the complete picture. The admin dashboard shows:

- Total platform cost across all departments
- Cross-department comparison charts
- Per-user cost data for any user
- Model and provider spend totals for vendor negotiations

## Troubleshooting

| Issue | Diagnosis | Solution |
|-------|-----------|----------|
| **Cost data not appearing** | Analytics may not have processed recent requests | Check `systemprompt infra logs view --level error` for analytics pipeline errors |
| **Department showing as "Unknown"** | Users without a department assigned | Update user records: `systemprompt admin users update <id> --department "Name"` |
| **Cost estimates seem wrong** | Model pricing may not be current | Verify model pricing configuration matches your provider's current rates |
| **Export missing data** | Date range may not cover the full period | Expand the date range and verify the time zone setting |
| **Analyst sees no data** | RBAC filtering with no department assigned | Assign the analyst to a department in user management |
| **Agent cost not attributed** | Agent name mismatch between config and requests | Verify agent names in `services/agents/` match the names in request logs |
