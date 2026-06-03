# Manage Permissions

Change what a user is allowed to do, then watch the governance pipeline honour the change on the very next call - no
restart, no new token. This is the live proof that authority is data, not a baked-in constant.

## When to Use

Use this skill to demonstrate role-based access control end to end: take one user, grant a privileged tool call,
revoke the user's `admin` role with a single CLI command, show the same call now **denied**, then re-grant it and show
it **allowed** again. The whole flip happens against the running system with the same bearer token.

## Why it works (the key fact)

The governance `scope_check` policy decides admin-only tools (`mcp__systemprompt__*`) by reading the caller's roles
**from the database on every request** (`users.roles`), not from a cached token claim. So:

- Editing roles with `admin users role …` takes effect on the **next** governance decision.
- **No reload, no service restart, no re-issued token** is required - the same bearer token flips outcome because
  scope is derived live from the DB. (A minted plugin token carries `scope=hook:govern hook:track`, so the DB role,
  not the token, decides admin access.)

`admin` role -> Admin scope (all tools). `user` / anything else -> User scope (denied admin-only tools).

## Safety: never target the configured system admin

The profile designates one system admin (here `ed`). Demoting that user trips a startup guard
("system admin exists but does not carry the admin role") that blocks CLI access. **Always run this demo against a
dedicated throwaway user**, never the system admin. The sequence below creates one and deletes it at the end.

## How to Use

### 1. Create a dedicated demo user and mint its token

`issue-plugin-token` refuses non-admin users, so promote first, mint the token while the user is admin, then flip the
role. The token stays valid; only the DB role changes.

```bash
systemprompt admin users create --name perms_demo --email perms_demo@demo.local --if-not-exists
# grab the id from `admin users show`, then:
systemprompt admin users role promote <user_id>                 # make admin so a token can be minted
systemprompt admin keys issue-plugin-token --email perms_demo@demo.local   # copy the JWT from the output
```

Hold that token as `$TK`. It is bound to this demo user (`sub`), with hook/plugin scope - so the user's DB role
governs every decision.

### 2. Revoke admin, then show the request DENIED

```bash
systemprompt admin users role demote <user_id>

curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TK" -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"mcp__systemprompt__users_show","agent_id":"associate_agent","session_id":"perms-deny","tool_input":{}}'
# -> {"permissionDecision":"deny", reason: "tool mcp__systemprompt__users_show requires admin"}
```

### 3. Grant admin, then show the SAME request ALLOWED

```bash
systemprompt admin users role promote <user_id>

curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TK" -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"mcp__systemprompt__users_show","agent_id":"associate_agent","session_id":"perms-allow","tool_input":{}}'
# -> {"permissionDecision":"allow"}   (same token, no reload - the DB role changed)
```

The only thing that changed between the deny and the allow is one CLI command. Same token, same tool, same endpoint.

### 4. Confirm in the audit trail

```bash
systemprompt infra db query "SELECT decision, tool_name, policy, reason, created_at FROM governance_decisions WHERE session_id IN ('perms-deny','perms-allow') ORDER BY created_at"
```

Two rows: one `deny` (scope_check), one `allow`, seconds apart, for the same user.

### 5. Clean up

```bash
systemprompt admin users delete <user_id>     # remove the throwaway demo user
```

### Arbitrary role sets

`promote`/`demote` are shortcuts for the `admin` role. To set any role list use:

```bash
systemprompt admin users role assign <user_id> --roles user,admin
```

### Typical workflow

1. `admin users create` + `role promote` + `keys issue-plugin-token` - a demo user and its token.
2. `role demote` -> govern POST -> **deny** (rejected request).
3. `role promote` -> govern POST -> **allow** (accepted request).
4. `infra db query` the two `governance_decisions` rows - the flip, audited.
5. `admin users delete` - clean up.

The takeaway: permissions are live state. One command re-grants or revokes authority, and the next governed action
reflects it immediately - which is exactly what an enterprise needs to prove to an auditor.
