---
title: "Demo: Detailed Breakdown — What Happens Under the Hood"
description: "Step-by-step technical breakdown of the governance pipeline for both the allowed and blocked demo paths, with CLI commands to inspect every layer."
author: "systemprompt.io"
slug: "demo-breakdown"
keywords: "demo, breakdown, audit, trace, analytics, governance, cli"
kind: "guide"
public: true
tags: ["demo", "governance", "audit", "analytics", "trace"]
published_at: "2026-03-20"
updated_at: "2026-03-26"
after_reading_this:
  - "Inspect every governance layer using CLI commands"
  - "Compare audit trails between allowed and denied tool calls"
  - "Understand the full request lifecycle from hook evaluation to analytics"
related_docs:
  - title: "Allowed Path"
    url: "/documentation/demo-happy-path"
  - title: "Blocked Path"
    url: "/documentation/demo-refused-path"
  - title: "Audit Trails & Events"
    url: "/documentation/events"
  - title: "Cost Tracking"
    url: "/documentation/cost-tracking"
---

## Overview

This page walks through exactly what happens in the governance pipeline for both demo paths, with CLI commands to inspect every layer. Run the allowed path and blocked path demos first, then use the commands below to examine the results.

> **Running from the terminal?** The [Governance API](/documentation/demo-terminal-governance) page shows the same governance pipeline using direct curl calls — no Cowork required.

---

## Setup: The two skills

| | example-web-search | use-dangerous-secret |
|---|---|---|
| **Purpose** | Demonstrates an allowed tool call | Demonstrates a blocked tool call |
| **Tool used** | WebSearch | Write (or similar) |
| **Governance outcome** | **Allow** — no policy violations | **Deny** — secret detected in tool input |
| **Policy triggered** | None | `secret_injection` |
| **Secret pattern** | N/A | `sk-ant-` prefix (Anthropic API key pattern) |

The key difference: both skills go through the same governance pipeline. The outcome depends on the **content** of the tool input, not the identity of the agent or the type of tool.

---

## Step 1: Run both demos in Cowork

Open Claude Code with the enterprise-demo plugin installed.

```
# Allowed path — web search passes governance
> Search the web for the latest news about AI governance

# Blocked path — secret detection blocks the tool call
> Use the dangerous secret skill to demonstrate secret detection
```

---

## Step 2: List recent governance decisions

Navigate to `/admin/governance` in the browser, or use the CLI:

```bash
systemprompt infra logs request list --limit 5
```

This shows the two requests you just made. Note the request IDs — you'll need them for the next steps.

**What to look for:**
- Both requests should appear with timestamps
- The web search request should show the tool call completed
- The secret detection request should show the tool call was blocked

---

## Step 3: Compare governance decisions

Navigate to `/admin/governance` and find both entries:

### Allowed path (web search) should show:

| Field | Value |
|-------|-------|
| **Tool** | WebSearch |
| **Decision** | allow |
| **Policy** | — |
| **Reason** | All governance rules passed |
| **Evaluated rules** | secret_detection: pass, scope_check: pass, tool_blocklist: pass, rate_limit: pass |

### Blocked path (secret detection) should show:

| Field | Value |
|-------|-------|
| **Tool** | Write |
| **Decision** | deny |
| **Policy** | secret_injection |
| **Reason** | Secret detected in tool input |
| **Evaluated rules** | secret_detection: **fail** (short-circuit) |

The critical difference: the governance endpoint evaluated the tool input content. The `sk-ant-` pattern triggered the secret detection rule, which short-circuited evaluation and returned a deny before the remaining rules were checked.

---

## Step 4: Compare audit trails

```bash
# Allowed path audit
systemprompt infra logs audit <allowed-request-id> --full

# Blocked path audit
systemprompt infra logs audit <blocked-request-id> --full
```

### Allowed path audit should show:

```
=== REQUEST TRACE ===
--- IDENTITY ---
User:         <your user>
Session:      ses-xxxxx

--- GOVERNANCE ---
Hook:         PreToolUse
Tool:         WebSearch
Decision:     allow
Rules:        secret_detection: pass, scope_check: pass, tool_blocklist: pass, rate_limit: pass

--- TOOL CALLS ---
1. WebSearch    → OK   (Xms)

--- TRACKING ---
PostToolUse event logged
```

### Blocked path audit should show:

```
=== REQUEST TRACE ===
--- IDENTITY ---
User:         <your user>
Session:      ses-xxxxx

--- GOVERNANCE ---
Hook:         PreToolUse
Tool:         Write
Decision:     deny
Policy:       secret_injection
Reason:       Secret detected — sk-ant-*** pattern matched
Rules:        secret_detection: FAIL (short-circuit)

--- TOOL CALLS ---
(blocked — tool never executed)
```

The critical difference: **no tool execution** in the blocked path. The governance hook denied the call before the tool could run.

---

## Step 5: View analytics

```bash
# Overall overview
systemprompt analytics overview --since 1h

# Cost breakdown
systemprompt analytics costs breakdown --by model
```

**What to look for:**
- Both requests appear as AI inference calls (the model processed both messages)
- The allowed path shows a tool call in the tool usage stats
- The blocked path shows a governance denial in the governance stats
- Cost tracking captures both requests (the AI still ran, even when the tool was blocked)

---

## Step 6: Inspect tool usage

```bash
# All tool calls
systemprompt analytics tools stats
```

This shows which tools were called, success/failure rates, and average durations. WebSearch should show one successful call. The blocked Write call should appear as a governance denial, not a tool failure.

---

## How to replicate

These steps are all you need:

```bash
# 1. Allowed path — invoke example-web-search in Cowork
# Ask: "Search the web for the latest news about AI governance"

# 2. Blocked path — invoke use-dangerous-secret in Cowork
# Ask: "Use the dangerous secret skill to demonstrate secret detection"

# 3. Compare
systemprompt infra logs request list --limit 5
systemprompt infra logs audit <allowed-path-id> --full
systemprompt infra logs audit <blocked-path-id> --full
```

The Cowork invocations make real AI inference calls. The audit and analytics commands are read-only and free.

---

## What this proves

1. **PreToolUse governance works** — Every tool call is evaluated by the governance endpoint before execution. The hook has full authority to allow or deny.
2. **Content-based policy enforcement works** — The governance decision is based on what the tool input contains, not just who is calling. Secret patterns are detected regardless of intent.
3. **Audit is complete** — Every governance decision is logged with the full evaluation context: which rules ran, which triggered, and why.
4. **Analytics are real** — Cost, token usage, governance decisions — all tracked per-tool, per-session, per-policy.
5. **The governance gap is filled** — You can answer: "What did the AI attempt to do, was it authorized, and if not, why was it blocked?"
