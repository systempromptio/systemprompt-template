---
title: "Terminal Demo: Agent Messaging — Allowed vs Refused"
description: "Send messages to two agents with different scopes. The admin-scope agent executes MCP tools; the user-scope agent cannot. See both results on the dashboard."
author: "systemprompt.io"
slug: "demo-terminal-agents"
keywords: "demo, terminal, agents, messaging, governance, scope, mcp"
kind: "guide"
public: true
tags: ["demo", "terminal", "agents", "governance", "mcp"]
published_at: "2026-03-27"
updated_at: "2026-03-31"
after_reading_this:
  - "Send messages to agents via the CLI"
  - "See how admin-scope agents can use MCP tools while user-scope agents cannot"
  - "Retrieve structured artifacts from agent responses"
  - "Audit the execution trace and verify event counts"
related_docs:
  - title: "Setup & Authentication"
    url: "/documentation/demo-terminal-setup"
  - title: "Audit Trail Demo"
    url: "/documentation/demo-terminal-audit"
  - title: "Agents"
    url: "/documentation/agents"
  - title: "Access Control"
    url: "/documentation/access-control"
  - title: "Request Tracing Demo"
    url: "/documentation/demo-terminal-tracing"
---

## Overview

This demo sends the same request to two agents with different scopes. The `developer_agent` (admin scope, MCP access) returns a real list of agents. The `associate_agent` (user scope, no MCP) refuses because it has no tool access.

**Prerequisites:** Complete [Setup & Authentication](/documentation/demo-terminal-setup) first.

---

## Demo 1: Allowed Path — Admin Scope

### Create an Isolated Context

```bash
CONTEXT_OUTPUT=$(systemprompt core contexts create --name "Demo 1 - Happy Path $(date +%H:%M:%S)" 2>&1)
CONTEXT_ID=$(echo "$CONTEXT_OUTPUT" | grep "^ID:" | awk '{print $2}')
echo "Context: $CONTEXT_ID"
```

### Send the Message

```bash
systemprompt admin agents message developer_agent \
  -m "List all agents running on this platform" \
  --context-id "$CONTEXT_ID" \
  --blocking --timeout 60
```

### What Happens

1. `developer_agent` receives the message
2. It has **admin scope** and the **systemprompt MCP server** configured
3. The agent calls the `list_agents` MCP tool
4. The PreToolUse governance hook evaluates the call and **allows** it
5. The tool executes and returns a real list of running agents

> **Why context isolation?** Every agent conversation gets its own ContextId. This prevents cross-contamination between sessions and enables per-context artifact retrieval, cost tracking, and trace linking. The ContextId is a Rust newtype — the compiler prevents confusing it with other IDs.

> **Why governance is synchronous:** The PreToolUse hook blocks tool execution until the backend returns allow/deny. This is the enforcement point. If governance were async, tools could execute before the decision arrives.

### Retrieve the Artifact

The agent produces a structured artifact — typed data that any surface (dashboard, mobile app, CLI) can render:

```bash
ARTIFACT_ID=$(systemprompt core artifacts list --context-id "$CONTEXT_ID" 2>&1 \
  | grep -oP '"id":\s*"\K[^"]+' | head -1)

systemprompt core artifacts show "$ARTIFACT_ID" --full
```

### Dashboard

Open [/admin/events](/admin/events). You should see the agent message event with:

- Agent name: `developer_agent`
- Tool calls: 1 MCP call (`list_agents`)
- Status: completed

---

## Demo 2: Refused Path — User Scope

### Create Context and Send Message

```bash
CONTEXT_OUTPUT=$(systemprompt core contexts create --name "Demo 2 - Refused Path $(date +%H:%M:%S)" 2>&1)
CONTEXT_ID=$(echo "$CONTEXT_OUTPUT" | grep "^ID:" | awk '{print $2}')
echo "Context: $CONTEXT_ID"

systemprompt admin agents message associate_agent \
  -m "List all agents running on this platform using the CLI tools" \
  --context-id "$CONTEXT_ID" \
  --blocking --timeout 60
```

### What Happens

1. `associate_agent` receives the message
2. It has **user scope** and **no MCP servers** configured
3. The agent cannot see any admin tools
4. It responds: *"I do not have access to that tool. This operation requires elevated permissions that have not been granted to this agent."*

> **Why two layers of defense:** Demo 2 denies at the *mapping* level (no tools available). Demo 5 denies at the *rule* level (governance blocks the call). These are independent — even if tool mappings were misconfigured, governance would still catch unauthorized access.

### Dashboard

Open [/admin/events](/admin/events). This trace shows:

- Agent name: `associate_agent`
- AI requests: 1
- MCP tool calls: 0
- The agent responded without attempting any tool use

---

## Side-by-Side Comparison

| | developer_agent | associate_agent |
|---|---|---|
| **Scope** | admin | user |
| **MCP servers** | systemprompt | none |
| **Tool calls** | 1 (list_agents) | 0 |
| **Artifact** | Structured agent list | None |
| **Traced events** | ~11 | ~4 |
| **AI requests** | ~3 | 1 |

The governance enforcement happens at the mapping level. The user-scope agent never sees admin tools — it cannot attempt a call that would be denied, because the tools are not available to it in the first place.

---

## Audit

Verify both traces exist with expected event counts:

```bash
# List the two most recent traces
systemprompt infra logs trace list --limit 2

# Inspect the happy path trace (should show ~11 events, 3 AI requests, 1 MCP call)
TRACE_ID=$(systemprompt infra logs trace list --limit 1 2>&1 | grep -oP '"trace_id":\s*"\K[0-9a-f-]+' | head -1)
systemprompt infra logs trace show "$TRACE_ID" --all

# Cost breakdown by agent
systemprompt analytics costs breakdown --by agent

# Governance log (most recent decision)
tail -5 /tmp/systemprompt-governance-*.log
```

**Expected results:**

| | developer_agent | associate_agent |
|---|---|---|
| Trace events | ~11 | ~4 |
| AI requests | ~3 | 1 |
| MCP tool calls | 1 | 0 |
| Governance decisions | 1 (allow) | 0 |

---

## Next

Run [Audit Trails & Costs](/documentation/demo-terminal-audit) to inspect the traces generated by these two demos.
