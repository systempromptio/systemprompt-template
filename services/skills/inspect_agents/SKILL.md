# Inspect Agents

Work with the agents in the workspace: list them, check their configuration and the MCP tools they can reach, send one
a message over the A2A protocol, and read the artifacts and traces it produces.

## When to Use

Use this skill to understand the agent fleet and exercise it: confirm which agents exist and their scope, validate a
config before relying on it, see which tools an agent is allowed to call, or drive an agent end to end and inspect its
output. The demo ships two agents - `developer_agent` (admin scope, all skills) and `associate_agent` (user scope,
restricted) - which together demonstrate scoped access.

## How to Use

### Discover and inspect

```bash
systemprompt admin agents list                  # configured agents: name, port, enabled, primary, default
systemprompt admin agents status                # process status: running, PID
systemprompt admin agents registry              # agents live in the A2A gateway registry
systemprompt admin agents show developer_agent  # full config: scope, model, skills, mcp servers
systemprompt admin agents show associate_agent
```

### Validate and list tools

```bash
systemprompt admin agents validate developer_agent    # check config for errors
systemprompt admin agents tools developer_agent       # MCP tools this agent may call
systemprompt admin agents tools associate_agent       # note the narrower set (user scope)
```

### Message an agent over A2A

Messaging spends real provider tokens. Use `--blocking` to wait for completion and reuse a `--context-id` for a
multi-turn conversation:

```bash
systemprompt core contexts create --name "agent-demo"
systemprompt admin agents message developer_agent \
  -m "List all agents and summarise their scopes." \
  --context-id <context-id> --blocking --timeout 60
```

### Read what it produced

```bash
systemprompt core artifacts list --context-id <context-id>
systemprompt core artifacts show <artifact-id> --full
systemprompt admin agents task <task-id>              # task details and the agent's response
```

### Follow the execution

```bash
systemprompt infra logs trace list --agent developer_agent --limit 5
systemprompt infra logs trace show <trace-id> --all
systemprompt admin agents logs developer_agent
```

### Typical workflow

1. `admin agents list` / `admin agents show <name>` - see the fleet and one agent's scope.
2. `admin agents tools <name>` - confirm what it is allowed to call.
3. `admin agents message <name> -m "…" --context-id <id> --blocking` - drive it.
4. `core artifacts show <id> --full` and `infra logs trace show <trace-id> --all` - inspect the result and the trace.
