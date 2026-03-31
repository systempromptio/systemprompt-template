---
title: "Terminal Demo: Request Tracing & Benchmark"
description: "Trace typed data flow through the governance pipeline, inspect all newtype IDs, use CLI log commands, and run a 200-request production benchmark with latency percentiles."
author: "systemprompt.io"
slug: "demo-terminal-tracing"
keywords: "demo, terminal, tracing, benchmark, typed, ids, performance, latency"
kind: "guide"
public: true
tags: ["demo", "terminal", "tracing", "benchmark", "performance", "typed-ids"]
published_at: "2026-03-31"
updated_at: "2026-03-31"
after_reading_this:
  - "Trace typed data flow through the governance pipeline"
  - "Inspect all newtype IDs created by a request"
  - "Use CLI log commands for observability"
  - "Run a 200-request benchmark and read latency percentiles"
  - "Estimate enterprise capacity from benchmark numbers"
related_docs:
  - title: "MCP Access Tracking Demo"
    url: "/documentation/demo-terminal-mcp"
  - title: "Governance API Demo"
    url: "/documentation/demo-terminal-governance"
  - title: "Setup & Authentication"
    url: "/documentation/demo-terminal-setup"
  - title: "Tool Governance"
    url: "/documentation/tool-governance"
---

## Overview

This is the deep-dive demo. It traces how data flows through the Rust typed pipeline, shows every ID the system creates, demonstrates CLI observability commands, and runs a production-grade benchmark. No AI calls — pure infrastructure.

**Prerequisites:** Get your plugin token from [Setup & Authentication — Step 3](/documentation/demo-terminal-setup):

```bash
TOKEN="<paste-your-plugin-token-here>"
URL="http://localhost:8080"  # or your deployed instance URL
```

**Cost:** Free (governance API calls + DB queries, no AI).

---

## Part 1: Typed Data Flow

Send a governance request and a tracking request, observing the full payloads:

### 1a. Governance Request

```bash
SESSION_ID="demo-trace-$(date +%s)"

curl -s -w "\nHTTP %{http_code} in %{time_total}s" \
  -X POST "$URL/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Read",
    "agent_id": "developer_agent",
    "session_id": "'$SESSION_ID'",
    "tool_input": {"file_path": "/src/main.rs"}
  }' | python3 -m json.tool
```

### 1b. Track Request

```bash
curl -s -o /dev/null -w "HTTP %{http_code} in %{time_total}s\n" \
  -X POST "$URL/api/public/hooks/track?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PostToolUse",
    "tool_name": "Read",
    "agent_id": "developer_agent",
    "session_id": "'$SESSION_ID'",
    "tool_input": {"file_path": "/src/main.rs"},
    "tool_result": "fn main() { println!(\"Hello\"); }"
  }'
```

> **Why a single untyped boundary:** JSON is untyped ONLY at the HTTP edge. `serde::from_value()` immediately converts to `HookEventPayload` — a validated Rust struct. From that point, `session_id` becomes `SessionId` (newtype). The compiler prevents passing it where a `UserId` is expected. Zero runtime cost for this type safety.

---

## Part 2: All Typed IDs

Query the database to see every ID created by the requests:

```bash
systemprompt infra db query \
  "SELECT id, user_id, session_id, agent_id, decision, plugin_id, created_at FROM governance_decisions WHERE session_id = '$SESSION_ID' ORDER BY created_at DESC LIMIT 3"
```

```bash
systemprompt infra db query \
  "SELECT id, user_id, session_id, event_type, tool_name, created_at FROM plugin_usage_events WHERE session_id = '$SESSION_ID' ORDER BY created_at DESC LIMIT 3"
```

### ID Reference

| ID | Rust Type | Source |
|---|---|---|
| decision_id | `String` (UUID v4) | Server-generated primary key |
| user_id | `UserId(String)` | JWT 'sub' claim |
| session_id | `SessionId(String)` | Client payload |
| agent_id | `AgentName(String)` | Hook payload |
| plugin_id | `PluginId(String)` | Query parameter |
| trace_id | `TraceId(String)` | Per-request UUID |
| context_id | `ContextId(String)` | CLI context creation |

> **Why newtypes:** `pub struct UserId(String)` and `pub struct SessionId(String)` are different types. `fn record(user: &UserId, session: &SessionId)` — the compiler enforces correct ID types at every call site. You cannot pass a SessionId where a UserId is expected. Zero runtime cost — newtypes compile to bare strings.

> **Why compile-time SQL:** Every SQL query that writes these rows uses `sqlx::query!{}` macros — checked against the live database schema at compile time. If a column is renamed or a type changes, the code won't compile.

---

## Part 3: CLI Log Commands

Four views into the same data, from summary to full detail:

```bash
# Trace list — recent execution traces
systemprompt infra logs trace list --limit 3

# Trace detail — all events for a specific trace
TRACE_ID=$(systemprompt infra logs trace list --limit 1 2>&1 | grep -oP '"trace_id":\s*"\K[0-9a-f-]+' | head -1)
systemprompt infra logs trace show "$TRACE_ID" --all

# AI request log — model, tokens, cost
systemprompt infra logs request list --limit 5

# Application logs — structured tracing output
systemprompt infra logs view --level info --since 5m
```

> **Why observable by default:** Every request is traced. No special debug mode needed. The CLI provides 4 different views into the same data, from summary (`trace list`) to full detail (`trace show --all`).

---

## Part 4: Request Flow Map

Every governance request passes through 6 typed stages:

| Stage | Rust Struct | What Happens |
|---|---|---|
| Router | `HookEventPayload` | JSON deserialized via serde |
| JWT | `JwtClaims { sub: UserId }` | Token validated, user extracted |
| Scope | `GovernanceContext` | Agent scope resolved from DB |
| Rules | `Vec<RuleEvaluation>` | scope_check, secret_injection, rate_limit |
| Audit | `sqlx::query!{} INSERT` | Async DB write via tokio::spawn |
| Response | `GovernanceResponse` | Serialized back to JSON |

> **Why typed at every stage:** No dictionaries, no raw strings, no unvalidated JSON passes between stages. If you change a column name in a migration, every query that touches it fails to compile. The GovernanceContext carries UserId + SessionId + AgentName (newtypes). The RuleEvaluation returns typed enums.

---

## Part 5: Production Benchmark

Fire 200 parallel requests to measure real throughput and latency:

```bash
# Install hey (HTTP load testing tool)
curl -sL https://hey-release.s3.us-east-2.amazonaws.com/hey_linux_amd64 -o /tmp/hey && chmod +x /tmp/hey

BENCH_SESSION="bench-$(date +%s)"

# Governance endpoint — 200 requests, 100 concurrent
/tmp/hey -n 200 -c 100 -m POST \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"Read","agent_id":"developer_agent","session_id":"'$BENCH_SESSION'","tool_input":{"file_path":"/src/main.rs"}}' \
  "$URL/api/public/hooks/govern?plugin_id=enterprise-demo"
```

### What to Look For

| Metric | Typical Value | Why It Matters |
|---|---|---|
| Throughput | 500-2000 req/s | Single instance capacity |
| p50 latency | 2-10ms | Median governance decision time |
| p90 latency | 10-30ms | Tail latency under load |
| p99 latency | 30-80ms | Worst-case (DB pool contention) |

### Enterprise Capacity Estimate

At N req/s (single instance), assuming 10 tool calls/min per developer:

| Deployment | Concurrent Developers |
|---|---|
| 1 instance | ~N × 6 |
| 3 instances + PgBouncer | ~N × 18 |
| 10 instances + PgBouncer | ~N × 60 |

The governance check (p50 ≈ 5ms) adds <2% overhead to Claude's AI response time (1-5 seconds).

### Verify Benchmark in Database

```bash
systemprompt infra db query \
  "SELECT decision, COUNT(*) as count FROM governance_decisions WHERE session_id = '${BENCH_SESSION}' GROUP BY decision"
```

Expected: 200 allow decisions (all clean Read inputs from admin-scope agent).

> **Why zero-cost abstractions:** Newtypes compile to bare strings — no boxing, no indirection. Tokio async runtime handles 100 concurrent connections with work-stealing. In-memory rate limiter: zero DB round-trips in the hot path. OnceLock scope cache: agent config loaded once, not per-request. `tokio::spawn` audit writes: non-blocking async DB INSERT.

---

## Audit

Verify all benchmark data was recorded:

```bash
# Decision counts from benchmark
systemprompt infra db query \
  "SELECT decision, COUNT(*) FROM governance_decisions WHERE session_id LIKE 'bench-%' GROUP BY decision"

# Recent traces
systemprompt infra logs trace list --limit 3

# Dashboard
echo "Open: $URL/admin/"
```

---

## What This Proves

1. **Type safety** — JSON is untyped only at the HTTP boundary; everything else is validated Rust structs with newtype IDs
2. **Query safety** — every SQL query is compile-time checked against the live schema
3. **Performance** — governance adds <2% overhead to AI response time; handles hundreds of developers per instance
4. **Observability** — every request is traced; 4 CLI commands provide summary through to full detail
5. **Audit completeness** — every decision, every ID, every event persists in the database and appears on the dashboard
