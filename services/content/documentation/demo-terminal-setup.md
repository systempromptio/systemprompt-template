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

All commands assume `http://localhost:8080` and that the platform is built and running.

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

## Step 2: Get an Authentication Token

The governance API requires a bearer token. Generate one from the CLI:

```bash
TOKEN=$(systemprompt cloud auth token)
echo "$TOKEN"
```

This token is short-lived. If you get a 401 response on any curl call, re-run the command above.

You will use this token for the [Governance API](/documentation/demo-terminal-governance) demos (04, 05, 06).

---

## Step 3: Access the Admin Dashboard

Navigate to [http://localhost:8080/admin/login](http://localhost:8080/admin/login) in your browser. Enter your email to receive a magic link. After authentication, the dashboard shows real-time metrics for every action you run from the terminal.

Key pages to keep open during demos:

| Page | URL | What It Shows |
|------|-----|---------------|
| Dashboard | `/admin/` | Metric ribbon, AI usage, cost breakdown |
| Events | `/admin/events` | Full audit trail of all platform activity |
| Governance | `/admin/governance` | Every tool governance decision (allow/deny) |

---

## Step 4: Get the Plugin Token (for Demo 07)

The [MCP Access Tracking](/documentation/demo-terminal-mcp) demo uses a **plugin token** instead of the CLI auth token. To get it:

1. Open [http://localhost:8080/admin/](http://localhost:8080/admin/)
2. Click the **key icon** in the top-right corner
3. Reveal and copy the plugin token

Save it for later:

```bash
PLUGIN_TOKEN="<paste-your-plugin-token-here>"
```

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
