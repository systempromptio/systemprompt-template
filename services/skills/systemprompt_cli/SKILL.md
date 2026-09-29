# systemprompt CLI Reference

A navigable map of the `systemprompt` CLI. Use it to find the right command, then drill in with `--help`.

## When to Use

Use this skill whenever you need to operate the Enterprise Demo through its CLI: managing skills, services, agents, configuration, governance logs, analytics, cloud deploys, or MCP plugins. It tells you which of the 8 domains owns a task and how to discover the exact command.

## How to Use

Every command follows one shape:

```
systemprompt <domain> <subcommand> [args] [options]
```

The CLI is self-documenting. Most commands nest two or three levels deep, so walk the help tree rather than guessing:

```bash
systemprompt --help                       # top-level: the 8 domains
systemprompt analytics --help             # a domain's subcommands
systemprompt analytics costs --help       # a subcommand's own commands (summary, trends, breakdown)
```

### The 8 domains

| Domain | Purpose |
|--------|---------|
| `core` | Skills, content, files, contexts, plugins, hooks, artifacts |
| `infra` | Services, database, jobs, logs (view, stream, request, trace, audit) |
| `admin` | Users, agents, config, setup, session |
| `cloud` | Auth, deploy, sync, secrets, tenant, domain, profiles |
| `analytics` | Overview, conversations, agents, tools, requests, sessions, content, traffic, costs |
| `web` | Content-types, templates, assets, sitemap, validate |
| `plugins` | Extensions, MCP servers, capabilities |
| `build` | Build the core workspace and MCP extensions |

Note: most `analytics` subcommands need a further verb, e.g. `analytics costs summary`, `analytics requests stats`, `analytics agents list`.

### Running the CLI through the admin MCP server

The `systemprompt` MCP server (admin-only) has **typed tools** for the common questions. Prefer them to a hand-built CLI command: they carry their flags in the schema, page on their own, and never return more than a model can read.

| Question | Tool |
|----------|------|
| What did one person do: conversations, active days, skills, titles | `user_activity` |
| One row per conversation in a window | `conversation_list` |
| Who is spending, how much | `usage_by_user` |
| Debug one request (not for counting activity) | `request_log` |
| What was said in one request | `conversation_audit` (`max_chars` 300-500) |
| Who is registered, with which role | `users` |
| The dashboard | `admin_report` (`{"report":"costs","days":7}`) |

For people and usage questions use the skills `admin_person_activity` (one person) and `admin_ai_usage` (spend, adoption). Rows are named `<user_id> (<display name>)`; the id is what other tools take as `user`. Map ids to emails with `users`.

**Paging.** `limit` is clamped to 100. `request_log`: a non-empty `next_cursor` means older rows - pass it as `cursor`. `conversation_audit`: `has_more` means pass `next_offset` as `offset`; `message_count`/`tool_call_count` are totals. `users`: `next_cursor` is the next `offset`. `usage_by_user`: `truncated` means raise `limit` or narrow the window. To reach an earlier window set `since`/`until`; do not page back to it.

For anything else, the server's `systemprompt` tool executes a CLI command. Pass the command **without** the `systemprompt` prefix as a `command` argument:

```bash
systemprompt plugins mcp call systemprompt systemprompt --args '{"command":"core skills list"}'
```

Through the tool the output format is already set and your credential is forwarded: never pass `--json`, `--yaml`, `--format` or `--export` (they are stripped), and never override the profile or database. A result over 1 MB is stored whole as an artifact and you get a pointer to it - narrow the query rather than repeating it. If a command rejects a flag, run `<command> --help` **once**; do not guess flags.

Prefer this over raw bash when operating remotely or as an agent: the server handles authentication, profile routing, and session context automatically. See the `inspect_mcp_and_skills` skill for listing and calling MCP tools.

#### Rules for reporting

- Never conclude a day, total or trend from one page. Use an aggregate tool, or page to the end and say "at least N" if you stopped.
- When two sources disagree, report both figures and do not invent a cause.
- State the window, and name any source that was denied, truncated or empty.

### Flags that exist

| Command | Flags |
|---------|-------|
| `infra logs request list` | `--since`, `--until`, `--user <id>`, `--model`, `--provider`, `--limit/-n`, `--before <cursor>`; pages by cursor only, no status filter |
| `infra logs audit <id>` | `--messages/-m`, `--tools/-t`, `--offset`, `--limit/-n` (0 = all), `--max-content <chars>` |
| `analytics costs breakdown` | `--by model\|provider\|agent\|user`, `--since`, `--until`, `--limit/-n` |
| `analytics costs summary` / `trends`, `sessions stats` / `trends` | `--since`, `--until` (`trends`: `--group-by`) |
| `analytics requests list` | `--since`, `--until`, `--user <id>`, `--model`, `--limit/-n`, `--offset` |
| `analytics conversations list` | `--since`, `--until`, `--source agent\|gateway\|all`, `--user <id>`, `--limit/-n` |
| `admin users list` | `--limit`, `--offset`, `--role admin\|user\|anonymous`, `--status`, `--include-anonymous` (with `--role` paging is ignored) |
| `admin users role assign <user-id>` | `--roles <role>[,<role>]` - only `admin`, `user`, `anonymous` exist |
| `infra logs governance report` | `--since`, `--group-by policy\|tool\|user`, `--limit` |
| `infra logs trace list` | `--since`, `--agent`, `--status`, `--tool`, `--decision`, `--has-mcp`, `--all`, `--limit/-n` |

Times: `30m`, `24h`, `7d`, or `YYYY-MM-DD[THH:MM:SS]`; `--until` is exclusive. A cursor is `<created_at>@<request_id>`; pass the last row's `cursor` to `--before`.

### Common options

| Option | Description |
|--------|-------------|
| `--json` / `--yaml` | Structured output - use `--json` when parsing programmatically from a shell (the MCP tool sets it for you) |
| `--profile <name>` | Target a specific profile without switching the active session |
| `-n, --limit <N>` | Cap rows on list commands (logs, analytics) |
| `--since <dur>` | Time window, e.g. `1h`, `24h`, `7d` |

### Fast paths to the task skills

This skill is the index. For an actual task, jump to the skill that owns it:

| You want to... | Skill |
|----------------|-------|
| See a dangerous capability refused by policy | `use_dangerous_secret` |
| Exercise all four governance stages and read the audit | `demonstrate_governance` |
| Edit a user's roles and watch a request flip allow/deny | `manage_permissions` |
| Reconstruct the live conversation's structured data | `inspect_conversation` |
| Deep-dive one AI gateway request and its trace | `inspect_ai_requests` |
| Fleet rollups: cost, agents, tools, sessions | `analytics_dashboards` |
| Discover, validate, and message agents | `inspect_agents` |
| List/call MCP tools and sync skills | `inspect_mcp_and_skills` |
| Start/inspect services, database, and jobs | `manage_services` |
| AI spend and adoption: this week's cost, top users, skills and models | `admin_ai_usage` |
| What one person or team has been working on | `admin_person_activity` |
| Who am I on this instance, and what does my access grant | `who_am_i` |

### Examples

```bash
systemprompt core skills list
systemprompt infra services status
systemprompt analytics overview
systemprompt analytics costs summary
systemprompt admin session show
```
