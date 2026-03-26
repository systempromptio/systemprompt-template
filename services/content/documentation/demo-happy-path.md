---
title: "Demo: Happy Path — Admin Agent with MCP Access"
description: "Live CLI demo showing an admin-scoped agent successfully using the systemprompt MCP server through the governed pipeline."
author: "systemprompt.io"
slug: "demo-happy-path"
keywords: "demo, happy path, admin, mcp, governance, agent"
kind: "guide"
public: true
tags: ["demo", "governance", "mcp", "admin"]
published_at: "2026-03-20"
updated_at: "2026-03-20"
after_reading_this:
  - "Run the happy path demo end-to-end"
  - "Understand what governance checks pass and why"
  - "Read the audit trail proving the request was authorized"
related_docs:
  - title: "Refused Path"
    url: "/documentation/demo-refused-path"
  - title: "Detailed Breakdown"
    url: "/documentation/demo-breakdown"
  - title: "Access Control"
    url: "/documentation/access-control"
  - title: "MCP Servers"
    url: "/documentation/mcp-servers"
---

## Overview

This demo sends a real message to `developer_agent`, which has **admin** OAuth scope and access to the **systemprompt** MCP server (the CLI executor tool). The agent uses the MCP tool to answer the question. The full request is governed, traced, and logged.

**Cost:** This makes one real AI inference call.

---

## Prerequisites

```bash
systemprompt infra services status
```

Ensure the API and `developer_agent` are running.

---

## Step 1: Send the message

```bash
systemprompt admin agents message developer_agent \
  -m "List all agents running on this platform" \
  --blocking --timeout 60
```

### What happens in the system

1. **Authentication** — The request is authenticated via OAuth2. `developer_agent` has scope `admin`.
2. **RBAC check** — The ACL table is consulted. Admin scope has access to `developer_agent` and the `enterprise-demo` plugin. **Allowed.**
3. **Agent loaded** — `developer_agent` is initialized with its system prompt, skills (General Assistance, Rust Standards, Architecture Standards), and MCP server mapping (`systemprompt`).
4. **AI inference** — The message is sent to the AI model (Claude). The model sees the available MCP tools and decides to call the `systemprompt` CLI tool.
5. **MCP tool governance** — Before the tool call executes:
   - OAuth2 token validated for the `systemprompt` MCP server
   - Agent-tool mapping checked: `developer_agent` is mapped to `systemprompt`. **Allowed.**
   - Pre-hook fires (authenticate, rate-limit, log)
6. **Tool execution** — The `systemprompt` MCP server executes the CLI command (e.g., `admin agents list`)
7. **Post-hook** — Result logged, duration recorded
8. **Response** — The AI model formats the tool output and returns it to the user
9. **Audit** — Full 5-layer trace recorded: Identity, Agent Context, Permissions, Tool Execution, AI Request

### What to look for in the output

- The agent should return a list of agents (associate_agent, developer_agent, systemprompt_admin)
- The response should indicate it used a tool to get this information (not just guessing)

---

## Step 2: View the audit trail

```bash
# List recent requests — find the request ID
systemprompt infra logs request list --limit 5

# Full audit trail
systemprompt infra logs audit <request-id> --full
```

### What to look for

- **Identity layer** — Your user, admin role, session ID
- **Agent layer** — `developer_agent`, `enterprise-demo` plugin, model name
- **Permissions layer** — ACL check: `agent:developer_agent` + `role:admin` → **allow**
- **Tool layer** — `systemprompt` MCP server called, tool executed, duration, status: OK
- **AI Request layer** — Tokens in/out, cost, latency, status: completed

---

## Step 3: View the trace

```bash
systemprompt infra logs trace list --agent developer_agent --limit 3
systemprompt infra logs trace show <trace-id> --all
```

The trace shows every step the agent took: receiving the message, deciding to call the MCP tool, the tool call itself, and formatting the response.

---

## Step 4: View analytics

```bash
systemprompt analytics agents show developer_agent
```

Shows: total requests, success rate, average latency, token usage, cost. The request from Step 1 should appear in these stats.
