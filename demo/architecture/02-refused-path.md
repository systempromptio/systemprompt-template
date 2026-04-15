# Demo 02: Refused Path — Architecture

## What it does

User-scope agent tries to call an admin-only MCP tool. Governance DENIES it at the rule level.

## Flow

```
  curl: POST /api/public/hooks/govern
    │
    ▼
  ┌─────────────────────────────────────────────────────────┐
  │  PreToolUse Hook Simulation                             │
  │  Payload: {                                             │
  │    agent_id: "associate_agent",                         │
  │    tool_name: "mcp__systemprompt__list_agents",         │
  │    session_id: "demo-refused-path"                      │
  │  }                                                      │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Scope Resolution                                       │
  │  resolve_agent_scope("associate_agent") → "user"        │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Rule Engine: rules::evaluate()                         │
  │                                                         │
  │  Rule 1: scope_check                                    │
  │    tool starts with "mcp__systemprompt__"               │
  │    requires admin scope                                 │
  │    agent scope = "user"                                 │
  │    → FAIL                                               │
  │                                                         │
  │  decision = "deny", policy = "scope_restriction"        │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Audit: INSERT governance_decisions                     │
  │  decision="deny", policy="scope_restriction"            │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Response: HTTP 200                                     │
  │  { permissionDecision: "deny",                          │
  │    permissionDecisionReason: "[GOVERNANCE] ..." }       │
  └─────────────────────────────────────────────────────────┘
```

## Defense-in-Depth

Two independent layers prevent unauthorized tool access:

```
  Layer 1: MAPPING (preventive)
  ─────────────────────────────
  In Claude Code, user-scope agents have no admin
  MCP servers in their tool list. The tool doesn't
  appear — no call is possible.

  Layer 2: GOVERNANCE RULES (enforcement)
  ───────────────────────────────────────
  Even if mapping were misconfigured, the scope_check
  rule evaluates every PreToolUse hook call. User scope
  calling admin tools → DENY.

  Neither layer depends on the other.
```

## Why Rust

- **Type-safe scope resolution**: Agent scope is resolved from a static `HashMap` initialized at startup — not a runtime string comparison
- **Defense in depth**: Even if the mapping were bypassed, the governance layer catches it at the rule level
- **Audit completeness**: The denial is recorded with typed IDs — policy, reason, and evaluated rules stored as JSONB
