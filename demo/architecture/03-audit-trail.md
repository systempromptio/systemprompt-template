# Demo 03: Audit Trail — Architecture

## What it does

Queries the trace and cost systems to show what happened in Demos 01 and 02.

## Flow

```
  CLI: infra logs trace list --limit 2
    │
    ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Trace Store (PostgreSQL)                               │
  │  SELECT trace_id, agent, status, event_count            │
  │  FROM execution_traces                                  │
  │  ORDER BY created_at DESC LIMIT 2                       │
  │  (compile-time checked SQL via sqlx::query!{})          │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Trace Detail: infra logs trace show <id> --all         │
  │                                                         │
  │  Per trace shows:                                       │
  │  ├── AI requests (count, model, tokens, cost)           │
  │  ├── MCP tool calls (tool_name, server, duration)       │
  │  ├── Execution steps (step_type, timing)                │
  │  ├── Skills loaded (skill_id, source)                   │
  │  └── Governance decisions (allow/deny, rules)           │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Cost Breakdown: analytics costs breakdown --by agent   │
  │                                                         │
  │  Aggregates across traces:                              │
  │  ├── developer_agent: $X.XX (3 AI requests)             │
  │  └── associate_agent: $X.XX (1 AI request)              │
  └─────────────────────────────────────────────────────────┘
```

## Data Model

```
  execution_traces
  ├── trace_id     : TraceId (UUID, primary key)
  ├── agent        : AgentName (newtype)
  ├── user_id      : UserId (newtype)
  ├── session_id   : SessionId (newtype)
  ├── status       : TraceStatus (enum)
  ├── events       : Vec<TraceEvent> (JSONB)
  └── created_at   : DateTime<Utc>
```

## Why Rust

- **Compile-time SQL**: Every trace query uses `sqlx::query_as!{}` — the column names, types, and return struct are all verified at build time against the database schema
- **Typed aggregation**: Cost breakdown returns typed `CostBreakdown { agent: AgentName, total_cost: Decimal, request_count: i64 }` — not raw tuples
- **No audit gaps**: Because all IDs are newtypes, every event is guaranteed to carry the correct `TraceId` and `UserId` — you can't accidentally log with the wrong ID
