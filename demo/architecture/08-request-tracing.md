# Demo 08: Request Tracing — Architecture

## What it does

End-to-end request tracing showing typed data, all IDs, log commands, system flow, and a 100-request benchmark.

## Part 1: Typed Data Flow

```
  ── HTTP BOUNDARY (the only untyped moment) ──

  curl sends JSON:
  {
    "hook_event_name": "PreToolUse",     ← String
    "tool_name": "Read",                  ← String
    "agent_id": "developer_agent",        ← String
    "session_id": "demo-trace-1711...",   ← String
    "tool_input": {"file_path": "..."}    ← arbitrary JSON
  }
    │
    ▼
  Axum handler: Json(raw) → serde::from_value()
    │
    ▼
  ── TYPED FROM HERE ON ──

  HookEventPayload {
    event: HookEvent::PreToolUse(PreToolUseEvent {
      tool_name: String,
      tool_input: serde_json::Value,
    }),
    common: CommonFields {
      session_id: String,        → SessionId::new()
      agent_id: Option<String>,  → AgentName::new()
    },
  }
    │
    ▼
  GovernanceContext {
    user_id: UserId,           ← from JWT claims (newtype)
    session_id: SessionId,     ← from payload (newtype)
    agent_id: AgentName,       ← from payload (newtype)
    agent_scope: String,       ← resolved from DB
    tool_name: String,
    tool_input: Value,
  }
    │
    ▼
  Vec<RuleEvaluation> {
    rule: String,
    result: "pass" | "fail",
    detail: String,
  }
    │
    ▼
  GovernanceResponse {
    hook_specific_output: HookSpecificOutput {
      hook_event_name: String,
      permission_decision: String,     ← "allow" | "deny"
      permission_decision_reason: Option<String>,
    }
  }
    │
    ▼
  serde::to_value() → JSON response

  ── HTTP BOUNDARY (back to untyped) ──
```

## Part 2: All Typed IDs

```
  ┌──────────────────┬──────────────────────┬─────────────────────────────┐
  │ ID               │ Rust Type            │ Source                      │
  ├──────────────────┼──────────────────────┼─────────────────────────────┤
  │ decision_id      │ String (UUID v4)     │ Server-generated PK         │
  │ user_id          │ UserId(String)       │ JWT 'sub' claim             │
  │ session_id       │ SessionId(String)    │ Client payload              │
  │ agent_id         │ AgentName(String)    │ Client payload              │
  │ plugin_id        │ PluginId(String)     │ Query parameter             │
  │ trace_id         │ TraceId(String)      │ Per-request UUID            │
  │ context_id       │ ContextId(String)    │ CLI context creation        │
  └──────────────────┴──────────────────────┴─────────────────────────────┘

  Newtype pattern:
    pub struct UserId(String);
    pub struct SessionId(String);
    // UserId and SessionId are DIFFERENT types
    // fn record(user: &UserId, session: &SessionId) — compiler enforced
```

## Part 3: Log Commands

```
  infra logs trace list     → execution_traces table (TraceId, AgentName)
  infra logs trace show     → trace events (AI requests, tool calls, steps)
  infra logs request list   → ai_requests table (model, tokens, cost)
  infra logs view           → application logs (tracing crate output)
```

## Part 4: Request Flow (6 stages)

```
  Client → Router → JWT → Scope → Rules → Audit → Response
           │         │      │       │       │
           │         │      │       │       └─ sqlx::query!{} INSERT
           │         │      │       └─ Vec<RuleEvaluation> (typed)
           │         │      └─ GovernanceContext (typed)
           │         └─ JwtClaims { sub: UserId } (typed)
           └─ HookEventPayload (serde deserialized, typed)
```

## Part 5: Benchmark

```
  100 concurrent curl requests
    │
    ▼
  Each request does:
    1. JWT validation (jsonwebtoken crate, ~0.1ms)
    2. Scope resolution (DB query, sqlx::query!{})
    3. Rule evaluation (3 rules, in-memory)
    4. Audit write (tokio::spawn, async INSERT)
    5. Response serialization (serde)
    │
    ▼
  Metrics collected:
    • Wall-clock time (all 100 requests)
    • Throughput (requests/second)
    • Per-request latency (min, max, avg)
    • Success rate (HTTP 200 count)
    • DB verification (SELECT COUNT(*) GROUP BY decision)
    │
    ▼
  + Single-request latency test (5 sequential requests)
    Shows true per-request cost without contention
```

## Why Rust (Summary)

| Property | How Rust delivers it |
|----------|---------------------|
| Type safety | Newtype IDs (`UserId != SessionId`) enforced at compile time |
| Data integrity | Serde deserializes JSON into validated structs at the boundary |
| Query safety | `sqlx::query!{}` checks every SQL query against the live schema at compile time |
| Performance | Zero-cost abstractions: newtypes compile to bare strings, no boxing |
| Concurrency | Tokio async runtime handles 100 concurrent connections with work-stealing |
| Audit completeness | Typed structs guarantee every record has the correct IDs — no field can be accidentally omitted |
| No GC pauses | Ownership model eliminates garbage collection — predictable latency under load |
