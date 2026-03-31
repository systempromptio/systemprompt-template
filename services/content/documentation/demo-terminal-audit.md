---
title: "Terminal Demo: Audit Trails & Cost Tracking"
description: "Inspect execution traces, compare allowed vs refused paths, and view cost breakdown by agent. Every event persists in the database and appears on the dashboard."
author: "systemprompt.io"
slug: "demo-terminal-audit"
keywords: "demo, terminal, audit, traces, cost tracking, analytics"
kind: "guide"
public: true
tags: ["demo", "terminal", "audit", "traces", "cost-tracking"]
published_at: "2026-03-27"
updated_at: "2026-03-31"
after_reading_this:
  - "List and inspect execution traces from the CLI"
  - "Compare trace depth between allowed and denied agent paths"
  - "View cost breakdown by agent"
related_docs:
  - title: "Agent Messaging Demo"
    url: "/documentation/demo-terminal-agents"
  - title: "Governance API Demo"
    url: "/documentation/demo-terminal-governance"
  - title: "Cost Tracking"
    url: "/documentation/cost-tracking"
  - title: "Events"
    url: "/documentation/events"
  - title: "Request Tracing Demo"
    url: "/documentation/demo-terminal-tracing"
---

## Overview

After running the [Agent Messaging](/documentation/demo-terminal-agents) demos, every request is fully traced. This demo inspects those traces from the terminal and shows cost attribution.

**Prerequisites:** Run [Agent Messaging](/documentation/demo-terminal-agents) first. You need at least two traces in the system.

---

## Step 1: List Recent Traces

```bash
systemprompt infra logs trace list --limit 2
```

Extract trace IDs and agent names:

```bash
TRACES_JSON=$(systemprompt infra logs trace list --limit 2 2>&1)

TRACE_1=$(echo "$TRACES_JSON" | grep -oP '"trace_id":\s*"\K[0-9a-f-]+' | head -1)
TRACE_2=$(echo "$TRACES_JSON" | grep -oP '"trace_id":\s*"\K[0-9a-f-]+' | head -2 | tail -1)
AGENT_1=$(echo "$TRACES_JSON" | grep -oP '"agent":\s*"\K[^"]+' | head -1)
AGENT_2=$(echo "$TRACES_JSON" | grep -oP '"agent":\s*"\K[^"]+' | head -2 | tail -1)

echo "Trace 1: $AGENT_1 ($TRACE_1)"
echo "Trace 2: $AGENT_2 ($TRACE_2)"
```

---

## Step 2: Inspect the Allowed Path Trace

```bash
systemprompt infra logs trace show "$TRACE_1" --all
```

### What to Look For

The allowed path trace (developer_agent) should show:

- **~11 traced events** across the full execution
- **3 AI requests** (reasoning, tool selection, response synthesis)
- **1 MCP tool call** (list_agents on the systemprompt server)
- **7 execution steps** with timing for each
- Token counts and cost per request

Every layer is captured: identity, governance evaluation, tool execution, response.

> **Why trace everything:** Every AI interaction is traced, costed, and attributed — regardless of outcome. The trace uses typed IDs (TraceId, SessionId, UserId) that are newtypes in Rust. The compiler prevents mixing them up, which means every trace event is guaranteed to carry the correct IDs. No audit-linking bugs possible.

---

## Step 3: Inspect the Refused Path Trace

```bash
systemprompt infra logs trace show "$TRACE_2" --all
```

### What to Look For

The refused path trace (associate_agent) should show:

- **~4 traced events** — much shorter
- **1 AI request** (the agent responds immediately)
- **0 MCP tool calls** — no tools were available
- **3 execution steps**

The contrast is the point: the user-scope agent generates a minimal trace because governance prevented tool access entirely.

> **Why the contrast matters:** The refused path proves that access denial is not a failure mode — it's a designed outcome with its own complete audit trail. The system traces denials with the same fidelity as approvals.

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

| Property | Allowed Path | Refused Path |
|----------|-------------|--------------|
| Events traced | ~11 | ~4 |
| AI requests | ~3 | 1 |
| MCP tool calls | 1 | 0 |
| Cost | Higher (multi-turn + tool) | Minimal (single response) |
| Governance | Evaluated and passed | Enforced at mapping level |

Every AI interaction is traced, costed, and attributed — regardless of outcome.

> **Why cost attribution matters:** Cost breakdown by agent answers the CTO question: "which teams are spending how much on AI?" Typed `CostBreakdown { agent: AgentName, total_cost: Decimal, request_count: i64 }` structs — not raw tuples — ensure accurate attribution.

---

## Next

Run [Governance API](/documentation/demo-terminal-governance) to call the governance endpoint directly with curl.
