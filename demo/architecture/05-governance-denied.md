# Demo 05: Governance Denied Path — Architecture

## What it does

Two denial scenarios via direct governance API calls. Part 1: scope restriction. Part 2: blocklist + scope restriction.

## Flow (Part 1: Scope Restriction)

```
  curl: POST /api/public/hooks/govern
  agent_id=associate_agent, tool=mcp__systemprompt__list_agents
    │
    ▼
  ┌─────────────────────────────────────────────────────────┐
  │  PreToolUse Hook → POST /api/public/hooks/govern        │
  │  tool_name: "mcp__systemprompt__list_agents"             │
  │  agent_id: "associate_agent"                             │
  │  ── JSON deserialized into HookEventPayload ──          │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  JWT Validation → UserId extracted                      │
  │  Scope Resolution                                       │
  │  resolve_agent_scope("associate_agent") → "user"        │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Rule Engine                                            │
  │                                                         │
  │  Rule 1: scope_check                                    │
  │    tool_name starts with "mcp__systemprompt__"          │
  │    requires admin scope                                 │
  │    agent scope = "user"                                 │
  │    → FAIL                                               │
  │                                                         │
  │  (remaining rules skipped — first failure is decisive)  │
  │                                                         │
  │  decision = "deny"                                      │
  │  policy = "scope_restriction"                           │
  │  reason = "user scope cannot access admin-only tools"   │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Audit: INSERT governance_decisions                     │
  │  decision="deny", policy="scope_restriction"            │
  │  evaluated_rules: [{ rule: "scope_check",               │
  │    result: "fail", detail: "..." }]                     │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Response: HTTP 200                                     │
  │  { hookSpecificOutput: {                                │
  │      permissionDecision: "deny",                        │
  │      permissionDecisionReason: "[GOVERNANCE] ..."       │
  │  }}                                                     │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  Hook receives "deny" → Claude Code BLOCKS the tool call
  In Claude Code: [GOVERNANCE] <reason> — tool never executes
```

## Flow (Part 2: Blocklist + Scope)

```
  curl: POST /api/public/hooks/govern
  agent_id=associate_agent, tool=mcp__systemprompt__delete_agent
    │
    ▼
  scope_check: FAIL (user scope + mcp__systemprompt__*)
  blocklist: FAIL (delete_* is a destructive operation)
    │
    ▼
  decision = "deny" — two rules triggered
```

## Contrast: Demo 04 vs Demo 05

```
  Demo 04 (ALLOW)                Demo 05 (DENY)
  ───────────────                ───────────────
  agent: developer_agent         agent: associate_agent
  scope: admin                   scope: user
  scope_check: PASS              scope_check: FAIL
  decision: allow                decision: deny
  policy: default_allow          policy: scope_restriction
  tool executes: YES             tool executes: NO
```

## Why Rust

- **Typed decisions**: The `permissionDecision` field is not a free-form string — it's serialized from a Rust enum with exactly two variants: `Allow` and `Deny`
- **Evaluated rules as typed structs**: `Vec<RuleEvaluation { rule: String, result: RuleResult, detail: String }>` — each rule evaluation is a typed struct, not a loose JSON object
- **Audit fidelity**: The deny reason, policy name, and full rule evaluation chain are all stored as typed JSONB — queryable and verifiable after the fact
