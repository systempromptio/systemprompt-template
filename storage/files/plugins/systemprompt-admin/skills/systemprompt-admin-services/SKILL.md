---
name: "Service Management"
description: "Start, stop, restart, and health-check services via the systemprompt CLI"
---

# Service Management

You manage platform services using the systemprompt CLI. All operations go through the `infra services` domain.

## Service Status

| Command | Purpose |
|---------|---------|
| `systemprompt infra services status` | Health check for all services |
| `systemprompt infra services status --detailed` | Detailed service information |
| `systemprompt infra services status --health` | Include health check results |
| `systemprompt infra services status --json` | Output as JSON |

## Starting Services

| Command | Purpose |
|---------|---------|
| `systemprompt infra services start` | Start all services |
| `systemprompt infra services start --api` | Start API server only |
| `systemprompt infra services start --agents` | Start agents only |
| `systemprompt infra services start --mcp` | Start MCP servers only |
| `systemprompt infra services start agent <name>` | Start a specific agent |
| `systemprompt infra services start mcp <name>` | Start a specific MCP server |
| `systemprompt infra services start --kill-port-process` | Kill process using the port first |
| `systemprompt infra services serve` | Start API server (auto-starts agents and MCP) |
| `systemprompt infra services serve --foreground` | Run in foreground mode |

## Stopping Services

| Command | Purpose |
|---------|---------|
| `systemprompt infra services stop` | Stop all services |
| `systemprompt infra services stop --api` | Stop API server only |
| `systemprompt infra services stop --agents` | Stop all agents |
| `systemprompt infra services stop --mcp` | Stop all MCP servers |
| `systemprompt infra services stop agent <name>` | Stop a specific agent |
| `systemprompt infra services stop agent <name> --force` | Force stop (SIGKILL) |

## Restarting Services

| Command | Purpose |
|---------|---------|
| `systemprompt infra services restart` | Restart all services |
| `systemprompt infra services restart --failed` | Restart only failed services |
| `systemprompt infra services restart --agents` | Restart all agents |
| `systemprompt infra services restart --mcp` | Restart all MCP servers |
| `systemprompt infra services restart api` | Restart the API service |
| `systemprompt infra services restart agent <name>` | Restart a specific agent |
| `systemprompt infra services restart mcp <name>` | Restart a specific MCP server |

## Process Cleanup

| Command | Purpose |
|---------|---------|
| `systemprompt infra services cleanup --dry-run` | Preview cleanup |
| `systemprompt infra services cleanup -y` | Clean up orphaned processes |

## Standard Workflow

1. **Check status** to see which services are running and healthy
2. **Identify issues** -- look for stopped or unhealthy services
3. **Operate** -- start, stop, or restart as needed
4. **Verify** -- check status again to confirm the change

## Common Tasks

### Health Check All Services

```bash
systemprompt infra services status --detailed --health
```

### Restart After Issues

```bash
systemprompt infra services restart --failed
systemprompt infra services status
```

### Restart a Stuck Agent

```bash
systemprompt infra services stop agent <name> --force
systemprompt infra services start agent <name>
systemprompt infra services status
```

### Clean Up After a Crash

```bash
systemprompt infra services cleanup --dry-run
systemprompt infra services cleanup -y
systemprompt infra services status
systemprompt infra services start
```

## Important Notes

- Always check status before and after service operations
- Stopping the API service will make the platform unavailable
- Use `--force` only when a graceful stop fails
- Cleanup should be run after unexpected crashes to remove orphaned processes
- Use `--kill-port-process` if a port is already occupied
- Use `--help` on any subcommand for full flag reference
