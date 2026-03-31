---
title: "Terminal Demo: Setup & Authentication"
description: "Verify platform services, obtain authentication tokens, and access the admin dashboard. Required before running any terminal demo."
author: "systemprompt.io"
slug: "demo-terminal-setup"
keywords: "demo, terminal, setup, authentication, token, preflight"
kind: "guide"
public: true
tags: ["demo", "terminal", "authentication", "setup"]
published_at: "2026-03-27"
updated_at: "2026-03-31"
after_reading_this:
  - "Verify all platform services are running"
  - "Obtain an authentication token for API calls"
  - "Obtain a plugin token for MCP access tracking"
  - "Access the admin dashboard via magic link"
related_docs:
  - title: "Agent Messaging Demo"
    url: "/documentation/demo-terminal-agents"
  - title: "Governance API Demo"
    url: "/documentation/demo-terminal-governance"
  - title: "Authentication"
    url: "/documentation/authentication"
  - title: "Installation"
    url: "/documentation/installation"
  - title: "Request Tracing Demo"
    url: "/documentation/demo-terminal-tracing"
---

## Overview

These terminal demos let you exercise every governance feature using only a terminal. No Cowork session or Claude Code required. Each demo produces data that is immediately visible on the admin dashboard.

All commands assume the platform is built and running. Dashboard links below use relative paths — prepend your platform URL (e.g. `http://localhost:8080` for local dev).

### Why Preflight Matters

The platform runs as a set of coordinated services: 3 AI agents (subprocesses running Claude Code), 2 MCP servers (tool providers via TCP/stdio), and background jobs. The preflight check queries the service registry (PostgreSQL) with compile-time checked SQL (`sqlx::query!{}`) to verify all components are healthy. Without this, subsequent demos may fail silently — an agent message would hang waiting for an MCP server that isn't running.

---

## Step 1: Preflight Check

Verify all services are healthy:

```bash
systemprompt infra services status
```

You should see:

- **3 agents** running
- **2 MCP servers** running (systemprompt, skill-manager)

The agents map to the demo scenarios as follows:

| Service Agent | Agent ID | Scope | Used In |
|---|---|---|---|
| platform | developer_agent | admin | Demos 01, 04 |
| revenue | associate_agent | user | Demos 02, 05 |
| admin | admin_agent | admin | Background |

If anything is down:

```bash
systemprompt infra services cleanup --yes
systemprompt infra services start --kill-port-process
```

---

## Step 2: Access the Admin Dashboard

Navigate to [/admin/login](/admin/login) in your browser. Enter your email to receive a magic link. After authentication, the dashboard shows real-time metrics for every action you run from the terminal.

Key pages to keep open during demos:

| Page | URL | What It Shows |
|------|-----|---------------|
| Dashboard | `/admin/` | Governance overview, policy violations, MCP access, cost breakdown |
| Events | `/admin/events` | Full audit trail of all platform activity |
| Governance | `/admin/governance` | Every tool governance decision (allow/deny) |

---

## Step 3: Get the Plugin Token

The governance and MCP demos require a bearer token. To get it:

1. Open [/admin/](/admin/)
2. Click the **Share & Install** icon (connected dots) in the top-right corner
3. Select the **Cowork** tab
4. Click the **eye icon** to reveal the token, then click **Copy**

Save it as an environment variable:

```bash
TOKEN="<paste-your-plugin-token-here>"
```

You will use this token for the [Governance API](/documentation/demo-terminal-governance) demos (04-06) and the [MCP Access Tracking](/documentation/demo-terminal-mcp) demo (07).

---

## Audit

Verify the preflight succeeded:

```bash
systemprompt infra services status
```

Expected: 3 agents running, 2 MCP servers running, all background jobs healthy. If any service shows "stopped", restart with `systemprompt infra services start --kill-port-process`.

---

## Notes

- **JSON formatting:** curl responses pipe through `python3 -m json.tool` for readability. You can substitute `jq` if preferred.
- **Cost:** Each AI inference call costs approximately $0.01 (varies by configured model).
- **Isolation:** Each demo creates an isolated context so results do not interfere with each other.

---

## Demo Sequence

Run the demos in order. Each builds on the previous:

| Demo | Page | What It Covers |
|------|------|---------------|
| 01-02 | [Agent Messaging](/documentation/demo-terminal-agents) | Allowed vs refused agent paths |
| 03 | [Audit Trails & Costs](/documentation/demo-terminal-audit) | Inspect traces from demos 01-02 |
| 04-06 | [Governance API](/documentation/demo-terminal-governance) | Direct curl calls: allow, deny, secret detection |
| 07 | [MCP Access Tracking](/documentation/demo-terminal-mcp) | MCP tool calls + database audit queries |
| 08 | [Request Tracing & Benchmark](/documentation/demo-terminal-tracing) | Typed data flow, all IDs, 200-request benchmark |
