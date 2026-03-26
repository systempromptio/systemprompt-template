---
title: "Demo: Detailed Breakdown — What Happens Under the Hood"
description: "Step-by-step technical breakdown of the governance pipeline for both the happy and refused demo paths, with CLI commands to inspect every layer."
author: "systemprompt.io"
slug: "demo-breakdown"
keywords: "demo, breakdown, audit, trace, analytics, governance, cli"
kind: "guide"
public: true
tags: ["demo", "governance", "audit", "analytics", "trace"]
published_at: "2026-03-20"
updated_at: "2026-03-20"
after_reading_this:
  - "Inspect every governance layer using CLI commands"
  - "Compare audit trails between allowed and denied requests"
  - "Understand the full request lifecycle from authentication to analytics"
related_docs:
  - title: "Happy Path"
    url: "/documentation/demo-happy-path"
  - title: "Refused Path"
    url: "/documentation/demo-refused-path"
  - title: "Audit Trails & Events"
    url: "/documentation/events"
  - title: "Cost Tracking"
    url: "/documentation/cost-tracking"
---

## Overview

This page walks through exactly what happens in the governance pipeline for both demo paths, with CLI commands to inspect every layer. Run the happy path and refused path demos first, then use the commands below to examine the results.

---

## Setup: The two agents

| | developer_agent | associate_agent |
|---|---|---|
| **OAuth scope** | admin | user |
| **MCP servers** | systemprompt (CLI executor, port 5010) | none |
| **Skills** | General Assistance, Rust Standards, Architecture Standards | General Assistance |
| **Config file** | `services/agents/developer_agent.yaml` | `services/agents/associate_agent.yaml` |

The key difference: `developer_agent` has `systemprompt` in its `mcpServers` list. `associate_agent` does not. This is the governance boundary.

---

## Step 1: Run both demos

```bash
# Happy path — admin agent with systemprompt MCP
systemprompt admin agents message developer_agent \
  -m "List all agents running on this platform" \
  --blocking --timeout 60

# Refused path — user agent without systemprompt MCP
systemprompt admin agents message associate_agent \
  -m "List all agents running on this platform using the CLI tools" \
  --blocking --timeout 60
```

---

## Step 2: List recent requests

```bash
systemprompt infra logs request list --limit 5
```

This shows the two requests you just made. Note the request IDs — you'll need them for the next steps.

**What to look for:**
- Both requests should appear with timestamps
- `developer_agent` request should show status: completed
- `associate_agent` request should also show completed (the AI processed it, even though tools were denied)

---

## Step 3: Compare audit trails

```bash
# Happy path audit
systemprompt infra logs audit <developer-request-id> --full

# Refused path audit
systemprompt infra logs audit <associate-request-id> --full
```

### Happy path audit should show:

```
=== REQUEST TRACE ===
--- IDENTITY ---
User:         <your user>
Role:         admin
Department:   <your department>
Session:      ses-xxxxx

--- AGENT ---
Agent:        developer_agent
Plugin:       enterprise-demo
Model:        claude-4-sonnet

--- PERMISSIONS ---
ACL:          agent:developer_agent → role:admin → ALLOW
MCP:          systemprompt → role:admin → ALLOW

--- TOOL CALLS ---
1. systemprompt (CLI)    → OK   (Xms)

--- AI REQUEST ---
Tokens In:    X,XXX
Tokens Out:   X,XXX
Cost:         $X.XXXX
Status:       completed
```

### Refused path audit should show:

```
=== REQUEST TRACE ===
--- IDENTITY ---
User:         <your user>
Role:         admin
Department:   <your department>
Session:      ses-xxxxx

--- AGENT ---
Agent:        associate_agent
Plugin:       enterprise-demo
Model:        claude-4-sonnet

--- PERMISSIONS ---
ACL:          agent:associate_agent → role:admin → ALLOW

--- TOOL CALLS ---
(none — systemprompt not in agent's MCP mapping)

--- AI REQUEST ---
Tokens In:    X,XXX
Tokens Out:   X,XXX
Cost:         $X.XXXX
Status:       completed
```

The critical difference: **no tool calls** in the refused path. The `systemprompt` MCP server was never offered to the agent because it's not in the `mcpServers` list.

---

## Step 4: Examine execution traces

```bash
# Developer agent traces — should show MCP tool calls
systemprompt infra logs trace list --agent developer_agent --limit 3
systemprompt infra logs trace show <trace-id> --all

# Associate agent traces — should show NO MCP tool calls
systemprompt infra logs trace list --agent associate_agent --limit 3
systemprompt infra logs trace show <trace-id> --all
```

The `--all` flag shows every step in the execution: message received, tool discovery, tool calls (or lack thereof), response generation.

---

## Step 5: View analytics

```bash
# Per-agent analytics
systemprompt analytics agents show developer_agent
systemprompt analytics agents show associate_agent

# Cost breakdown by agent
systemprompt analytics costs breakdown --by agent

# Overall overview
systemprompt analytics overview --since 1h
```

**What to look for:**
- Both agents show one request each
- `developer_agent` may show higher token usage (tool call adds to the conversation)
- Cost breakdown shows per-agent spend
- Tool usage analytics may show `systemprompt` tool with 1 call from `developer_agent`, 0 from `associate_agent`

---

## Step 6: Inspect tool usage

```bash
# All tool calls
systemprompt analytics tools stats

# Specific tool details
systemprompt analytics tools list
```

This shows which MCP tools were called, by which agents, success/failure rates, and average durations. The `systemprompt` tool should show exactly one call from `developer_agent`.

---

## How to replicate

These three commands are all you need:

```bash
# 1. Happy path
systemprompt admin agents message developer_agent \
  -m "List all agents running on this platform" \
  --blocking --timeout 60

# 2. Refused path
systemprompt admin agents message associate_agent \
  -m "List all agents running on this platform using the CLI tools" \
  --blocking --timeout 60

# 3. Compare
systemprompt infra logs request list --limit 5
systemprompt infra logs audit <happy-path-id> --full
systemprompt infra logs audit <refused-path-id> --full
systemprompt analytics costs breakdown --by agent
```

Each run of demos 1 and 2 makes one AI inference call. The audit and analytics commands are read-only and free.

---

## What this proves

1. **RBAC works** — Different agents have different OAuth scopes. The ACL table governs access.
2. **MCP governance works** — Agent-tool mapping is explicit. Tools not in the mapping don't exist for that agent.
3. **Audit is complete** — Every request has a full 5-layer trace: who, which agent, what permissions, which tools, what the AI did.
4. **Analytics are real** — Cost, token usage, latency — all tracked per-agent, per-tool, per-model.
5. **The governance gap is filled** — You can answer: "What did the AI do on behalf of this user, and was it authorized?"
