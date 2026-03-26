---
name: "Agent Management"
description: "List, message, configure, and restart agents via the systemprompt CLI"
---

# Agent Management

You manage AI agents using the systemprompt CLI. All operations go through the `admin agents` domain.

## Agent CRUD

| Command | Purpose |
|---------|---------|
| `systemprompt admin agents list` | List all agents |
| `systemprompt admin agents list --enabled` | List enabled agents only |
| `systemprompt admin agents show <name>` | View agent config and details |
| `systemprompt admin agents create --name <name> --port <port>` | Create a new agent |
| `systemprompt admin agents create --name <name> --port <port> --display-name "Name" --description "desc" --system-prompt-file prompt.md` | Create with full config |
| `systemprompt admin agents edit <name>` | Edit an agent's configuration |
| `systemprompt admin agents edit <name> --set key=value` | Edit specific field |
| `systemprompt admin agents edit <name> --enable` | Enable an agent |
| `systemprompt admin agents edit <name> --disable` | Disable an agent |
| `systemprompt admin agents delete <name> -y` | Delete an agent |

## Agent Communication (A2A Protocol)

| Command | Purpose |
|---------|---------|
| `systemprompt admin agents message <name> -m "task"` | Send a message (async) |
| `systemprompt admin agents message <name> -m "task" --blocking` | Send and wait for response |
| `systemprompt admin agents message <name> -m "task" --blocking --timeout 60` | Send with timeout (seconds) |
| `systemprompt admin agents message <name> -m "task" --stream` | Stream response |
| `systemprompt admin agents message <name> -m "task" --context-id <id>` | Send within a context |
| `systemprompt admin agents task <name> --task-id <id>` | Get task details/status |

## Agent Status & Monitoring

| Command | Purpose |
|---------|---------|
| `systemprompt admin agents status` | Check which agents are running |
| `systemprompt admin agents status <name>` | Check specific agent |
| `systemprompt admin agents registry` | Get running agents from gateway |
| `systemprompt admin agents registry --running` | Only running agents |
| `systemprompt admin agents logs <name>` | View agent logs |
| `systemprompt admin agents logs <name> -f` | Follow/stream agent logs |
| `systemprompt admin agents logs <name> -n 100` | Last 100 log lines |
| `systemprompt admin agents validate <name>` | Validate agent configuration |
| `systemprompt admin agents tools <name>` | List MCP tools available to agent |
| `systemprompt admin agents tools <name> --detailed` | Detailed tool info |

## Skills & MCP Configuration

Agents reference skills and MCP servers in their config:

```yaml
metadata:
  mcpServers:
    - systemprompt
  skills:
    - skill_name
```

Use `edit` with `--skill` / `--remove-skill` and `--mcp-server` / `--remove-mcp-server` to modify.

## Standard Workflow

1. **List agents** to see all available agents and their status
2. **Show agent** to inspect configuration, skills, and MCP servers
3. **Validate** before making changes to catch config errors early
4. **Operate** -- create, edit, or send messages
5. **Verify** -- check status or show the agent again

## Common Tasks

### Send a Task to an Agent

```bash
systemprompt admin agents list
systemprompt admin agents message <name> -m "Your task description" --blocking --timeout 120
```

### Create a New Agent

```bash
systemprompt admin agents create --name my_agent --port 9025 --display-name "My Agent" --description "Agent description" --system-prompt-file prompt.md
systemprompt admin agents validate my_agent
systemprompt admin agents show my_agent
```

### Debug an Agent

```bash
systemprompt admin agents status <name>
systemprompt admin agents logs <name> -n 100
systemprompt admin agents validate <name>
systemprompt admin agents tools <name>
```

### Check All Agent Tools

```bash
systemprompt admin agents registry --running
systemprompt admin agents tools <name> --detailed
```

## Important Notes

- Agent names are lowercase with underscores (e.g., `content_writer`)
- Config files are in `services/agents/` as YAML files
- Use `--blocking` when you need the agent's response before continuing
- Always validate config after edits and before restarts
- Use `--help` on any subcommand for full flag reference
