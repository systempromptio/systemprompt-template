# User Report

Answer "who is using this platform?" - recent signups, who holds which role, who is suspended, and what a single
account looks like up close. This is the roster; for what those people actually *did*, use `admin_activity_report`.

## Ask me things like

- "Who signed up this week?"
- "List the admins."
- "Show me recent users as a table."
- "How many users do we have, and how many are active?"
- "Tell me everything about you@example.com."

## When to Use

Use this skill for anything keyed to a **person**: the roster, role membership, account status, signup rate,
active sessions per user. Reach for `admin_activity_report` instead when the question is about traffic, spend,
requests, or failures.

## How commands run

Every command below runs through the admin `systemprompt` MCP server, which exposes exactly one tool - also
named `systemprompt` - taking a single `command` argument. Pass the CLI command **without** the `systemprompt`
prefix:

```json
{ "command": "admin users list --limit 50 --json" }
```

The server is admin-only. A non-admin caller gets
`Insufficient permissions. User must have one of: ["admin"]` - that is the gate working, not a bug.

## Output format: pick one deliberately

- **Answering in chat** - omit `--json`. The CLI prints a formatted table; summarise it in prose plus a
  markdown table of the rows that matter. Do not paste raw CLI output wholesale.
- **Feeding a dashboard artifact** - always add `--json`. The CLI then emits a typed envelope:
  `{"artifact_type":"table","columns":[{"name","column_type"}],"items":[{...}]}` for lists, or
  `{"artifact_type":"presentation_card","sections":[{"heading","content"}]}` for stats. The **User Directory**
  artifact renders the table envelope directly - if it is installed, prefer refreshing it over re-pasting rows.

## The roster

```bash
systemprompt admin users list --limit 50                    # newest first
systemprompt admin users list --role admin                  # admin | user | anonymous
systemprompt admin users list --status suspended            # active | inactive | suspended | pending | deleted | temporary
systemprompt admin users list --limit 50 --offset 50        # page 2
systemprompt admin users count
systemprompt admin users stats                              # totals, created_24h, by-role and by-status splits
```

`list` returns `id`, `name`, `email`, `status`, `roles`, `created_at`. **Read `roles` before reporting a
headcount**: most workspaces carry a large tail of `anonymous` users minted per browser fingerprint
(`fp_*@anonymous.local`). Those are visitors, not accounts. When someone asks "how many users", give the
signed-in count (`--role user` plus `--role admin`) and mention the anonymous tail separately rather than
quoting the raw total.

## One account

```bash
systemprompt admin users show <user-id>
systemprompt admin users search "<name | email | full name>"
systemprompt admin users session list <user-id>             # sessions for one account
systemprompt admin users webauthn list --user <user-id>     # registered passkeys
```

Start from `search` when you only have a name or an email - it returns the id the other commands need.

## Roles and access

```bash
systemprompt admin users role list <user-id>
```

This skill is read-only. To **change** a role (promote, demote, assign), follow `manage_roles`; to
create, edit, delete, or merge accounts, follow `manage_users`; suspensions and IP bans are
`block_users`, and force-logout is `manage_sessions`. For the governance side of roles - watching a
request flip from denied to allowed - use `manage_permissions`.

## Export

```bash
systemprompt admin users export --json                      # full records, for offline analysis
```

## Typical workflow

1. `admin users stats` - headcount, split by role and status, signups in the last 24h.
2. `admin users list --limit 50` - the recent roster; separate real accounts from the anonymous tail.
3. `admin users list --role admin` - confirm who holds elevated access.
4. `admin users search` then `show` - drill into anyone who looks anomalous.
5. Hand off to `admin_activity_report` to see what those accounts are actually spending and running.
