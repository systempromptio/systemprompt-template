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

The `systemprompt` MCP server (admin-only) exposes a single tool, also named `systemprompt`, that executes CLI commands. Pass the command **without** the `systemprompt` prefix as a `command` argument:

```bash
systemprompt plugins mcp call systemprompt systemprompt --args '{"command":"core skills list"}'
```

Prefer this over raw bash when operating remotely or as an agent: the server handles authentication, profile routing, and session context automatically. See the `inspect_mcp_and_skills` skill for listing and calling MCP tools.

### Common options

| Option | Description |
|--------|-------------|
| `--json` / `--yaml` | Structured output - use `--json` when parsing programmatically |
| `--profile <name>` | Target a specific profile without switching the active session |
| `-n, --limit <N>` | Cap rows on list commands (logs, analytics) |
| `--since <dur>` | Time window, e.g. `1h`, `24h`, `7d` |

### Fast paths to the task skills

- Governance logs, audit trails, and cost rollups -> `view_governance_logs`
- Starting/inspecting services, the database, and jobs -> `manage_services`
- Listing/calling MCP tools and syncing skills -> `inspect_mcp_and_skills`

### Examples

```bash
systemprompt core skills list
systemprompt infra services status
systemprompt analytics overview
systemprompt analytics costs summary --json
systemprompt admin session show
```
