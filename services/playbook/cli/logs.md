---
title: "Logs & Debugging Playbook"
description: "View, search, and analyze logs for debugging and monitoring."
author: "SystemPrompt"
slug: "cli-logs"
keywords: "logs, debugging, tracing, monitoring, errors"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Logs & Debugging Playbook

View, search, and analyze logs for debugging and monitoring.

---

## View Recent Logs

```json
{ "command": "infra logs view" }
{ "command": "infra logs view --tail 50" }
{ "command": "infra logs view --tail 100" }
```

---

## Filter by Log Level

```json
{ "command": "infra logs view --level error" }
{ "command": "infra logs view --level warn" }
{ "command": "infra logs view --level info" }
{ "command": "infra logs view --level debug" }
```

---

## Filter by Time Range

```json
{ "command": "infra logs view --since 1h" }
{ "command": "infra logs view --since 24h" }
{ "command": "infra logs view --since 7d" }
{ "command": "infra logs view --level error --since 1h" }
```

---

## Stream Logs in Real-Time

Stream logs live in terminal (not MCP):

```bash
systemprompt infra logs stream
systemprompt infra logs stream --level error
systemprompt infra logs stream --level error --module agent
```

---

## Search Logs

```json
{ "command": "infra logs search \"connection refused\"" }
{ "command": "infra logs search \"timeout\" --since 1h" }
{ "command": "infra logs search \"job failed\"" }
{ "command": "infra logs search \"error\" --level error" }
```

---

## View Log Summary

```json
{ "command": "infra logs summary" }
{ "command": "infra logs summary --since 1h" }
{ "command": "infra logs summary --since 24h" }
```

---

## Show Specific Log Entry

```json
{ "command": "infra logs show <log-id>" }
```

---

## Trace Execution

View detailed traces for debugging request flows:

```json
{ "command": "infra logs trace" }
{ "command": "infra logs trace show <trace-id>" }
{ "command": "infra logs trace show <trace-id> --all" }
```

---

## Inspect AI Requests

```json
{ "command": "infra logs request list" }
{ "command": "infra logs request list --limit 1" }
{ "command": "infra logs request show <request-id>" }
```

**Tip**: When using `--limit 1`, the command shows a trace hint for quick access to the full trace:
```
ℹ For full trace: systemprompt infra logs trace show <trace-id> --all
```

---

## Audit a Request

Full audit trail for an AI request:

```json
{ "command": "infra logs audit <request-id>" }
{ "command": "infra logs audit <request-id> --full" }
```

---

## View MCP Tool Executions

```json
{ "command": "infra logs tools" }
{ "command": "infra logs tools --since 1h" }
```

---

## Export Logs

```json
{ "command": "infra logs export --format json --since 24h -o logs.json" }
{ "command": "infra logs export --format csv --since 7d -o logs.csv" }
```

---

## Cleanup Old Logs

```json
{ "command": "infra logs cleanup --days 30" }
{ "command": "infra logs cleanup --days 7 --dry-run" }
{ "command": "infra logs delete" }
```

---

## Common Debugging Workflows

### Find Recent Errors

```json
{ "command": "infra logs view --level error --since 1h" }
{ "command": "infra logs summary --since 1h" }
```

### Debug a Failed Request

1. Find the error:
```json
{ "command": "infra logs view --level error --since 1h" }
```

2. Get the request ID from the error, then audit:
```json
{ "command": "infra logs audit <request-id> --full" }
```

3. View the full trace:
```json
{ "command": "infra logs trace show <trace-id> --all" }
```

### Debug Agent Issues

```json
{ "command": "admin agents logs <agent-name>" }
{ "command": "infra logs search \"agent\" --level error --since 1h" }
```

### Debug MCP Server Issues

```json
{ "command": "plugins mcp logs <server-name>" }
{ "command": "infra logs tools --since 1h" }
```

---

## Local Log Files (Terminal Only)

MCP servers and agents write to log files in the `logs/` directory:

```bash
# List available log files
ls -la logs/

# View MCP server logs
tail -100 logs/mcp-content-manager.log
tail -100 logs/mcp-systemprompt.log

# View agent logs
tail -100 logs/agent-linkedin.log
tail -100 logs/agent-blog.log

# Search for specific errors in log files
grep -i "error\|failed" logs/mcp-content-manager.log | tail -50
grep "research_content\|gemini" logs/mcp-content-manager.log | tail -20
```

**Note**: Local log files only contain logs from locally-running services. When using a remote profile (e.g., production), use `plugins mcp logs` to fetch logs from the remote database.

---

## Database Queries for Tool Executions

Query the `mcp_tool_executions` table for detailed tool execution history:

```json
{ "command": "infra db query \"SELECT created_at, tool_name, status, error_message FROM mcp_tool_executions WHERE tool_name = 'research_content' ORDER BY created_at DESC LIMIT 5\"" }
{ "command": "infra db query \"SELECT created_at, server_name, tool_name, status, error_message FROM mcp_tool_executions WHERE status != 'success' ORDER BY created_at DESC LIMIT 10\"" }
```

Key columns in `mcp_tool_executions`:
- `tool_name`, `server_name` - identify the tool
- `status` - success/failure
- `error_message` - detailed error text
- `execution_time_ms` - performance metric
- `trace_id` - correlate with other logs

---

## Local vs Remote Logs

| Profile Type | Log Source | Command |
|--------------|------------|---------|
| Local | Log files in `logs/` | `tail logs/mcp-*.log` |
| Local | Database | `plugins mcp logs <server>` |
| Remote/Production | Remote database only | `plugins mcp logs <server>` |

When debugging production issues, always use CLI commands which fetch from the remote database. Local log files won't contain production errors.

---

## Troubleshooting

**No logs found** -- check time range with `--since`. Logs may have been cleaned up.

**Too many logs** -- use `--level error` or `--level warn` to filter. Add `--tail 50` to limit results.

**Can't find specific error** -- use `infra logs search "pattern"` to search by keyword.

**Need full context** -- use `infra logs audit <request-id> --full` for complete request trail.

**Generic "Tool execution failed"** -- check MCP logs with `plugins mcp logs <server>` or query `mcp_tool_executions` table for `error_message`.

**Local logs empty but tool failing** -- you may be on a remote profile. Check with `admin session show` and use database queries instead.

---

## Quick Reference

| Task | Command |
|------|---------|
| Recent logs | `infra logs view --tail 50` |
| Error logs | `infra logs view --level error` |
| Last hour errors | `infra logs view --level error --since 1h` |
| Search logs | `infra logs search "pattern"` |
| Log summary | `infra logs summary --since 1h` |
| View trace | `infra logs trace show <id>` |
| View trace (full) | `infra logs trace show <id> --all` |
| Latest AI request | `infra logs request list --limit 1` |
| Audit request | `infra logs audit <id> --full` |
| Tool executions | `infra logs tools` |
| MCP server logs | `plugins mcp logs <server>` |
| Export logs | `infra logs export --format json -o logs.json` |
| Clean old logs | `infra logs cleanup --days 30` |
| Stream live | `infra logs stream` (terminal only) |