---
title: "Demo: Allowed Skill — Web Search Passes Governance"
description: "Live demo showing the example-web-search skill passing through the PreToolUse governance hook. The tool call is evaluated, allowed, and tracked."
author: "systemprompt.io"
slug: "demo-happy-path"
keywords: "demo, happy path, governance, allowed, web search, hook"
kind: "guide"
public: true
tags: ["demo", "governance", "allowed", "hooks"]
published_at: "2026-03-20"
updated_at: "2026-03-26"
after_reading_this:
  - "Run the allowed path demo end-to-end using Cowork"
  - "Understand what governance checks pass and why"
  - "Read the audit trail proving the tool call was allowed"
related_docs:
  - title: "Blocked Path"
    url: "/documentation/demo-refused-path"
  - title: "Detailed Breakdown"
    url: "/documentation/demo-breakdown"
  - title: "Tool Governance"
    url: "/documentation/tool-governance"
  - title: "Hooks"
    url: "/documentation/hooks"
---

## Overview

This demo uses Cowork (Claude Code) with the **enterprise-demo** plugin installed. The `example-web-search` skill instructs Claude to use the WebSearch tool. The PreToolUse governance hook evaluates the tool input, finds no policy violations, and allows the call. The full request is governed, traced, and logged.

**Cost:** This makes one real AI inference call plus one web search.

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

> "Search the web for the latest news about AI governance"

This triggers the `example-web-search` skill, which instructs Claude to use the WebSearch tool.

### What happens in the system

1. **Skill loaded** — Claude Code loads the `example-web-search` skill from the enterprise-demo plugin
2. **Tool selection** — Claude decides to call the WebSearch tool based on the skill instruction
3. **PreToolUse hook fires** — The HTTP hook sends the tool name and input to the governance endpoint (`/api/public/hooks/govern`)
4. **Governance evaluation** — The endpoint runs four rules in sequence:
   - Secret detection — scans tool input for API keys, tokens, passwords. **No match.**
   - Scope check — validates agent scope against tool restrictions. **Allowed.**
   - Tool blocklist — checks for destructive operations. **Not blocked.**
   - Rate limiting — checks call frequency. **Within limits.**
5. **Hook returns allow** — `permissionDecision: allow` sent back to Claude Code
6. **Tool executes** — WebSearch runs and returns results
7. **PostToolUse hook fires** — Async tracking hook logs the event to the platform
8. **Response** — Claude formats the search results and presents them to the user

### What to look for in the output

- Claude should return web search results about AI governance
- The response should indicate it used the WebSearch tool (not just answering from memory)

---

## Step 2: View the governance decision

Navigate to `/admin/governance` in the browser, or use the CLI:

```bash
# View recent governance decisions
systemprompt infra logs view --level info --since 10m
```

### What to look for

- **Tool name** — `WebSearch`
- **Decision** — `allow`
- **Policy** — No policy triggered (all rules passed)
- **Timestamp** — Matches when you ran the demo

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
- **Governance layer** — PreToolUse hook evaluated, decision: allow
- **Tool layer** — WebSearch called, status: OK, duration recorded
- **Tracking layer** — PostToolUse event logged with tool name, duration, and result status

---

## Step 4: View analytics

```bash
systemprompt analytics overview --since 1h
```

Shows: total events, tool calls, sessions, and costs. The web search from Step 1 should appear in these stats.

---

## Next

Run the [blocked path demo](/documentation/demo-refused-path) to see the same governance pipeline deny a tool call.
