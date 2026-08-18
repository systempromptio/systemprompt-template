# Manage Sessions

See and control who is signed in: list a user's sessions, force one (or all) to end, and prune the
stale anonymous tail. The enforcement companion to `block_users` and `manage_roles` - a suspension
or demotion only bites a live session once that session ends.

## Ask me things like

- "What sessions does jane@example.com have open?"
- "Log that account out everywhere."
- "End this specific session."
- "Clean up anonymous users older than 30 days."

## When to Use

Use this skill when the question or action is about **live sign-ins**. Cross-user session activity
(what those sessions are doing) is `admin_activity_report`; blocking future sign-ins is
`block_users`.

## How commands run

Every command below runs through the admin `systemprompt` MCP server, which exposes exactly one
tool - also named `systemprompt` - taking a single `command` argument. Pass the CLI command
**without** the `systemprompt` prefix:

```json
{ "command": "admin users session list <user-id> --active" }
```

The server is admin-only. A non-admin caller gets
`Insufficient permissions. User must have one of: ["admin"]` - that is the gate working, not a bug.

## Guardrails (read first)

Ending a session logs a real person out **mid-work** - unsaved state in their client can be lost.
Confirm the target account (`admin users show <user-id>`), say which sessions will be ended, and
get the operator's go-ahead before running `end`, which requires `--yes`. The exception is incident
response paired with `block_users`: suspend first, then end sessions immediately.

## Commands

```bash
systemprompt admin users session list <user-id>                    # recent sessions for one account
systemprompt admin users session list <user-id> --active           # only live ones
systemprompt admin users session list <user-id> --limit 50

systemprompt admin users session end --session <session-id> --yes  # end one session
systemprompt admin users session end --user <user-id> --all --yes  # log the account out everywhere

systemprompt admin users session cleanup --days 30 --yes           # prune anonymous users older than N days
```

`cleanup` deletes stale **anonymous** accounts (the `fp_*@anonymous.local` visitor tail) older than
`--days` (default 30) - it does not touch signed-in users' sessions.

## Typical workflow

1. `admin users search "<name or email>"` - get the user id.
2. `admin users session list <user-id> --active` - what is actually live.
3. Confirm with the operator, then `session end` (one id, or `--user … --all`).
4. `session list <user-id> --active` again - verify nothing is left.
