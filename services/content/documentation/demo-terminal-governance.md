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
updated_at: "2026-03-31"
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
  - title: "Request Tracing Demo"
    url: "/documentation/demo-terminal-tracing"
---

## Overview

These demos call the governance endpoint directly with curl. No agent session required. Each call evaluates the same governance pipeline that runs during live Claude Code sessions and returns an allow or deny decision with the evaluation trace.

**Prerequisites:** Get your plugin token from [Setup & Authentication — Step 3](/documentation/demo-terminal-setup):

```bash
TOKEN="<paste-your-plugin-token-here>"
URL="http://localhost:8080"  # or your deployed instance URL
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
curl -s -X POST "$URL/api/public/hooks/govern?plugin_id=enterprise-demo" \
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

> **Why three independent rules:** The rule engine evaluates scope_check, secret_injection, and rate_limit independently. Even if scope allows access, secret injection still blocks leaked credentials. Rate limiting prevents runaway agents. Rules return typed `Vec<RuleEvaluation>` structs, not loose JSON.

> **Why async audit:** `tokio::spawn` fires the database write without blocking the HTTP response. The governance decision returns in ~12ms while the audit record writes asynchronously. This keeps the hot path fast — the caller never waits for the audit INSERT.

---

## Part 2: Governance Denies — Scope Check

The same tool, but requested by a user-scope agent:

```bash
curl -s -X POST "$URL/api/public/hooks/govern?plugin_id=enterprise-demo" \
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

> **Why short-circuit evaluation:** The first rule failure stops evaluation. No point checking secrets or rate limits if scope already denies. This keeps governance fast and avoids unnecessary work.

> **Why denials are audited:** Denied calls are audited with the same fidelity as approvals. The `evaluated_rules` JSONB column stores exactly which rule failed and why. This is critical for compliance — you need to prove that denials happen correctly.

### Dashboard

Open [/admin/governance](/admin/governance). Both decisions appear with timestamps, the tool name, agent ID, and the full evaluation trace.

---

## Part 3: Secret Detection

The governance layer scans every `tool_input` field for plaintext secrets. If a secret pattern is detected, the call is blocked immediately — regardless of agent scope.

### Test 1: AWS Access Key

```bash
curl -s -X POST "$URL/api/public/hooks/govern?plugin_id=enterprise-demo" \
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
curl -s -X POST "$URL/api/public/hooks/govern?plugin_id=enterprise-demo" \
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
curl -s -X POST "$URL/api/public/hooks/govern?plugin_id=enterprise-demo" \
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
curl -s -X POST "$URL/api/public/hooks/govern?plugin_id=enterprise-demo" \
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

> **Why recursive scanning:** The secret scanner traverses `serde_json::Value` recursively — ALL nested strings in tool_input are checked, not just top-level fields. An attacker can't hide a key in a nested JSON object.

> **Why audit without leaking:** The audit record stores the secret TYPE ("AWS access key") but NOT the actual secret value. Typed Rust structs enforce this separation at compile time.

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

Open [/admin/governance](/admin/governance). All 6 decisions are visible with:

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
| secret_detection | Scans `tool_input` for 37 secret patterns (AWS keys, GitHub tokens, Stripe keys, database connection strings, private keys, JWT tokens). Policy: `secret_injection` | Yes — immediate deny |
| scope_check | Agent scope vs tool requirements (admin tools require admin scope). Policy: `scope_restriction` | Yes — immediate deny |
| tool_blocklist | Destructive operation patterns (delete/drop/destroy) for non-admin scopes. Policy: `tool_blocklist` | Yes — immediate deny |
| rate_limit | Call frequency per session (300 calls/min). Policy: `rate_limit` | Yes — immediate deny |

Secret detection runs first. Even an admin-scope agent is blocked if tool input contains a plaintext secret. All four rules are evaluated independently; the first failure short-circuits evaluation.

---

## Quick Demo: Copy-Paste Script

Copy this entire block into Claude Code (or any terminal) to run all six governance tests in sequence. Replace `YOUR_TOKEN_HERE` with the plugin token from [Setup — Step 3](/documentation/demo-terminal-setup).

```bash
# ── Set your token and URL ──────────────────────────────────────
TOKEN="YOUR_TOKEN_HERE"
URL="http://localhost:8080"  # or your deployed instance URL
API="$URL/api/public/hooks/govern?plugin_id=enterprise-demo"

# ── Test 1: Admin scope → ALLOW ─────────────────────────────────
# The developer_agent has admin scope, so it can call any MCP tool.
echo "── Test 1: Admin agent calls admin MCP tool ──"
curl -s -X POST "$API" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "mcp__systemprompt__list_agents",
    "agent_id": "developer_agent",
    "session_id": "demo-governance"
  }' | python3 -m json.tool
echo ""

# ── Test 2: User scope → DENY (scope_restriction) ──────────────
# The associate_agent has user scope — admin-only tools are blocked.
echo "── Test 2: User agent calls admin MCP tool ──"
curl -s -X POST "$API" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "mcp__systemprompt__list_agents",
    "agent_id": "associate_agent",
    "session_id": "demo-governance"
  }' | python3 -m json.tool
echo ""

# ── Test 3: AWS key in tool input → DENY (secret_injection) ────
# The governance layer scans every field in tool_input for secrets.
# An AWS access key (AKIA...) is detected and the call is blocked
# before the tool ever executes.
echo "── Test 3: Bash command contains AWS access key ──"
curl -s -X POST "$API" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Bash",
    "agent_id": "developer_agent",
    "session_id": "demo-governance",
    "tool_input": {
      "command": "curl -H \"Authorization: AKIAIOSFODNN7EXAMPLE\" https://s3.amazonaws.com/bucket",
      "description": "Fetch S3 object"
    }
  }' | python3 -m json.tool
echo ""

# ── Test 4: GitHub PAT in file content → DENY (secret_injection)
# A Write tool call attempts to create a .env file containing a
# GitHub personal access token (ghp_...). Blocked immediately.
echo "── Test 4: Write .env file containing GitHub PAT ──"
curl -s -X POST "$API" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Write",
    "agent_id": "developer_agent",
    "session_id": "demo-governance",
    "tool_input": {
      "file_path": "/home/user/.env",
      "content": "GITHUB_TOKEN=ghp_ABCDEFghijklmnop1234567890abcdef\nDATABASE_URL=postgres://localhost/db"
    }
  }' | python3 -m json.tool
echo ""

# ── Test 5: RSA private key → DENY (secret_injection) ──────────
# Even an admin-scope agent is blocked. Secret detection runs first
# and overrides all other rules.
echo "── Test 5: Write SSH private key file ──"
curl -s -X POST "$API" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Write",
    "agent_id": "developer_agent",
    "session_id": "demo-governance",
    "tool_input": {
      "file_path": "/home/user/.ssh/id_rsa",
      "content": "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA..."
    }
  }' | python3 -m json.tool
echo ""

# ── Test 6: Clean input → ALLOW ─────────────────────────────────
# A normal Read call with no secrets in the input. All four rules
# pass and the tool is allowed to execute.
echo "── Test 6: Read a source file (clean input) ──"
curl -s -X POST "$API" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Read",
    "agent_id": "developer_agent",
    "session_id": "demo-governance",
    "tool_input": {
      "file_path": "/home/user/project/src/main.rs"
    }
  }' | python3 -m json.tool

# ── Check the dashboard ─────────────────────────────────────────
echo ""
echo "All 6 decisions are now visible at:"
echo "  /admin/governance"
echo ""
echo "Expected results:"
echo "  Test 1: ALLOWED  (admin scope, clean input)"
echo "  Test 2: DENIED   (user scope → scope_restriction)"
echo "  Test 3: DENIED   (AWS key → secret_injection)"
echo "  Test 4: DENIED   (GitHub PAT → secret_injection)"
echo "  Test 5: DENIED   (RSA key → secret_injection)"
echo "  Test 6: ALLOWED  (admin scope, clean input)"
```

### What to show the audience

After running the script, open [/admin/governance](/admin/governance). The metric ribbon updates immediately:

- **Total Decisions** increases by 6
- **Denied** increases by 4 (scope + 3 secrets)
- **Secret Breaches** increases by 3

Scroll the table to show the decision trail. Each row shows the tool name, the agent that requested it, the decision badge (ALLOWED / DENIED / SECRET BREACH), the policy that triggered the denial, and the timestamp. Click any user ID to see their full activity history.

The key takeaway: **the same governance pipeline that blocked these curl calls runs on every tool call in a live Claude Code session.** Policies evaluate content in real time, secrets are caught before the tool executes, and every decision is auditable.

---

## Audit

Verify governance decisions were recorded correctly:

```bash
# Governance: Check most recent allow/deny decisions
systemprompt infra db query \
  "SELECT decision, tool_name, policy, reason FROM governance_decisions ORDER BY created_at DESC LIMIT 5"

# Governance: Verify secret breach counts (should be 3 deny + 1 allow)
systemprompt infra db query \
  "SELECT decision, COUNT(*) as count FROM governance_decisions WHERE session_id = 'demo-secret-breach' GROUP BY decision ORDER BY decision"

# Full detail for secret breach tests
systemprompt infra db query \
  "SELECT decision, tool_name, policy, reason FROM governance_decisions WHERE session_id = 'demo-secret-breach' ORDER BY created_at"
```

**Expected results:**

| Test | Decision | Policy | Reason |
|------|----------|--------|--------|
| Governance allow | allow | default_allow | admin scope, all rules passed |
| Governance deny | deny | scope_restriction | user scope cannot access admin tools |
| Secret breach (tests 1-3) | deny | secret_injection | AWS key / GitHub PAT / PEM key detected |
| Secret breach (test 4) | allow | default_allow | clean input, no secrets found |

---

## Next

Run [MCP Access Tracking](/documentation/demo-terminal-mcp) to combine governance calls with live MCP tool execution and database audit queries.
