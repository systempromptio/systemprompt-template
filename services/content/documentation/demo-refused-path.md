---
title: "Demo: Refused Path — User Agent Denied Admin MCP"
description: "Live CLI demo showing a user-scoped agent being denied access to the admin MCP server by the governance layer."
author: "systemprompt.io"
slug: "demo-refused-path"
keywords: "demo, refused path, denied, user, mcp, governance, rbac"
kind: "guide"
public: true
tags: ["demo", "governance", "mcp", "denied", "rbac"]
published_at: "2026-03-20"
updated_at: "2026-03-20"
after_reading_this:
  - "Run the refused path demo end-to-end"
  - "Understand what governance checks fail and why"
  - "Read the audit trail proving the request was denied"
related_docs:
  - title: "Happy Path"
    url: "/documentation/demo-happy-path"
  - title: "Detailed Breakdown"
    url: "/documentation/demo-breakdown"
  - title: "Access Control"
    url: "/documentation/access-control"
  - title: "Tool Governance"
    url: "/documentation/tool-governance"
---

## Overview

This demo sends the **same request** to `associate_agent`, which has **user** OAuth scope and access only to the **no** MCP server (not systemprompt). The governance layer blocks access to the admin CLI tool. The denial is fully logged.

**Cost:** This makes one real AI inference call (the model still processes the message, but the tool call is blocked).

---

## Prerequisites

```bash
systemprompt infra services status
```

Ensure the API and `associate_agent` are running.

---

## Step 1: Send the message

```bash
systemprompt admin agents message associate_agent \
  -m "List all agents running on this platform using the CLI tools" \
  --blocking --timeout 60
```

### What happens in the system

1. **Authentication** — The request is authenticated via OAuth2. `associate_agent` has scope `user`.
2. **RBAC check** — The ACL table is consulted. User scope has access to `associate_agent`. **Allowed** (the agent itself is accessible).
3. **Agent loaded** — `associate_agent` is initialized with its system prompt, skills (General Assistance only), and MCP server mapping (`no` only).
4. **AI inference** — The message is sent to the AI model. The model sees the available MCP tools — **only no tools are available**, not systemprompt.
5. **Tool governance** — The agent cannot see the `systemprompt` MCP server at all. It is not in the agent's `mcpServers` list. The governance layer enforces this at the mapping level — the tool simply does not exist for this agent.
6. **Response** — The agent either:
   - Explains it doesn't have access to CLI tools and cannot list agents
   - Attempts to answer from its own knowledge (without tool use)
   - Uses no tools (which don't help with listing agents)
7. **Audit** — Full trace recorded, showing no MCP tool call for `systemprompt` (because the mapping prevented it)

### What to look for in the output

- The agent should **not** return a real list of agents
- The response should indicate it cannot access the CLI tools or does not have the right permissions
- No `systemprompt` MCP tool call appears in the trace

---

## Step 2: View the audit trail

```bash
# List recent requests — find the request ID
systemprompt infra logs request list --limit 5

# Full audit trail
systemprompt infra logs audit <request-id> --full
```

### What to look for

- **Identity layer** — Same user, but agent is `associate_agent`
- **Agent layer** — `associate_agent`, user scope, no MCP only
- **Permissions layer** — Agent is accessible, but `systemprompt` MCP is NOT in the tool mapping
- **Tool layer** — Either empty (no tool calls) or shows only `no` calls (which don't help)
- **AI Request layer** — Tokens in/out, cost. The model still ran, but couldn't use the admin tools

### Compare with the happy path

Run both audit trails side by side:

```bash
# Happy path (developer_agent) — should show systemprompt tool call
systemprompt infra logs audit <happy-path-id> --full

# Refused path (associate_agent) — should show NO systemprompt tool call
systemprompt infra logs audit <refused-path-id> --full
```

The difference is the governance layer. Same question, different agent, different permissions, different outcome.

---

## Step 3: View the trace

```bash
systemprompt infra logs trace list --agent associate_agent --limit 3
```

The trace shows the agent received the message but did not make a `systemprompt` MCP tool call. The mapping prevented it.

---

## Why this matters

This is the core governance proposition: **access is explicitly declared, not defaulted.** `associate_agent` is a user-scope agent for frontline employees. It should not have access to the admin CLI. The governance layer enforces this without the agent having to know about it — the tool simply does not exist in its context.

If someone tries to reconfigure an agent to add unauthorized MCP servers, the change goes through the same governed pipeline: RBAC check, audit log, permission validation. The platform prevents privilege escalation by design.
