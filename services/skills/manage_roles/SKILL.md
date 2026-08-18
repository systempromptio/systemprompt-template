# Manage Roles

Grant and revoke access levels: promote to admin, demote back to user, or set an explicit role
list. Read-only role questions ("who are the admins?") belong to `admin_user_report`.

## Ask me things like

- "Make jane@example.com an admin."
- "Revoke admin from that account."
- "What roles does this user hold?"
- "Set this service account's roles to user only."

## When to Use

Use this skill when a user's **role set must change**. For watching the governance side of a role
change - a request flipping from denied to allowed - use `manage_permissions`. For suspending
access entirely use `block_users`.

## How commands run

Every command below runs through the admin `systemprompt` MCP server, which exposes exactly one
tool - also named `systemprompt` - taking a single `command` argument. Pass the CLI command
**without** the `systemprompt` prefix:

```json
{ "command": "admin users role promote jane@example.com" }
```

The server is admin-only. A non-admin caller gets
`Insufficient permissions. User must have one of: ["admin"]` - that is the gate working, not a bug.

## The token caveat (tell the user every time)

A role change takes effect on the **next token issue**, not immediately: scopes are minted when the
session token is issued. After promoting someone, tell them to sign out of the Bridge and back in,
or their admin surfaces will keep returning permission errors. The same applies after a demotion -
the old token keeps its admin scopes until it expires or the session ends; if revocation must be
immediate, end their sessions via `manage_sessions`.

## Commands

```bash
systemprompt admin users role list <user-id>
systemprompt admin users role promote <user-id-or-email>       # grant the admin role
systemprompt admin users role demote <user-id-or-email>        # remove the admin role
systemprompt admin users role assign <user-id> --roles user,admin   # set an explicit list
```

`promote`/`demote` touch only the `admin` role and leave everything else in place. `assign`
replaces the role set with exactly what you pass - list current roles first so nothing is dropped
by accident.

## Guardrails

1. Confirm the target with `admin users search` → `show` before changing anything - promoting the
   wrong account is a security incident, not a typo.
2. State the change plainly ("grant admin to Jane Doe, jane@example.com") and get the operator's
   go-ahead first.
3. Admin grants are gated: the `systemprompt` MCP server and the admin skills are admin-only, so a
   promotion widens what that person can see and do across the whole instance. Demote promptly when
   access is no longer needed.

## Typical workflow

1. `admin users search "<name or email>"` → note the id.
2. `admin users role list <user-id>` - current state.
3. Confirm with the operator, then `role promote` / `role demote` / `role assign`.
4. `role list` again to verify, and tell the affected user to sign out and back in.
