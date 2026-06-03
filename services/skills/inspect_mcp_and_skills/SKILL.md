# Inspect MCP and Skills

Discover and exercise the MCP plugin surface - especially the admin-only `systemprompt` MCP server that runs CLI commands as a tool - and keep the skills catalogue in sync.

## When to Use

Use this skill to check which MCP servers are healthy, list the tools a server exposes, call a tool directly, read MCP server logs, or sync skills between disk and the database.

## How to Use

### MCP servers and tools

```bash
systemprompt plugins mcp list                           # configured MCP servers
systemprompt plugins mcp status                         # runtime status: running, PID, port
systemprompt plugins mcp validate                       # validate server configurations
systemprompt plugins mcp tools                          # all tools across servers
systemprompt plugins mcp tools --server systemprompt    # tools from the admin server only
systemprompt plugins mcp logs systemprompt              # server logs for debugging
```

> **Run tool enumeration via the direct CLI, not the passthrough.** `plugins mcp tools` and
> `admin agents tools` enumerate the live MCP server, which needs the admin MCP-server session. Run
> through the *direct* CLI (an authenticated admin), they return the tool list. Run *through* the
> `systemprompt` MCP passthrough (i.e. `plugins mcp call systemprompt systemprompt --args
> '{"command":"plugins mcp tools"}'`), the nested CLI has no admin MCP session and the enumeration comes
> back empty with `auth_required` — the nested process cannot authenticate to the live server. This is a
> known limitation of the passthrough (a loopback bridge/admin token for the nested CLI is tracked as core
> tech debt); enumerate tools with the direct CLI.

### Calling the systemprompt tool

The `systemprompt` server exposes one tool, also named `systemprompt`, that executes a CLI command. Pass the command **without** the `systemprompt` prefix:

```bash
systemprompt plugins mcp call systemprompt systemprompt --args '{"command":"core skills list"}'
systemprompt plugins mcp call systemprompt systemprompt --args '{"command":"infra services status"}'
```

The tool is admin-only: it is namespaced `mcp__systemprompt__*` and the governance `scope_check` policy denies it unless the caller has `admin` scope. Authentication is handled automatically by the bridge loopback proxy once an admin is signed in - no manual token step. If `plugins mcp status` shows `systemprompt` running and a `call` returns CLI output (rather than a JWT/auth error), the admin is authenticated end to end.

### Skills catalogue

```bash
systemprompt core skills list                           # configured skills
systemprompt core skills show <skill_id>                # config + instruction body for one skill
```

Skills are defined on disk under `services/skills/<id>/` (a `config.yaml` plus a body file) and surfaced to agents via a marketplace's `skills.include` list. The YAML is bootstrap state: the stack ingests it at startup and the database owns it at runtime. After editing skill YAML, restart the services so the change is reloaded:

```bash
systemprompt infra services restart api          # reload the API server's config
# full reload of API + agents + MCP servers:
systemprompt infra services stop && systemprompt infra services start
```

### Typical workflow

1. `plugins mcp status` - confirm `systemprompt` is running.
2. `plugins mcp tools --server systemprompt` - see the available tool.
3. `plugins mcp call systemprompt systemprompt --args '{"command":"core skills list"}'` - run a command through it.
4. `core skills list` - confirm the skills you expect are present and enabled.
