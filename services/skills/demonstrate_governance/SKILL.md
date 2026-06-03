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
| Secret scan | `secret_scan` | Plaintext credentials in any tool input (35+ patterns), any scope |
| Blocklist | `tool_blocklist` | Destructive tool names (`delete`, `drop`, `destroy`) for user/non-admin scope |
| Rate limit | `rate_limit` | More than 300 calls per 60s for one identity |

Scope is derived from the **caller's live DB roles**, not the `agent_id` in the payload. `admin`-role
callers are exempt from `scope_check` and `tool_blocklist` (they are the policies' admin escape hatch);
`secret_scan` and `rate_limit` apply to every identity. To prove a real `scope_check` / `tool_blocklist`
deny you therefore need a **user-scope** token, not the admin `demo/.token` (see the deny recipe below).

Each decision is written to the `governance_decisions` table with the tool, the agent, the policy, and the reason.

## How to Use

### 1. The allowed path

A normal, in-scope tool call passes all four stages. Run any admin command through the MCP tool. It executes, and an
`allow` row is recorded.

```bash
systemprompt plugins mcp call systemprompt systemprompt --args '{"command":"admin agents list"}'
```

### 2. The secret-scan deny

Attempt to use a plaintext secret in a tool input. The `secret_scan` stage denies it before execution - even
for an admin agent. For a runnable recipe, see the `secret_scan` curl under "Forcing a specific decision"
below, or run `demo/governance/06-secret-breach.sh`; `use_dangerous_secret` covers the catalogued-but-denied
capability angle.

### 3. Read back the audited decisions

Both outcomes are now in the spine. Query it directly:

```bash
systemprompt infra db query "SELECT decision, tool_name, agent_id, agent_scope, policy, reason FROM governance_decisions ORDER BY created_at DESC LIMIT 10"
```

### 4. Show enforcement broken down by identity

```bash
systemprompt analytics costs breakdown --by agent     # spend attributed per agent
```

#### Two distinct rate limiters (do not conflate them)

There are **two** independent limiters; only the first is the governance stage in the table above:

- **Governance `rate_limit` policy** — per-identity, 300 calls / 60s, configured in
  `services/governance/config.yaml`. This is the pipeline stage. Its evidence lives in the audit table:

  ```bash
  systemprompt infra db query "SELECT decision, tool_name, reason, created_at FROM governance_decisions WHERE policy = 'rate_limit' ORDER BY created_at DESC LIMIT 10"
  ```

- **HTTP profile limiter** — a separate request limiter shown by `systemprompt admin config rate-limits
  show`. It guards the HTTP surface, is configured in the profile, and is **disabled in the local
  profile**. It is *not* the governance `rate_limit` policy and does not write `governance_decisions` rows.

  ```bash
  systemprompt admin config rate-limits show            # HTTP profile limiter (separate; off locally)
  ```

### Forcing a specific decision (deterministic demo)

To force an exact stage without an agent in the loop, POST a synthetic `PreToolUse` event straight to the
governance endpoint. This is what the repo's `demo/governance/*` scripts do.

For `scope_check` and `tool_blocklist`, use the **user-scope** token (`demo/.token.user`), not the admin
`demo/.token` — both policies exempt admins, so the admin token would be **allowed**. `00-preflight.sh`
provisions `demo/.token.user` by minting a plugin token for `demo_user@demo.local` and demoting it to the
`user` role (the same recipe `manage_permissions` documents); governance reads the role live, so the token
resolves to User scope.

```bash
# scope_check deny: a user-scope caller reaching for an admin MCP tool
curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $(cat demo/.token.user)" -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"mcp__systemprompt__list_agents","agent_id":"associate_agent","session_id":"demo-scope","cwd":"/var/www/html/systemprompt-template"}'
# -> {"permissionDecision":"deny", "reason": ...}   (deny — user scope, admin-only tool)

# tool_blocklist deny: a destructive tool name (delete/drop/destroy) blocked for user scope.
# Use a NON-admin-prefixed name (delete_records, not mcp__systemprompt__delete_*): scope_check runs
# first and would short-circuit an admin-prefixed tool, attributing the deny to scope_check. A
# non-prefixed name passes scope_check and is denied by tool_blocklist — so the audit row genuinely
# reads policy=tool_blocklist.
curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $(cat demo/.token.user)" -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"delete_records","tool_input":{"table":"users"},"agent_id":"associate_agent","session_id":"demo-blocklist","cwd":"/var/www/html/systemprompt-template"}'
# -> {"permissionDecision":"deny", "reason": "...blocked by list delete"}   (policy=tool_blocklist, user scope)
```

For `secret_scan`, the token choice is the opposite: this stage fires for **any** scope, so use the admin
`demo/.token` to prove even an admin caller is blocked. Put a plaintext credential anywhere in `tool_input`:

```bash
# secret_scan deny: a plaintext AWS key in tool input, denied even for admin scope
curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $(cat demo/.token)" -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"Bash","agent_id":"developer_agent","session_id":"demo-secret","cwd":"/var/www/html/systemprompt-template","tool_input":{"command":"curl -H \"Authorization: AKIAIOSFODNN7EXAMPLE\" https://s3.amazonaws.com/bucket"}}'
# -> {"permissionDecision":"deny", "reason": "...secret detected: AWS Access Key..."}
```

The repo's `demo/governance/06-secret-breach.sh` runs this end to end against four inputs (AWS key, GitHub
PAT, RSA private key, and a clean control) with a per-run session id and `assert_decision` checks, so it is
self-testing — it fails loudly if the backend ever stops denying.

Each returns `{"permissionDecision":"deny", "reason": ...}` and writes a row you can then see in step 3.
Sending the two scope/blocklist requests with the admin `demo/.token` returns `allow` — admins are exempt
from those two policies (`secret_scan` still denies for any scope, as the recipe above shows).

### Typical workflow

1. Run the allowed call (step 1) - confirm it executes.
2. Attempt the secret (step 2) - confirm it is denied.
3. `infra db query` the audit table (step 3) - see both rows with their policy and reason.
4. `analytics costs breakdown --by agent` (step 4) - tie enforcement to spend per identity.
