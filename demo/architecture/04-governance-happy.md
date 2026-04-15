# Demo 04: Governance Happy Path — Architecture

## What it does

Direct call to the governance API simulating a Claude Code PreToolUse hook. Admin-scope agent with clean tool input — all 3 rules pass.

## Flow

```
  curl: POST /api/public/hooks/govern
  agent_id=developer_agent, tool_name=Read, tool_input={file_path}
    │
    ▼
  ┌─────────────────────────────────────────────────────────┐
  │  PreToolUse Hook Simulation                             │
  │  Payload deserialized into HookEventPayload (typed)     │
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
  Hook receives "allow" → Claude Code proceeds with tool execution
```

## Key difference from Demo 01

Demo 01 uses `mcp__systemprompt__list_agents` (admin MCP tool). Demo 04 uses `Read` with a file path — a general tool that any scope can use. This exercises all 3 rules cleanly rather than relying on scope alone.

## Why Rust

- **Typed boundary**: JSON → `HookEventPayload` (serde) → typed processing → `GovernanceResponse` (serde) → JSON. Untyped only at the two HTTP boundaries
- **Newtype enforcement**: `GovernanceContext` carries `UserId` and `SessionId` as distinct types — cannot be swapped
- **Async audit**: `tokio::spawn` fires the DB write without blocking the response
- **Compile-time SQL**: The `INSERT INTO governance_decisions` uses `sqlx::query!{}` — if the schema changes, the code won't compile
