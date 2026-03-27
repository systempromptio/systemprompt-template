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
updated_at: "2026-03-27"
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
---

## Overview

These terminal demos let you exercise every governance feature using only a terminal. No Cowork session or Claude Code required. Each demo produces data that is immediately visible on the admin dashboard.

All commands assume `https://abc3dd581f80.systemprompt.io` and that the platform is built and running.

---

## Step 1: Preflight Check

Verify all services are healthy:

```bash
systemprompt infra services status
```

You should see:

- **3 agents** running (platform, revenue, admin)
- **2 MCP servers** running (systemprompt, skill-manager)

If anything is down:

```bash
systemprompt infra services cleanup --yes
systemprompt infra services start --kill-port-process
```

---

## Step 2: Access the Admin Dashboard

Navigate to [https://abc3dd581f80.systemprompt.io/admin/login](https://abc3dd581f80.systemprompt.io/admin/login) in your browser. Enter your email to receive a magic link. After authentication, the dashboard shows real-time metrics for every action you run from the terminal.

Key pages to keep open during demos:

| Page | URL | What It Shows |
|------|-----|---------------|
| Dashboard | `/admin/` | Metric ribbon, AI usage, cost breakdown |
| Events | `/admin/events` | Full audit trail of all platform activity |
| Governance | `/admin/governance` | Every tool governance decision (allow/deny) |

---

## Step 3: Get the Plugin Token

The governance and MCP demos require a bearer token. To get it:

1. Open [https://abc3dd581f80.systemprompt.io/admin/](https://abc3dd581f80.systemprompt.io/admin/)
2. Click the **Share & Install** icon (connected dots) in the top-right corner
3. Select the **Cowork** tab
4. Click the **eye icon** to reveal the token, then click **Copy**

Save it as an environment variable:

```bash
TOKEN="<paste-your-plugin-token-here>"
```

You will use this token for the [Governance API](/documentation/demo-terminal-governance) demos (04-06) and the [MCP Access Tracking](/documentation/demo-terminal-mcp) demo (07).

---

## Notes

- **JSON formatting:** curl responses pipe through `python3 -m json.tool` for readability. You can substitute `jq` if preferred.
- **Cost:** Each AI inference call costs approximately $0.01 on Gemini Flash.
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
