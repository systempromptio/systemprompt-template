---
title: "Terminal Demo: MCP Access Tracking & Database Audit"
description: "Combine governance API calls, live MCP tool execution, and direct database queries. See governance decisions, MCP access events, and tool call tracking on the dashboard."
author: "systemprompt.io"
slug: "demo-terminal-mcp"
keywords: "demo, terminal, mcp, governance, database, audit, tracking"
kind: "guide"
public: true
tags: ["demo", "terminal", "mcp", "audit", "database", "governance"]
published_at: "2026-03-27"
updated_at: "2026-03-31"
after_reading_this:
  - "Execute MCP tool calls and see them tracked in the database"
  - "Query the governance_decisions table directly"
  - "Query user_activity for MCP access events"
  - "See all events reflected on the admin dashboard in real time"
related_docs:
  - title: "Governance API Demo"
    url: "/documentation/demo-terminal-governance"
  - title: "Setup & Authentication"
    url: "/documentation/demo-terminal-setup"
  - title: "Tool Governance"
    url: "/documentation/tool-governance"
  - title: "MCP Servers"
    url: "/documentation/mcp-servers"
  - title: "Dashboard"
    url: "/documentation/dashboard"
  - title: "Request Tracing Demo"
    url: "/documentation/demo-terminal-tracing"
---

## Overview

This capstone demo combines governance API calls, a live MCP tool call, and direct database queries. It shows the full audit trail from governance decision through tool execution to database persistence.

**Prerequisites:**
- Complete [Setup & Authentication](/documentation/demo-terminal-setup)
- Get the **plugin token** from the dashboard (Step 4 in setup)

```bash
TOKEN="<your-plugin-token>"
URL="http://localhost:8080"  # or your deployed instance URL
```

This demo uses the plugin token (not the CLI auth token). The plugin token authenticates as the installed plugin, which is how MCP servers authenticate in production.

---

## Part 1: Governance — Allowed (Clean Input)

An admin agent reading a source file. No secrets, valid scope:

```bash
curl -s -X POST "$URL/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Read",
    "agent_id": "developer_agent",
    "session_id": "demo-mcp",
    "tool_input": {
      "file_path": "/src/main.rs"
    }
  }' | python3 -m json.tool
```

**Expected:** `decision: allow` — clean input, all rules passed.

> **Why baseline verification:** Before testing denial, confirm the allow path works. This establishes that the rule engine is functioning and clean inputs pass through. If this call fails, the deny tests would be meaningless.

---

## Part 2: Governance — Denied (Secret in Input)

The same admin agent, but the tool input contains an AWS access key:

```bash
curl -s -X POST "$URL/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Bash",
    "agent_id": "developer_agent",
    "session_id": "demo-mcp",
    "tool_input": {
      "command": "curl -H \"Authorization: AKIAIOSFODNN7EXAMPLE\" https://s3.amazonaws.com/bucket"
    }
  }' | python3 -m json.tool
```

**Expected:** `decision: deny` — AWS access key detected in tool input.

> **Why independent rules:** Scope doesn't override security. Even though the agent has admin scope, the secret_injection rule blocks the call. Rules are evaluated independently — this is defense in depth.

---

## Part 3: MCP Tool Call

Execute a real MCP tool call. This goes through OAuth authentication and is tracked in the database:

```bash
systemprompt plugins mcp call skill-manager list_plugins
```

The output shows the tool execution result including server name, tool name, execution time, and success status. This call is authenticated via the platform's OAuth layer and recorded as an MCP access event.

> **Why typed OAuth flow:** The OAuth exchange returns typed `TokenResponse { access_token, token_type, expires_in }`, not a raw JSON blob. `record_mcp_access()` takes typed `McpServerId` and `UserId` parameters — you can't accidentally log the wrong server or user.

> **Why dual audit tables:** Governance decisions and MCP access are stored in separate tables (`governance_decisions` and `user_activity`) because they serve different purposes. governance_decisions tracks policy enforcement. user_activity tracks usage patterns. Both use compile-time checked SQL (`sqlx::query!`).

---

## Part 4: Database Audit Queries

Query the database directly to see the governance decisions and MCP access events that were just created.

### Governance Decisions

```bash
systemprompt infra db query \
  "SELECT decision, tool_name, reason FROM governance_decisions ORDER BY created_at DESC LIMIT 5"
```

You should see the allow and deny decisions from Parts 1 and 2, plus any decisions from previous demos.

### MCP Access Events

```bash
systemprompt infra db query \
  "SELECT action, entity_name, description FROM user_activity WHERE category = 'mcp_access' ORDER BY created_at DESC LIMIT 5"
```

This shows the MCP tool call from Part 3, along with the OAuth authentication event.

---

## Dashboard

Open the dashboard and look for:

| Page / Tab | Section | What You See |
|---|---------|-------------|
| [/admin/](/admin/) Governance tab | **Policy Violations** | Governance denials with reason and timestamp |
| [/admin/](/admin/) MCP & Usage tab | **MCP Server Access** | Authentication events with Granted/Rejected badges |
| [/admin/](/admin/) MCP & Usage tab | **Live Activity** | Real-time stream of all platform events |
| [/admin/governance](/admin/governance) | **Governance Decisions** | Full list of allow/deny decisions with evaluation trace |

Every event that appeared in the database queries above is also visible on the dashboard. The dashboard is a live view of the same data.

---

## What This Proves

This demo shows the complete end-to-end audit chain:

1. **Governance evaluates tool calls before execution** — the allow/deny decision is made and recorded before any tool runs
2. **Secret detection blocks regardless of agent scope** — even the admin-scope developer_agent is denied when tool input contains a secret
3. **MCP tool calls are authenticated via OAuth** — every call goes through the platform's authentication layer
4. **Every event persists in the database** — governance decisions, MCP access events, and tool call results are all queryable via SQL
5. **The dashboard shows everything in real time** — the same data is available via CLI, SQL, and the web dashboard

---

## Complete Demo Summary

If you ran all terminal demos in sequence, you have now exercised:

| Demo | Commands Run | Events Generated |
|------|-------------|-----------------|
| [Agent Messaging](/documentation/demo-terminal-agents) | 2 agent messages | ~15 traced events, ~4 AI requests, 1 MCP tool call |
| [Audit Trails](/documentation/demo-terminal-audit) | Trace inspection + cost breakdown | Read-only queries |
| [Governance API](/documentation/demo-terminal-governance) | 6 curl calls | 6 governance decisions (4 deny, 2 allow) |
| [MCP Access Tracking](/documentation/demo-terminal-mcp) | 2 curl + 1 MCP + 2 DB queries | 2 governance decisions + 1 MCP access event |
| [Request Tracing](/documentation/demo-terminal-tracing) | Typed data flow + 200-request benchmark | Governance decisions, plugin usage, benchmark stats |

All events are visible on the [admin dashboard](/admin/).

---

## Next

Run [Request Tracing & Benchmark](/documentation/demo-terminal-tracing) for the deep-dive into typed data flow, all IDs, and production performance.
