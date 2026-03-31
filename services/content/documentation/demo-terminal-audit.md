---
title: "Terminal Demo: Audit Trails & Cost Tracking"
description: "Inspect governance decisions from the database, compare allowed vs denied policies, and view cost breakdown by agent. Every decision persists in the database and appears on the dashboard."
author: "systemprompt.io"
slug: "demo-terminal-audit"
keywords: "demo, terminal, audit, governance, decisions, cost tracking, analytics"
kind: "guide"
public: true
tags: ["demo", "terminal", "audit", "governance", "cost-tracking"]
published_at: "2026-03-27"
updated_at: "2026-03-31"
after_reading_this:
  - "Query governance decisions from the database"
  - "Compare policy outcomes between allowed and denied paths"
  - "View cost breakdown by agent"
related_docs:
  - title: "Agent Messaging Demo"
    url: "/documentation/demo-terminal-agents"
  - title: "Governance API Demo"
    url: "/documentation/demo-terminal-governance"
  - title: "Agent Tracing Demo"
    url: "/documentation/demo-terminal-agent-tracing"
  - title: "Cost Tracking"
    url: "/documentation/cost-tracking"
  - title: "Events"
    url: "/documentation/events"
---

## Overview

After running the [Governance Decisions](/documentation/demo-terminal-governance) demos, every decision is recorded. This demo inspects those decisions and shows cost attribution.

**Prerequisites:** Run [Governance Decisions](/documentation/demo-terminal-governance) first. You need at least two governance decisions in the system.

---

## Step 1: List Recent Governance Decisions

```bash
systemprompt infra db query "SELECT decision, tool_name, agent_id, agent_scope, policy, reason FROM governance_decisions ORDER BY created_at DESC LIMIT 5"
```

This returns the most recent governance decisions with their outcomes, the tool requested, the agent identity, scope, and which policy produced the result.

---

## Step 2: Inspect the ALLOW Decision (developer_agent)

### What to Look For

The allowed decision for developer_agent should show:

- **decision=allow** — the request was permitted
- **scope=admin** — the agent operates with admin-level access
- **policy=default_allow** — the default policy grants access to admin-scope agents

Admin-scope agents pass governance evaluation because the default policy permits tool access at that privilege level.

> **Why decisions are stored:** Every governance evaluation is persisted as a database row — not just logged. This means you can query, aggregate, and audit decisions with standard SQL. Typed columns (decision, scope, policy) enforce valid values at the schema level.

---

## Step 3: Inspect the DENY Decision (associate_agent)

### What to Look For

The denied decision for associate_agent should show:

- **decision=deny** — the request was blocked
- **scope=user** — the agent operates with user-level access
- **policy=scope_restriction** — the policy restricts tool access for user-scope agents

The contrast is the point: the user-scope agent is denied because governance enforces scope-based restrictions before any tool execution occurs.

> **Why the contrast matters:** The denied decision proves that access denial is not a failure mode — it's a designed outcome with its own complete audit record. The system records denials with the same fidelity as approvals.

---

## Step 4: Cost Breakdown

```bash
systemprompt analytics costs breakdown --by agent
```

This shows token consumption and cost attribution per agent. The developer_agent will show higher costs (multiple AI calls, tool execution). The associate_agent will show minimal cost (single response, no tools).

---

## Dashboard

Open the dashboard and observe:

- [/admin/](/admin/) **Governance tab** — metric ribbon (Total Decisions, Allowed, Denied, Secret Breaches), policy violations, recent governance events
- [/admin/](/admin/) **MCP & Usage tab** — AI usage chart with 24h/7d/14d toggle, live activity feed, MCP server access events, cost breakdown by agent
- [/admin/governance](/admin/governance) — full governance decision log with search
- [/admin/events](/admin/events) — complete audit trail of all platform activity

All data visible on the dashboard is the same data you queried from the terminal. The dashboard is a live view of the same database.

---

## What This Proves

| Property | ALLOW (developer_agent) | DENY (associate_agent) |
|----------|------------------------|------------------------|
| Decision | allow | deny |
| Policy | default_allow | scope_restriction |
| Scope | admin | user |
| Tool access | Granted | Blocked |
| Cost | Higher (multi-turn + tool) | Minimal (single response) |

Every governance decision is recorded, queryable, and attributed — regardless of outcome.

> **Why cost attribution matters:** Cost breakdown by agent answers the CTO question: "which teams are spending how much on AI?" Typed `CostBreakdown { agent: AgentName, total_cost: Decimal, request_count: i64 }` structs — not raw tuples — ensure accurate attribution.

---

> **Note:** For full agent execution traces (AI requests, MCP tool calls, artifacts), run Demo 09 and see [Agent Tracing Demo](/documentation/demo-terminal-agent-tracing).

---

## Next

Run [Governance API](/documentation/demo-terminal-governance) to call the governance endpoint directly with curl.
