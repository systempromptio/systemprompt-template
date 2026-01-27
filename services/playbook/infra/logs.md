---
title: "Log Querying Playbook"
description: "Query application logs stored in PostgreSQL."
keywords:
  - logs
  - query
  - debugging
  - tracing
---

# Log Querying

Query application logs stored in the PostgreSQL `logs` table.

> **Help**: `{ "command": "playbook logs" }` via `systemprompt_help`

---

## Table Schema

| Column | Type | Description |
|--------|------|-------------|
| `id` | text | Unique log entry ID |
| `timestamp` | timestamp | When the log was created |
| `level` | varchar | Log level: `DEBUG`, `INFO`, `WARN`, `ERROR` |
| `module` | varchar | Source module (e.g., `ai`, `mcp_task`, `analytics`) |
| `message` | text | Log message content |
| `metadata` | text | JSON metadata (nullable) |
| `user_id` | varchar | Associated user ID (nullable) |
| `session_id` | varchar | Session ID (nullable) |
| `task_id` | varchar | A2A Task ID (nullable) |
| `trace_id` | varchar | Trace ID for distributed tracing (nullable) |
| `context_id` | varchar | A2A Context ID (nullable) |
| `client_id` | varchar | Client identifier (nullable) |

---

## Commands

### Basic Query

```json
// MCP: systemprompt
{ "command": "infra db query \"SELECT * FROM logs LIMIT 10\"" }
{ "command": "infra db query \"SELECT * FROM logs LIMIT 10\" --format json" }
```

---

## Common Queries

### Recent Errors

```json
// MCP: systemprompt
{ "command": "infra db query \"SELECT timestamp, level, module, message FROM logs WHERE level = 'ERROR' ORDER BY timestamp DESC LIMIT 20\"" }
```

### Errors by Module

```json
// MCP: systemprompt
{ "command": "infra db query \"SELECT timestamp, message, task_id, context_id FROM logs WHERE level = 'ERROR' AND module = 'ai' ORDER BY timestamp DESC LIMIT 10\"" }
```

### Filter by Task ID

Task IDs are UUIDs. Use exact match or partial match with LIKE:

```json
// MCP: systemprompt
// Exact match
{ "command": "infra db query \"SELECT timestamp, level, module, message FROM logs WHERE task_id = '08a90adb-f686-4029-bdaf-62049f662566' ORDER BY timestamp\"" }

// Partial match (first 8 chars)
{ "command": "infra db query \"SELECT timestamp, level, module, message FROM logs WHERE task_id LIKE '%08a90adb%' ORDER BY timestamp DESC\"" }
```

### Filter by Context ID

```json
// MCP: systemprompt
{ "command": "infra db query \"SELECT timestamp, level, module, message FROM logs WHERE context_id = '0822f708-6b42-45f6-9317-81b25a6166d9' ORDER BY timestamp\"" }
```

### Find Task and Context IDs

```json
// MCP: systemprompt
// List distinct task IDs
{ "command": "infra db query \"SELECT DISTINCT task_id FROM logs WHERE task_id IS NOT NULL ORDER BY task_id LIMIT 20\"" }

// Recent logs with task IDs
{ "command": "infra db query \"SELECT timestamp, level, module, task_id, context_id FROM logs WHERE task_id IS NOT NULL ORDER BY timestamp DESC LIMIT 10\"" }
```

### All Logs for a Task

```json
// MCP: systemprompt
{ "command": "infra db query \"SELECT timestamp, level, module, context_id, SUBSTRING(message, 1, 200) as msg FROM logs WHERE task_id = '08a90adb-f686-4029-bdaf-62049f662566' ORDER BY timestamp\"" }
```

### Log Level Distribution

```json
// MCP: systemprompt
{ "command": "infra db query \"SELECT level, COUNT(*) as count FROM logs GROUP BY level ORDER BY count DESC\"" }
```

### Logs by Time Range

```json
// MCP: systemprompt
{ "command": "infra db query \"SELECT timestamp, level, module, message FROM logs WHERE timestamp > NOW() - INTERVAL '1 hour' ORDER BY timestamp DESC LIMIT 50\"" }
```

### Module Activity

```json
// MCP: systemprompt
{ "command": "infra db query \"SELECT module, COUNT(*) as count FROM logs GROUP BY module ORDER BY count DESC LIMIT 20\"" }
```

---

## Streaming Logs

```json
// MCP: systemprompt
{ "command": "infra logs stream" }
{ "command": "infra logs stream --level error" }
{ "command": "infra logs stream --module ai" }
```

---

## Trace Logs

```json
// MCP: systemprompt
{ "command": "infra logs trace show <TASK_ID>" }
{ "command": "infra logs trace show <TASK_ID> --all" }
```

---

## Output Formats

```json
// MCP: systemprompt
// Table (default)
{ "command": "infra db query \"SELECT * FROM logs LIMIT 5\"" }

// JSON
{ "command": "infra db query \"SELECT * FROM logs LIMIT 5\" --format json" }

// CSV
{ "command": "infra db query \"SELECT * FROM logs LIMIT 5\" --format csv" }
```

---

## Tips

1. **Use SUBSTRING for long messages**: `SUBSTRING(message, 1, 200)` truncates to 200 chars
2. **Partial UUID matching**: Use `LIKE '%first8chars%'` when you only have part of a task/context ID
3. **Time-based filtering**: Use `NOW() - INTERVAL 'X hours/days'` for recent logs
4. **NULL handling**: Add `WHERE column IS NOT NULL` to filter out nulls

---

## Quick Reference

| Task | Command |
|------|---------|
| Recent errors | `SELECT ... WHERE level = 'ERROR' ORDER BY timestamp DESC LIMIT 20` |
| By task ID | `SELECT ... WHERE task_id = '<id>' ORDER BY timestamp` |
| By module | `SELECT ... WHERE module = '<module>' LIMIT 20` |
| Last hour | `SELECT ... WHERE timestamp > NOW() - INTERVAL '1 hour'` |
| Stream logs | `infra logs stream` |
| Trace task | `infra logs trace show <TASK_ID>` |
| Level distribution | `SELECT level, COUNT(*) ... GROUP BY level` |
