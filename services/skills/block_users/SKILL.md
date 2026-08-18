# Block Users

Cut off access: suspend an account, reinstate it, and manage the IP ban list. Two distinct levers:

- **Account suspension** - keyed to a *user*: `admin users update <id> --status suspended`.
- **IP bans** - keyed to an *address*, independent of any account: `admin users ban ...`.

## Ask me things like

- "Suspend that account until we've talked to them."
- "Reinstate jane@example.com."
- "Ban this IP, it's hammering the gateway."
- "Is 203.0.113.7 banned? Why?"
- "Show me every active ban."

## When to Use

Use this skill when access must be **removed or restored**. Routine account edits are
`manage_users`, role changes are `manage_roles`, and ending live sessions is `manage_sessions` -
suspension does not kill an existing session token, so pair the two when it matters.

## How commands run

Every command below runs through the admin `systemprompt` MCP server, which exposes exactly one
tool - also named `systemprompt` - taking a single `command` argument. Pass the CLI command
**without** the `systemprompt` prefix:

```json
{ "command": "admin users ban list" }
```

The server is admin-only. A non-admin caller gets
`Insufficient permissions. User must have one of: ["admin"]` - that is the gate working, not a bug.

## Guardrails (read first)

1. **Confirm identity before acting.** Never suspend or ban on a name match alone -
   `admin users search "<name or email>"` then `admin users show <user-id>`, and read it back to
   the operator.
2. **State the blast radius.** A suspension blocks new sign-ins but leaves live sessions running;
   offer `manage_sessions` (`session end --user <id> --all --yes`) to cut those too. An IP ban
   affects *everyone* behind that address - flag shared NAT/VPN egress risk before a permanent ban.
3. **Get explicit go-ahead** before any suspend, ban, or unban. Record the reason - `ban add`
   requires `--reason` for exactly this.

## Suspend and reinstate an account

```bash
systemprompt admin users update <user-id> --status suspended
systemprompt admin users update <user-id> --status active           # reinstate
systemprompt admin users list --status suspended                    # who is currently suspended
```

## IP bans

```bash
systemprompt admin users ban list                                   # active bans
systemprompt admin users ban add <ip> --reason "credential stuffing" --duration 24h
systemprompt admin users ban add <ip> --reason "abuse - repeat offender" --permanent
systemprompt admin users ban check <ip>                             # banned? since when? why?
systemprompt admin users ban remove <ip> --yes
systemprompt admin users ban cleanup --yes                          # purge expired bans
```

`--duration` takes a human duration (`24h`, `7d`); `--permanent` overrides it. `remove` and
`cleanup` require `--yes`.

## Typical workflow

1. `admin users search` → `show` - confirm exactly who or what is being blocked.
2. State the action and its blast radius; get the go-ahead.
3. Suspend (`update --status suspended`) and/or ban the IP with a written `--reason`.
4. `manage_sessions`: end the account's live sessions if immediate cut-off is required.
5. Verify: `admin users show <user-id>` shows `suspended`; `ban check <ip>` confirms the ban.
