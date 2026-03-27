---
title: "Terminal Demo: Governance API — Allow, Deny & Secret Detection"
description: "Call the governance endpoint directly with curl. Test scope-based rules, secret detection against AWS keys, GitHub PATs, and private keys, and see every decision on the dashboard."
author: "systemprompt.io"
slug: "demo-terminal-governance"
keywords: "demo, terminal, governance, api, curl, secrets, scope, allow, deny"
kind: "guide"
public: true
tags: ["demo", "terminal", "governance", "api", "secrets", "curl"]
published_at: "2026-03-27"
updated_at: "2026-03-27"
after_reading_this:
  - "Call the governance endpoint directly with curl"
  - "See how scope-based rules allow or deny tool calls"
  - "Test secret detection against AWS keys, GitHub PATs, and private keys"
  - "Understand the governance evaluation pipeline"
related_docs:
  - title: "Setup & Authentication"
    url: "/documentation/demo-terminal-setup"
  - title: "MCP Access Tracking Demo"
    url: "/documentation/demo-terminal-mcp"
  - title: "Tool Governance"
    url: "/documentation/tool-governance"
  - title: "Secrets"
    url: "/documentation/secrets"
---

> **See this in the presentation:** [Slide 9: Audit Trail & Access Control](/documentation/presentation#slide-9)

## Overview

These demos call the governance endpoint directly with curl. No agent session required. Each call evaluates the same governance pipeline that runs during live Claude Code sessions and returns an allow or deny decision with the evaluation trace.

**Prerequisites:** Get an auth token from [Setup & Authentication](/documentation/demo-terminal-setup):

```bash
TOKEN=$(systemprompt cloud auth token)
```

---

## The Governance Endpoint

```
POST /api/public/hooks/govern?plugin_id=enterprise-demo
Authorization: Bearer <token>
Content-Type: application/json
```

The request body describes the tool call being evaluated:

```json
{
  "hook_event_name": "PreToolUse",
  "tool_name": "<tool-name>",
  "agent_id": "<agent-name>",
  "session_id": "<session-id>",
  "tool_input": { }
}
```

The response contains the governance decision, reason, and the list of rules evaluated.

---

## Part 1: Governance Allows — Admin Scope

An admin-scope agent calling an MCP tool it has access to:

```bash
curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "mcp__systemprompt__list_agents",
    "agent_id": "developer_agent",
    "session_id": "demo-governance-happy"
  }' | python3 -m json.tool
```

**Expected:** `decision: allow` — all governance rules passed. The developer_agent has admin scope and the systemprompt MCP server is in its configuration.

---

## Part 2: Governance Denies — Scope Check

The same tool, but requested by a user-scope agent:

```bash
curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "mcp__systemprompt__list_agents",
    "agent_id": "associate_agent",
    "session_id": "demo-governance-denied"
  }' | python3 -m json.tool
```

**Expected:** `decision: deny` — the scope_check rule failed. The associate_agent has user scope, which does not grant access to admin-only MCP tools.

### Dashboard

Open [/admin/governance](http://localhost:8080/admin/governance). Both decisions appear with timestamps, the tool name, agent ID, and the full evaluation trace.

---

## Part 3: Secret Detection

The governance layer scans every `tool_input` field for plaintext secrets. If a secret pattern is detected, the call is blocked immediately — regardless of agent scope.

### Test 1: AWS Access Key

```bash
curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Bash",
    "agent_id": "developer_agent",
    "session_id": "demo-secret-breach",
    "tool_input": {
      "command": "curl -H \"Authorization: AKIAIOSFODNN7EXAMPLE\" https://s3.amazonaws.com/bucket",
      "description": "Fetch S3 object"
    }
  }' | python3 -m json.tool
```

**Expected:** `decision: deny` — secret_injection detected (AWS access key pattern `AKIA...`).

### Test 2: GitHub Personal Access Token

```bash
curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Write",
    "agent_id": "developer_agent",
    "session_id": "demo-secret-breach",
    "tool_input": {
      "file_path": "/home/user/.env",
      "content": "GITHUB_TOKEN=ghp_ABCDEFghijklmnop1234567890abcdef\nDATABASE_URL=postgres://localhost/db"
    }
  }' | python3 -m json.tool
```

**Expected:** `decision: deny` — secret_injection detected (GitHub PAT pattern `ghp_...`).

### Test 3: RSA Private Key

```bash
curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Write",
    "agent_id": "developer_agent",
    "session_id": "demo-secret-breach",
    "tool_input": {
      "file_path": "/home/user/.ssh/id_rsa",
      "content": "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA..."
    }
  }' | python3 -m json.tool
```

**Expected:** `decision: deny` — secret_injection detected (private key header).

### Test 4: Clean Input — Passes

```bash
curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Read",
    "agent_id": "developer_agent",
    "session_id": "demo-secret-breach",
    "tool_input": {
      "file_path": "/home/user/project/src/main.rs"
    }
  }' | python3 -m json.tool
```

**Expected:** `decision: allow` — no secrets detected, all rules passed.

---

## Summary

| # | Tool | Agent | Secret Type | Decision |
|---|------|-------|-------------|----------|
| 1 | mcp__systemprompt__list_agents | developer_agent | — | allow |
| 2 | mcp__systemprompt__list_agents | associate_agent | — | deny (scope_check) |
| 3 | Bash | developer_agent | AWS Access Key | deny (secret_injection) |
| 4 | Write | developer_agent | GitHub PAT | deny (secret_injection) |
| 5 | Write | developer_agent | RSA Private Key | deny (secret_injection) |
| 6 | Read | developer_agent | None (clean) | allow |

### Dashboard

Open [/admin/governance](http://localhost:8080/admin/governance). All 6 decisions are visible with:

- Timestamp
- Tool name and agent ID
- Decision (allow/deny)
- Policy that triggered the denial
- Full evaluation trace

---

## Governance Evaluation Pipeline

Every tool call passes through these rules in order. Any failure short-circuits the evaluation:

| Rule | What It Checks | Short-circuits? |
|------|---------------|-----------------|
| secret_detection | Scans `tool_input` for 37 secret patterns (AWS keys, GitHub tokens, Stripe keys, database connection strings, private keys, JWT tokens) | Yes — immediate deny |
| scope_check | Agent scope vs tool requirements (admin tools require admin scope) | Yes — immediate deny |
| tool_blocklist | Destructive operation patterns | Yes — immediate deny |
| rate_limit | Call frequency per agent | Yes — immediate deny |

Secret detection runs first. Even an admin-scope agent is blocked if tool input contains a plaintext secret.

---

## Next

Run [MCP Access Tracking](/documentation/demo-terminal-mcp) to combine governance calls with live MCP tool execution and database audit queries.
