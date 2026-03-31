---
title: "Terminal Demo: Governance Decisions — Allowed vs Denied"
description: "Call the governance API directly with curl to simulate Claude Code's PreToolUse hook. An admin-scope agent is allowed; a user-scope agent is denied. See both decisions on the dashboard."
author: "systemprompt.io"
slug: "demo-terminal-agents"
keywords: "demo, terminal, governance, curl, PreToolUse, scope, allow, deny"
kind: "guide"
public: true
tags: ["demo", "terminal", "governance", "curl", "scope"]
published_at: "2026-03-27"
updated_at: "2026-03-31"
after_reading_this:
  - "Call the governance endpoint with curl to simulate a PreToolUse hook"
  - "See how admin-scope agents receive ALLOW while user-scope agents receive DENY"
  - "Understand the two independent layers of defense (mapping vs rules)"
  - "Query governance decisions from the database"
related_docs:
  - title: "Setup & Authentication"
    url: "/documentation/demo-terminal-setup"
  - title: "Audit Trail Demo"
    url: "/documentation/demo-terminal-audit"
  - title: "Agent Tracing Demo"
    url: "/documentation/demo-terminal-agent-tracing"
  - title: "Agents"
    url: "/documentation/agents"
  - title: "Access Control"
    url: "/documentation/access-control"
---

## Overview

These demos call the governance endpoint directly with curl — simulating Claude Code's PreToolUse hook. No agent session required.

The `developer_agent` (admin scope) is allowed to call MCP tools. The `associate_agent` (user scope) is denied. Both decisions are logged and visible on the dashboard.

**Prerequisites:** Complete [Setup & Authentication](/documentation/demo-terminal-setup) first. You need a valid token in `demo/.token`.

---

## Demo 1: Allowed Path — Admin Scope

### Governance Check

```bash
curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "mcp__systemprompt__list_agents",
    "agent_id": "developer_agent",
    "session_id": "demo-happy-path",
    "tool_input": {}
  }' | python3 -m json.tool
```

The response contains `"decision": "allow"` — governance permits this tool call.

### MCP Tool Result

After governance returns ALLOW, Claude Code proceeds to execute the tool. You can run it manually:

```bash
systemprompt plugins mcp call systemprompt list_agents
```

This returns a real list of running agents from the platform.

### What Happens

1. The curl request simulates a Claude Code PreToolUse hook firing
2. The governance engine receives the request with JWT authentication
3. It evaluates all rules for `developer_agent` (admin scope)
4. `developer_agent` has admin scope and the `systemprompt` MCP server configured — the tool is allowed
5. The response returns `"decision": "allow"` with the matched policy
6. In a real deployment, Claude Code would then execute the MCP tool

> **Why governance is synchronous:** The PreToolUse hook blocks tool execution until the backend returns allow/deny. This is the enforcement point. If governance were async, tools could execute before the decision arrives.

### Dashboard

Open [/admin/governance](/admin/governance). You should see the allow decision with:

- Agent: `developer_agent`
- Tool: `mcp__systemprompt__list_agents`
- Decision: **allow**
- Policy: the matched governance policy name

---

## Demo 2: Refused Path — User Scope

### Governance Check

```bash
curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "mcp__systemprompt__list_agents",
    "agent_id": "associate_agent",
    "session_id": "demo-refused-path",
    "tool_input": {}
  }' | python3 -m json.tool
```

The response contains `"decision": "deny"` — governance blocks this tool call.

### What Happens

1. The curl request simulates a PreToolUse hook for `associate_agent`
2. The governance engine evaluates the `scope_restriction` rule
3. `associate_agent` has **user scope** — admin-only tools are denied
4. The response returns `"decision": "deny"` with the reason

> **Defense-in-depth — two independent layers:**
>
> **Layer 1 — Mapping (preventive).** In a real Claude Code deployment, user-scope agents never even see admin tools. The MCP server mapping excludes them entirely — the tool does not appear in the agent's tool list.
>
> **Layer 2 — Governance rules (enforcement).** Even if mapping were misconfigured, the `scope_restriction` rule evaluates every PreToolUse hook call. A user-scope agent calling an admin tool is denied and logged. Neither layer depends on the other.

### Dashboard

Open [/admin/governance](/admin/governance). You should see the deny decision with:

- Agent: `associate_agent`
- Tool: `mcp__systemprompt__list_agents`
- Decision: **deny**
- Policy: `scope_restriction`

---

## Side-by-Side Comparison

| | developer_agent | associate_agent |
|---|---|---|
| **Scope** | admin | user |
| **Governance decision** | allow | deny |
| **Policy matched** | admin tool access | scope_restriction |
| **Tool executes** | Yes (MCP call proceeds) | No (blocked by governance) |
| **Defense layer** | Passes both mapping and rules | Blocked at mapping level; denied at rules level |
| **AI cost** | Free (no AI call) | Free (no AI call) |

Both decisions are logged in `governance_decisions` and visible on the governance dashboard.

---

## Audit

Verify both governance decisions exist in the database:

```bash
systemprompt infra db query \
  "SELECT decision, tool_name, agent_id, agent_scope, policy FROM governance_decisions ORDER BY created_at DESC LIMIT 5"
```

**Expected results:**

| decision | tool_name | agent_id | agent_scope | policy |
|---|---|---|---|---|
| deny | mcp__systemprompt__list_agents | associate_agent | user | scope_restriction |
| allow | mcp__systemprompt__list_agents | developer_agent | admin | admin tool access |

---

## Next

Run [Audit Trails & Costs](/documentation/demo-terminal-audit) to inspect the governance trail and cost breakdown from these two demos.
