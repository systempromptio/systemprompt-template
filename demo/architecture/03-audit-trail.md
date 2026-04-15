# Demo 03: Audit Trail — Architecture

## What it does

Queries the governance_decisions table to show what happened in Demos 01 and 02 — the ALLOW and DENY decisions.

## Flow

```
  CLI: infra db query "SELECT ... FROM governance_decisions"
    │
    ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Governance Store (PostgreSQL)                          │
  │  SELECT decision, tool_name, agent_id, agent_scope,     │
  │         policy, reason                                  │
  │  FROM governance_decisions                              │
  │  ORDER BY created_at DESC LIMIT 5                       │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Expected Results                                       │
  │                                                         │
  │  Demo 01 (developer_agent):                             │
  │    decision=allow, scope=admin, policy=default_allow    │
  │                                                         │
  │  Demo 02 (associate_agent):                             │
  │    decision=deny, scope=user, policy=scope_restriction  │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Cost Breakdown: analytics costs breakdown --by agent   │
  │  Aggregates across all agent activity                   │
  └─────────────────────────────────────────────────────────┘
```

## Data Model

```
  governance_decisions
  ├── id             : String (UUID, primary key)
  ├── user_id        : UserId (from JWT)
  ├── session_id     : SessionId (from hook payload)
  ├── tool_name      : String
  ├── agent_id       : AgentName (newtype)
  ├── agent_scope    : String ("admin" | "user")
  ├── decision       : String ("allow" | "deny")
  ├── policy         : String ("default_allow" | "scope_restriction" | ...)
  ├── reason         : String
  ├── evaluated_rules: JSONB (Vec<RuleEvaluation>)
  └── created_at     : DateTime<Utc>
```

## Why Rust

- **Compile-time SQL**: Every governance query uses `sqlx::query_as!{}` — column names, types, and return struct verified at build time
- **Typed decisions**: `permissionDecision` is serialized from a Rust enum with exactly two variants: `Allow` and `Deny` — no free-form strings
- **Audit fidelity**: The deny reason, policy name, and full rule evaluation chain are stored as typed JSONB — queryable and verifiable
