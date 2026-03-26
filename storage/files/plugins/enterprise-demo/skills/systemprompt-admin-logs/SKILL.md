---
name: "Log Management"
description: "View, filter, stream, trace, and audit logs via the systemprompt CLI"
---

# Log Management

You view, filter, and analyze system logs using the systemprompt CLI. All operations go through the `infra logs` domain.

## Viewing Logs

| Command | Purpose |
|---------|---------|
| `systemprompt infra logs view` | View recent log entries |
| `systemprompt infra logs view --level error` | Filter by level (error, warn, info, debug) |
| `systemprompt infra logs view --since 1h` | Filter by time range |
| `systemprompt infra logs view --level error --since 1h` | Combine filters |
| `systemprompt infra logs view --module <module>` | Filter by module |
| `systemprompt infra logs view -n 50` | Show last 50 entries |
| `systemprompt infra logs show <id>` | Show specific log/trace details |
| `systemprompt infra logs summary` | Log summary statistics |
| `systemprompt infra logs summary --since 24h` | Summary for a time period |

## Streaming Logs

| Command | Purpose |
|---------|---------|
| `systemprompt infra logs stream` | Stream logs in real-time |
| `systemprompt infra logs stream --level error` | Stream only errors |
| `systemprompt infra logs stream --module <module>` | Stream specific module |
| `systemprompt infra logs follow` | Alias for stream |

## Searching Logs

| Command | Purpose |
|---------|---------|
| `systemprompt infra logs search "error"` | Search logs by pattern |
| `systemprompt infra logs search "timeout" --since 24h` | Search with time filter |
| `systemprompt infra logs search "error" --level error --limit 50` | Search with filters |
| `systemprompt infra logs search "error" --include-tools` | Include tool execution logs |

## AI Request Tracing

| Command | Purpose |
|---------|---------|
| `systemprompt infra logs request list` | List recent AI requests |
| `systemprompt infra logs request list --limit 10` | Limit results |
| `systemprompt infra logs request list --model <model>` | Filter by model |
| `systemprompt infra logs request list --provider <provider>` | Filter by provider |
| `systemprompt infra logs request show <request-id>` | Show AI request details |
| `systemprompt infra logs request show <request-id> --messages --tools --full` | Show full request |
| `systemprompt infra logs request stats` | Aggregate AI request statistics |
| `systemprompt infra logs audit <id> --full` | Full audit trail (request, task, or trace ID) |

## Execution Tracing

| Command | Purpose |
|---------|---------|
| `systemprompt infra logs trace list` | List recent execution traces |
| `systemprompt infra logs trace list --agent <name> --status failed` | Failed traces for an agent |
| `systemprompt infra logs trace list --has-mcp` | Traces with MCP tool calls |
| `systemprompt infra logs trace show <id>` | Show trace details |
| `systemprompt infra logs trace show <id> --all` | Show all trace data (steps, AI, MCP, artifacts) |

## MCP Tool Logs

| Command | Purpose |
|---------|---------|
| `systemprompt infra logs tools list` | List MCP tool executions |
| `systemprompt infra logs tools list --status error` | Failed tool executions |
| `systemprompt infra logs tools list --server <server>` | Filter by MCP server |
| `systemprompt plugins mcp logs <server-name>` | View MCP server logs directly |
| `systemprompt plugins mcp logs <server-name> -f` | Follow MCP server logs |

## Log Maintenance

| Command | Purpose |
|---------|---------|
| `systemprompt infra logs export --since 7d -o logs.json` | Export logs to file |
| `systemprompt infra logs export --format csv -o logs.csv` | Export as CSV |
| `systemprompt infra logs cleanup --older-than 30d --dry-run` | Preview cleanup |
| `systemprompt infra logs cleanup --older-than 30d -y` | Clean up old logs |
| `systemprompt infra logs delete -y` | Delete ALL logs (use with extreme caution) |

## Standard Workflow

1. **Quick check** -- `infra logs view --level error --since 1h` to find recent errors
2. **Summary** -- `infra logs summary --since 24h` for an overview
3. **Find request** -- `infra logs request list` to locate the failed AI request
4. **Audit** -- `infra logs audit <id> --full` to get full conversation context
5. **MCP logs** -- check `plugins mcp logs <server>` if tool errors are involved

## Common Tasks

### Debug a Recent Error

```bash
systemprompt infra logs view --level error --since 1h
systemprompt infra logs request list --limit 10
systemprompt infra logs audit <request-id> --full
```

### Debug MCP Tool Failures

```bash
systemprompt infra logs tools list --status error --since 1h
systemprompt plugins mcp logs <server-name>
```

### Debug Agent Issues

```bash
systemprompt infra logs trace list --agent <name> --status failed
systemprompt infra logs trace show <trace-id> --all
systemprompt infra logs audit <request-id> --full
```

### Monitor in Real-Time

```bash
systemprompt infra logs stream --level error
```

### Get AI Request Statistics

```bash
systemprompt infra logs request stats --since 24h
systemprompt infra logs request list --model <model> --limit 10
```

## Important Notes

- Log levels: `debug`, `info`, `warn`, `error`
- Time ranges use shorthand: `1h`, `24h`, `7d`, `30d`
- Request IDs from `request list` are used with `audit` for full context
- The `audit` command accepts request IDs, task IDs, or trace IDs
- Use `--help` on any subcommand for full flag reference
