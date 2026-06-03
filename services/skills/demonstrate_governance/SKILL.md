# Demonstrate Governance

Drive every stage of the governance pipeline end to end, then prove that each decision was audited. This is the
guided tour of the enforcement spine: the same four stages run on every tool call in the workspace.

## When to Use

Use this skill to show, in one sitting, that the four governance policies actually fire and that every allow and deny
lands an auditable row. The `use_dangerous_secret` skill shows a single deny case; this skill exercises all four
stages and then reconstructs the audit trail behind them.

## The pipeline

Every tool call runs a synchronous four-stage check before it executes (config in `services/governance/config.yaml`):

| Stage | Policy id | What it blocks |
|-------|-----------|----------------|
| Scope check | `scope_check` | Non-admin scope calling `mcp__systemprompt__*` tools |
| Secret scan | `secret_scan` | Plaintext credentials in any tool input (35+ patterns) |
| Blocklist | `tool_blocklist` | Destructive tool names matching `delete`, `drop`, `destroy` |
| Rate limit | `rate_limit` | More than 300 calls per 60s for one identity |

Each decision is written to the `governance_decisions` table with the tool, the agent, the policy, and the reason.

## How to Use

### 1. The allowed path

A normal, in-scope tool call passes all four stages. Run any admin command through the MCP tool. It executes, and an
`allow` row is recorded.

```bash
systemprompt plugins mcp call systemprompt systemprompt --args '{"command":"admin agents list"}'
```

### 2. The secret-scan deny

Attempt to use a plaintext secret in a tool input (see `use_dangerous_secret` for the full walkthrough). The
`secret_scan` stage denies it before execution - even for an admin agent.

### 3. Read back the audited decisions

Both outcomes are now in the spine. Query it directly:

```bash
systemprompt infra db query "SELECT decision, tool_name, agent_id, agent_scope, policy, reason FROM governance_decisions ORDER BY created_at DESC LIMIT 10"
```

### 4. Show enforcement broken down by identity

```bash
systemprompt analytics costs breakdown --by agent     # spend attributed per agent
systemprompt admin config rate-limits show            # live rate-limit window
```

### Forcing a specific decision (deterministic demo)

To force an exact stage without an agent in the loop, POST a synthetic `PreToolUse` event straight to the governance
endpoint. This is what the repo's `demo/governance/*` scripts do; they need the demo token from
`demo/00-preflight.sh`.

```bash
# scope_check deny: user-scope agent reaching for an admin MCP tool
curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $(cat demo/.token)" -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"mcp__systemprompt__list_agents","agent_id":"associate_agent","session_id":"demo-scope","cwd":"/var/www/html/systemprompt-template"}'

# blocklist deny: a destructive tool name (delete_*) is blocked regardless of scope
curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $(cat demo/.token)" -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"mcp__systemprompt__delete_agent","tool_input":{"agent_id":"test"},"agent_id":"associate_agent","session_id":"demo-blocklist","cwd":"/var/www/html/systemprompt-template"}'
```

Each returns `{"permissionDecision":"deny", "reason": ...}` and writes a row you can then see in step 3.

### Typical workflow

1. Run the allowed call (step 1) - confirm it executes.
2. Attempt the secret (step 2) - confirm it is denied.
3. `infra db query` the audit table (step 3) - see both rows with their policy and reason.
4. `analytics costs breakdown --by agent` (step 4) - tie enforcement to spend per identity.
