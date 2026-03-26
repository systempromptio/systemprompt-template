---
title: "Demo: Blocked Skill — Secret Detection Denies Tool Call"
description: "Live demo showing the use-dangerous-secret skill being blocked by the PreToolUse governance hook when a plaintext API key is detected in the tool input."
author: "systemprompt.io"
slug: "demo-refused-path"
keywords: "demo, refused path, denied, secret detection, governance, hook"
kind: "guide"
public: true
tags: ["demo", "governance", "denied", "hooks", "secrets"]
published_at: "2026-03-20"
updated_at: "2026-03-26"
after_reading_this:
  - "Run the blocked path demo end-to-end using Cowork"
  - "Understand what governance checks fail and why"
  - "Read the audit trail proving the tool call was denied"
related_docs:
  - title: "Allowed Path"
    url: "/documentation/demo-happy-path"
  - title: "Detailed Breakdown"
    url: "/documentation/demo-breakdown"
  - title: "Tool Governance"
    url: "/documentation/tool-governance"
  - title: "Secrets"
    url: "/documentation/secrets"
---

## Overview

This demo uses Cowork (Claude Code) with the **enterprise-demo** plugin installed. The `use-dangerous-secret` skill instructs Claude to write a file containing a plaintext API key (`sk-ant-demo-FAKE12345678901234567890`). The PreToolUse governance hook detects the secret pattern and **blocks the tool call before it executes**. The denial is fully logged.

**Cost:** This makes one real AI inference call (the model still processes the message, but the tool call is blocked by the governance hook).

---

## Prerequisites

```bash
# Platform services running
systemprompt infra services status

# Enterprise-demo plugin installed in Claude Code
claude plugin list
```

Ensure the platform is healthy and the enterprise-demo plugin appears in Claude Code's plugin list.

---

## Step 1: Invoke the skill in Cowork

Open Claude Code with the enterprise-demo plugin and ask:

> "Use the dangerous secret skill to demonstrate secret detection"

This triggers the `use-dangerous-secret` skill, which instructs Claude to write a file containing the test API key `sk-ant-demo-FAKE12345678901234567890`.

### What happens in the system

1. **Skill loaded** — Claude Code loads the `use-dangerous-secret` skill from the enterprise-demo plugin
2. **Tool selection** — Claude decides to call a tool (e.g., Write) with the secret value in the input
3. **PreToolUse hook fires** — The HTTP hook sends the tool name and input to the governance endpoint (`/api/public/hooks/govern`)
4. **Governance evaluation** — The endpoint runs the secret detection rule first:
   - Secret detection — scans tool input for API keys, tokens, passwords. **Match found: `sk-ant-` prefix pattern.**
   - Evaluation short-circuits — remaining rules are not evaluated
5. **Hook returns deny** — `permissionDecision: deny` with reason: "Secret detected in tool input" sent back to Claude Code
6. **Tool call blocked** — Claude Code prevents the tool from executing and displays the denial reason
7. **Governance decision logged** — The deny decision, policy (`secret_injection`), and redacted snippet are recorded in the governance audit trail

### What to look for in the output

- Claude should indicate the tool call was **blocked** by the governance hook
- The denial reason should reference secret detection
- No file should have been written — the tool never executed

---

## Step 2: View the governance decision

Navigate to `/admin/governance` in the browser. The most recent entry should show:

| Field | Value |
|-------|-------|
| **Tool** | Write (or whichever tool Claude attempted) |
| **Decision** | deny |
| **Policy** | secret_injection |
| **Reason** | Secret detected in tool input — `sk-ant-` pattern matched |

---

## Step 3: View the audit trail

```bash
# List recent requests — find the request ID
systemprompt infra logs request list --limit 5

# Full audit trail
systemprompt infra logs audit <request-id> --full
```

### What to look for

- **Identity layer** — Your user, session ID
- **Governance layer** — PreToolUse hook evaluated, decision: **deny**, policy: secret_injection
- **Tool layer** — Tool call attempted but blocked before execution
- **Evaluated rules** — Shows which rule triggered the denial and the redacted secret snippet

### Compare with the allowed path

Run both audit trails side by side:

```bash
# Allowed path (web search) — should show allow decision
systemprompt infra logs audit <allowed-path-id> --full

# Blocked path (secret) — should show deny decision
systemprompt infra logs audit <blocked-path-id> --full
```

The difference is the governance evaluation. Same pipeline, same hook, different content, different outcome.

---

## Why this matters

This is the core governance proposition: **policies evaluate tool call content in real time.** The agent was explicitly instructed to use the secret — the skill contains the API key in its instructions. But the governance hook intercepted the tool call, inspected the input, detected the secret pattern, and blocked execution before any data was exposed.

This demonstrates that enterprise governance can enforce content-based policies across all tool calls, regardless of what the agent or user intended. The governance endpoint evaluates 37 secret patterns covering AWS keys, GitHub tokens, Stripe keys, database connection strings, private keys, JWT tokens, and more.

If a user or skill attempts to pass sensitive data through a tool call, the governance layer catches it — even when the agent is explicitly told to do so. The policy always wins.
