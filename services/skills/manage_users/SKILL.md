# Manage Users

Change accounts: create them, edit their details, delete them, merge duplicates, and run bulk
clean-ups. This is the mutation side of the roster; for read-only questions ("who is on the
platform?") use `admin_user_report`.

## Ask me things like

- "Create an account for jane@example.com."
- "Fix the display name on that account."
- "Delete the test user I made yesterday."
- "These two accounts are the same person — merge them."
- "Clean up the anonymous users older than 90 days."

## When to Use

Use this skill whenever an account must **change**. Roles are `manage_roles`, suspensions and IP
bans are `block_users`, sessions are `manage_sessions`, and pure reporting is `admin_user_report`.

## How commands run

Every command below runs through the admin `systemprompt` MCP server, which exposes exactly one
tool - also named `systemprompt` - taking a single `command` argument. Pass the CLI command
**without** the `systemprompt` prefix:

```json
{ "command": "admin users create --name jane --email jane@example.com" }
```

The server is admin-only. A non-admin caller gets
`Insufficient permissions. User must have one of: ["admin"]` - that is the gate working, not a bug.

## Guardrails (read first)

Delete, merge, and bulk operations are destructive and the CLI enforces `--yes` on them. Never
supply `--yes` reflexively:

1. Identify the exact target first - `admin users search "<name or email>"` then
   `admin users show <user-id>`.
2. State in plain English what is about to change and to whom.
3. Get the operator's explicit go-ahead.
4. Only then re-run the command with `--yes`.

For bulk operations, always run with `--dry-run` first and report the would-affect count before
asking for the go-ahead.

## Create and update

```bash
systemprompt admin users create --name jane --email jane@example.com [--full-name "Jane Doe"] [--display-name Jane] [--if-not-exists]
systemprompt admin users update <user-id> --email new@example.com
systemprompt admin users update <user-id> --full-name "Jane Q. Doe" --display-name Jane
systemprompt admin users update <user-id> --status active          # active | inactive | suspended | pending | deleted | temporary
systemprompt admin users update <user-id> --email-verified true
```

`--if-not-exists` makes `create` idempotent - useful in scripted seeding. Status changes for
disciplinary reasons (suspension) belong to `block_users`; `update --status` here is for routine
lifecycle fixes.

## Delete and merge

```bash
systemprompt admin users delete <user-id> --yes                    # permanent
systemprompt admin users merge --source <dup-id> --target <keep-id> --yes
```

Merge moves the source account's sessions and tasks onto the target, then **deletes the source**.
Show both accounts before proposing a merge and say explicitly which one survives.

## Bulk operations

```bash
systemprompt admin users bulk delete --role anonymous --older-than 90 --dry-run
systemprompt admin users bulk delete --role anonymous --older-than 90 --limit 500 --yes
systemprompt admin users bulk update --set-status inactive --status pending --older-than 30 --dry-run
systemprompt admin users bulk update --set-status inactive --status pending --older-than 30 --yes
```

At least one filter (`--role`, `--status`, `--older-than`) is required - the CLI refuses an
unfiltered bulk run. Typical use: pruning the `anonymous` visitor tail
(`fp_*@anonymous.local` fingerprint accounts) that every workspace accumulates.

## Export and stats

```bash
systemprompt admin users export --json [--role user] [--status active] [--output users.json]
systemprompt admin users stats
```

## Typical workflow

1. `admin users search` → `show` - pin down the exact account.
2. Dry-run if bulk; state the change in plain English.
3. Execute with `--yes` only after the operator confirms.
4. `admin users show <user-id>` (or `stats` for bulk) - verify the result before reporting done.
