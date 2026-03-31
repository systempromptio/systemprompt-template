# Demo 04: Governance Happy Path — Architecture

## What it does

Admin-scope agent calls an MCP tool. The governance hook fires, evaluates rules, and ALLOWS the call.

## Flow

```
  developer_agent calls MCP tool
    │
    ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Claude Code: PreToolUse Hook                           │
  │  Fires SYNCHRONOUSLY before tool execution              │
  │  Constructs HookEventPayload (typed Rust struct)        │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  HTTP: POST /api/public/hooks/govern                    │
  │  Headers: Authorization: Bearer <JWT>                   │
  │  Query: plugin_id=enterprise-demo                       │
  │  Body: { hook_event_name, tool_name, agent_id,          │
  │          session_id, tool_input }                        │
  │                                                         │
  │  ── JSON boundary (only untyped moment) ──              │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Axum Handler: govern_tool_use()                        │
  │  1. extract_bearer_token() → &str                       │
  │  2. validate_jwt_token() → JwtClaims { sub: UserId }    │
  │  3. serde::from_value() → HookEventPayload (typed)      │
  │  ── All data now typed Rust structs ──                  │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Scope Resolution                                       │
  │  resolve_agent_scope("developer_agent") → "admin"       │
  │  Builds GovernanceContext {                              │
  │    user_id: UserId,                                     │
  │    session_id: SessionId,                               │
  │    agent_id: AgentName,                                 │
  │    agent_scope: "admin",                                │
  │    tool_name: String,                                   │
  │    tool_input: Value                                    │
  │  }                                                      │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Rule Engine: rules::evaluate()                         │
  │                                                         │
  │  Rule 1: scope_check                                    │
  │    admin scope → all tools allowed → PASS               │
  │                                                         │
  │  Rule 2: secret_injection                               │
  │    scan tool_input for AWS keys, PATs, PEM → PASS       │
  │                                                         │
  │  Rule 3: rate_limit                                     │
  │    <300 calls/min this session → PASS                   │
  │                                                         │
  │  All rules PASS → decision = "allow"                    │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Audit: tokio::spawn(record_decision())                 │
  │  INSERT INTO governance_decisions                       │
  │  (id, user_id, session_id, decision="allow",            │
  │   policy="default_allow", evaluated_rules=JSONB)        │
  │  (compile-time checked SQL)                             │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Response: HTTP 200                                     │
  │  GovernanceResponse → serde::to_value() → JSON          │
  │  { hookSpecificOutput: {                                │
  │      hookEventName: "PreToolUse",                       │
  │      permissionDecision: "allow"                        │
  │  }}                                                     │
  │  ── Back to JSON at the HTTP boundary ──                │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  Claude Code receives "allow" → tool executes
```

## Why Rust

- **Typed boundary**: JSON → `HookEventPayload` (serde) → typed processing → `GovernanceResponse` (serde) → JSON. Untyped only at the two HTTP boundaries
- **Newtype enforcement**: `GovernanceContext` carries `UserId` and `SessionId` as distinct types — cannot be swapped
- **Async audit**: `tokio::spawn` fires the DB write without blocking the response — the governance decision returns immediately while the audit record writes asynchronously
- **Compile-time SQL**: The `INSERT INTO governance_decisions` uses `sqlx::query!{}` — if the schema changes, the code won't compile
