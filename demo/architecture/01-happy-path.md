# Demo 01: Happy Path — Architecture

## What it does

Simulates a Claude Code PreToolUse hook workflow. Governance ALLOWS an admin-scope tool call, then the MCP tool executes and returns real data.

## Flow

```
  curl: POST /api/public/hooks/govern
    │
    ▼
  ┌─────────────────────────────────────────────────────────┐
  │  PreToolUse Hook Simulation                             │
  │  Payload: {                                             │
  │    agent_id: "developer_agent",                         │
  │    tool_name: "mcp__systemprompt__list_agents",         │
  │    tool_input: {},                                      │
  │    session_id: "demo-happy-path"                        │
  │  }                                                      │
  │  ── JSON boundary (only untyped moment) ──              │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Axum Handler: govern_tool_use()                        │
  │  1. extract_bearer_token() → &str                       │
  │  2. validate_jwt_token() → JwtClaims { sub: UserId }    │
  │  3. serde::from_value() → HookEventPayload (typed)      │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Scope Resolution                                       │
  │  resolve_agent_scope("developer_agent") → "admin"       │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Rule Engine: rules::evaluate()                         │
  │  scope_check: admin → PASS                              │
  │  secret_injection: clean input → PASS                   │
  │  rate_limit: within limits → PASS                       │
  │  decision = "allow"                                     │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Audit: INSERT governance_decisions (async)             │
  │  decision="allow", policy="default_allow"               │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Response: HTTP 200                                     │
  │  { permissionDecision: "allow" }                        │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  MCP Tool Execution (Part 2)                            │
  │  plugins mcp call systemprompt list_agents              │
  │  OAuth authentication → tool executes → JSON result     │
  └─────────────────────────────────────────────────────────┘
```

## Contrast with Demo 02

```
  Demo 01 (admin scope)          Demo 02 (user scope)
  ─────────────────────          ─────────────────────
  agent: developer_agent         agent: associate_agent
  scope: admin                   scope: user
  decision: ALLOW                decision: DENY
  policy: default_allow          policy: scope_restriction
  tool executes: YES             tool blocked: YES
```

## Why Rust

- **Typed boundary**: JSON → `HookEventPayload` (serde) → typed processing → `GovernanceResponse` → JSON. Untyped only at HTTP boundaries
- **Compile-time SQL**: `INSERT INTO governance_decisions` uses `sqlx::query!{}` — validated at build time
- **Async audit**: `tokio::spawn` writes the audit record without blocking the governance response
