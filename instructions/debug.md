# Debugging AI Requests

## Quick Start: Debug a Failed Task

Given a task_id (e.g., `5961fac1`), run these queries in order:

```bash
# 1. Find the task and get its trace_id
PGPASSWORD='systemprompt_dev_password' psql -h localhost -p 5432 -U systemprompt -d systemprompt_dev -c \
  "SELECT task_id, status, trace_id, agent_name, created_at FROM agent_tasks WHERE task_id LIKE '%5961fac1%'"

# 2. Get execution steps (shows error_message if any)
PGPASSWORD='systemprompt_dev_password' psql -h localhost -p 5432 -U systemprompt -d systemprompt_dev -c \
  "SELECT step_type, title, status, error_message, tool_name FROM task_execution_steps WHERE task_id = 'FULL_TASK_ID' ORDER BY step_number"

# 3. Get logs for the trace (shows full execution flow)
PGPASSWORD='systemprompt_dev_password' psql -h localhost -p 5432 -U systemprompt -d systemprompt_dev -c \
  "SELECT module, level, LEFT(message, 300) as msg, timestamp FROM logs WHERE trace_id = 'TRACE_ID' ORDER BY timestamp"
```

## Database Tables

### agent_tasks
Primary task table - use `task_id` (not `id`):

```sql
-- Find task by partial ID
SELECT task_id, status, trace_id, agent_name, metadata, created_at
FROM agent_tasks
WHERE task_id LIKE '%partial_id%';

-- Get recent failed tasks
SELECT task_id, status, trace_id, agent_name, created_at
FROM agent_tasks
WHERE status = 'failed'
ORDER BY created_at DESC
LIMIT 10;
```

### task_execution_steps
Shows step-by-step execution with errors:

```sql
-- Get steps for a task (includes error_message)
SELECT step_number, step_type, title, status, error_message, tool_name, duration_ms
FROM task_execution_steps
WHERE task_id = 'your-task-id'
ORDER BY step_number;
```

### logs
Primary source for detailed debugging:

```sql
-- Get logs for a specific trace
SELECT module, level, LEFT(message, 300) as msg, timestamp
FROM logs
WHERE trace_id = 'your-trace-id'
ORDER BY timestamp;

-- Get recent error logs
SELECT module, level, LEFT(message, 200) as msg, timestamp
FROM logs
WHERE level = 'ERROR'
ORDER BY timestamp DESC
LIMIT 20;
```

### ai_requests
AI API call metadata. Note: has both `id` (UUID PK) and `request_id` (app-level ID):

```sql
-- Get recent AI requests
SELECT id, request_id, provider, model, status, latency_ms, created_at
FROM ai_requests
ORDER BY created_at DESC
LIMIT 10;

-- Get specific request by request_id (the one shown in logs)
SELECT id, request_id, provider, model, status, latency_ms
FROM ai_requests
WHERE request_id = 'your-request-id';
```

### ai_request_messages
Messages sent to AI. FK is `request_id` -> `ai_requests.id` (the UUID, not the app request_id):

```sql
-- Get messages for a request (use ai_requests.id, not request_id)
SELECT role, LEFT(content, 2000) as content_preview, sequence_number
FROM ai_request_messages
WHERE request_id = 'ai-requests-uuid-id'
ORDER BY sequence_number;

-- Or join through ai_requests using the app request_id from logs
SELECT m.role, LEFT(m.content, 2000) as content_preview, m.sequence_number
FROM ai_request_messages m
JOIN ai_requests r ON m.request_id = r.id
WHERE r.request_id = 'app-request-id-from-logs'
ORDER BY m.sequence_number;
```

### ai_request_tool_calls
Tool calls made by AI:

```sql
-- Get tool calls for a request
SELECT tc.tool_name, LEFT(tc.tool_input, 500) as args, tc.created_at
FROM ai_request_tool_calls tc
WHERE tc.request_id = 'ai-requests-uuid-id';
```

## Key Log Modules

- `planned` - Planning strategy logs
- `ai` - AI service logs (planning, generation)
- `gemini_provider` - Gemini-specific logs
- `message_processor` - Message processing flow
- `a2a_jsonrpc` - A2A protocol errors
- `plan_executor` - Tool execution logs
- `sse_stream` / `sse_loop` / `sse_error` - SSE streaming logs
- `tyingshoelaces` - MCP tool execution logs

## Common Error Patterns

### MALFORMED_FUNCTION_CALL
Gemini returned an invalid tool call. Check:
1. Tool schemas in agent config
2. Recent changes to MCP tool definitions
3. The prompt might be confusing the model

### Tool execution failures
Check `mcp_tool_executions` table:
```sql
SELECT tool_name, status, LEFT(error_message, 200), created_at
FROM mcp_tool_executions
WHERE status = 'error'
ORDER BY created_at DESC
LIMIT 10;
```
