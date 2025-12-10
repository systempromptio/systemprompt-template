# Logging Guidelines

This guide explains how to use logging effectively in SystemPrompt Core for debugging, auditing, and system monitoring.

## Quick Start

### Basic Logging
```rust
use systemprompt_core_logging::LogService;

let logger = LogService::new(db_pool.clone(), request_context.log_context());

// Simple logs
logger.info("module", "Operation completed").await.ok();
logger.error("module", "Operation failed").await.ok();
logger.warn("module", "Warning message").await.ok();
logger.debug("module", "Debug information").await.ok();
```

### Logging with Metadata
```rust
logger.log(
    LogLevel::Info,
    "module",
    "Operation completed",
    Some(serde_json::json!({
        "status": "success",
        "duration_ms": 234,
        "item_count": 42,
    })),
).await.ok();
```

## Core Principles

### 1. Always Ignore Logging Errors
Logging should never cause operations to fail. Always use `.ok()` to ignore errors:

```rust
// ✅ CORRECT
logger.info("module", "message").await.ok();

// ❌ WRONG
logger.info("module", "message").await?;  // Don't propagate logging errors
```

### 2. Use Appropriate Log Levels

| Level | When to Use | Example |
|-------|------------|---------|
| `Error` | Critical failures that need attention | Database connection lost |
| `Warn` | Unexpected situations that don't stop operation | Retrying failed request |
| `Info` | Important business events | Task created, operation completed |
| `Debug` | Details useful for debugging | Parameter values, intermediate results |
| `Trace` | Very detailed information | Function entry/exit, loop iterations |

### 3. Include Metadata for Important Operations

Always include metadata for operations that:
- Create or modify data (tasks, artifacts, contexts)
- Complete long-running operations
- Handle failures or errors
- Change system state

```rust
// Creating a resource - include what was created
logger.log(
    LogLevel::Info,
    "api",
    "Task created",
    Some(json!({
        "task_id": task.id,
        "context_id": task.context_id,
        "tool_name": tool_name,
    })),
).await.ok();

// Completing an operation - include duration and result
let start = std::time::Instant::now();
let result = perform_operation().await?;
logger.log(
    LogLevel::Info,
    "processing",
    "Batch processing completed",
    Some(json!({
        "items_processed": result.count,
        "errors": result.errors,
        "duration_ms": start.elapsed().as_millis(),
    })),
).await.ok();
```

## Context Propagation

Context propagation ensures logs from related operations are grouped together for debugging.

### When Task ID Becomes Available

Update the logger after a task is created:

```rust
let mut logger = LogService::new(db_pool.clone(), request_context.log_context());

// Create task
let task_id = create_task(...).await?;

// Update logger with task_id
logger = logger.with_task_id(task_id.as_str());

// All subsequent logs will have task_id
logger.info("mcp_task", "Processing task").await.ok();
```

### Webhook Handlers with Task Events

Extract task ID from webhook requests:

```rust
let log_context = if matches!(event_type.as_str(), "task_completed" | "task_created") {
    request_context.log_context().with_task_id(&entity_id)
} else {
    request_context.log_context()
};

let logger = LogService::new(db.clone(), log_context);
```

### Scheduler Jobs with Unique Tracing

Each scheduled job run should have a unique trace ID:

```rust
let log_context = LogContext::new()
    .with_session_id("scheduler")
    .with_trace_id(&format!("scheduler-{}", uuid::Uuid::new_v4()))
    .with_user_id("system")
    .with_client_id("scheduler");

let logger = LogService::new(db_pool.clone(), log_context);
```

## Log Context Fields

Every log entry includes these fields automatically:

| Field | Purpose | Coverage Target |
|-------|---------|-----------------|
| `user_id` | Who performed the action | 100% |
| `session_id` | Which session | 100% |
| `trace_id` | Request identifier for tracing | 100% |
| `task_id` | Task being executed (optional) | 90% of task logs |
| `context_id` | Conversation/execution context (optional) | 95% of user operations |
| `client_id` | Which service/client | High coverage |
| `module` | Which module/subsystem | 100% |
| `level` | Log severity | 100% |
| `message` | Human-readable message | 100% |
| `metadata` | Rich structured data (optional) | 40% of important operations |

## Module Naming Conventions

Use consistent module names for easy filtering and analysis:

```
mcp_*          - MCP server operations (e.g., mcp_admin, mcp_task)
webhook_*      - Webhook handlers
scheduler_*    - Scheduled jobs
api_*          - API endpoints
database_*     - Database operations
auth_*         - Authentication/authorization
```

## Query Examples

### Find all logs for a task
```sql
SELECT * FROM logs
WHERE task_id = 'task_abc123'
ORDER BY timestamp;
```

### Find logs for a context
```sql
SELECT * FROM logs
WHERE context_id = 'ctx_xyz789'
ORDER BY timestamp;
```

### Find errors in a module
```sql
SELECT * FROM logs
WHERE module = 'mcp_admin'
AND level = 'error'
ORDER BY timestamp DESC;
```

### Check context coverage
```sql
SELECT
    COUNT(CASE WHEN context_id IS NOT NULL THEN 1 END) * 100.0 / COUNT(*) as coverage_pct
FROM logs
WHERE user_id NOT LIKE 'system%';
```

### Analyze operation duration
```sql
SELECT
    module,
    COUNT(*) as count,
    AVG(CAST(metadata->>'duration_ms' AS FLOAT)) as avg_duration_ms,
    MAX(CAST(metadata->>'duration_ms' AS FLOAT)) as max_duration_ms
FROM logs
WHERE metadata->>'duration_ms' IS NOT NULL
GROUP BY module;
```

## Common Patterns

### MCP Tool Execution
```rust
let task_id = task_helper::ensure_task_exists(...).await?;
logger = logger.with_task_id(task_id.as_str());

let start = std::time::Instant::now();
let result = execute_tool(...).await;

logger.log(
    LogLevel::Info,
    "mcp_admin",
    &format!("Tool executed | tool={}, status={}", tool_name, if result.is_ok() { "success" } else { "error" }),
    Some(json!({
        "tool_name": tool_name,
        "status": if result.is_ok() { "success" } else { "error" },
        "task_id": task_id.as_str(),
        "duration_ms": start.elapsed().as_millis(),
    })),
).await.ok();
```

### Database Operation with Error Handling
```rust
match operation.execute().await {
    Ok(count) => {
        logger.log(
            LogLevel::Info,
            "database",
            "Query executed successfully",
            Some(json!({
                "rows_affected": count,
                "operation": "update_users",
            })),
        ).await.ok();
    }
    Err(e) => {
        logger.log(
            LogLevel::Error,
            "database",
            &format!("Query failed: {}", e),
            Some(json!({
                "error": e.to_string(),
                "operation": "update_users",
            })),
        ).await.ok();
    }
}
```

### Batch Processing Job
```rust
let mut processed = 0;
let mut errors = 0;

for item in items {
    match process_item(&item).await {
        Ok(_) => processed += 1,
        Err(e) => {
            logger.warn("scheduler", &format!("Failed to process item: {}", e)).await.ok();
            errors += 1;
        }
    }
}

logger.log(
    LogLevel::Info,
    "scheduler",
    &format!("Batch processing completed | processed={}, errors={}", processed, errors),
    Some(json!({
        "job_name": "process_batch",
        "items_processed": processed,
        "items_failed": errors,
        "total_items": items.len(),
    })),
).await.ok();
```

## Anti-Patterns to Avoid

### ❌ Don't log with string concatenation in message
```rust
// Wrong - message should be simple, use metadata for complex data
logger.info(
    "module",
    &format!("User {} logged in from {} with role {}", user_id, ip, role)
).await.ok();

// Correct
logger.log(
    LogLevel::Info,
    "auth",
    "User logged in",
    Some(json!({
        "user_id": user_id,
        "ip": ip,
        "role": role,
    })),
).await.ok();
```

### ❌ Don't use `.info()` when you should use `.log()` with metadata
```rust
// Wrong - important operational data is lost
logger.info("mcp_admin", "Tool executed successfully").await.ok();

// Correct - metadata provides searchability
logger.log(
    LogLevel::Info,
    "mcp_admin",
    "Tool executed successfully",
    Some(json!({
        "tool_name": tool_name,
        "duration_ms": elapsed,
        "task_id": task_id,
    })),
).await.ok();
```

### ❌ Don't log inside loops without aggregating
```rust
// Wrong - creates excessive log entries
for item in items {
    logger.info("processing", &format!("Processing item {}", item.id)).await.ok();
}

// Correct - aggregate and log once
let start = std::time::Instant::now();
let mut count = 0;
for item in items {
    match process(item).await {
        Ok(_) => count += 1,
        Err(e) => {
            logger.warn("processing", &format!("Failed: {}", e)).await.ok();
        }
    }
}

logger.log(
    LogLevel::Info,
    "processing",
    "Batch completed",
    Some(json!({
        "items_processed": count,
        "total_items": items.len(),
        "duration_ms": start.elapsed().as_millis(),
    })),
).await.ok();
```

### ❌ Don't propagate logging errors
```rust
// Wrong
logger.info("module", "message").await?;

// Correct
logger.info("module", "message").await.ok();
```

## Debugging with Logs

### Finding a specific operation
```bash
# Find all logs for a task
curl "http://localhost:8080/api/v1/logs?task_id=task_abc123&format=json" | jq .

# Find errors in last hour
curl "http://localhost:8080/api/v1/logs?level=error&since=1h&format=json" | jq .
```

### Streaming logs in development
```bash
just log
```

### Analyzing patterns
```sql
-- Find slowest operations
SELECT module, message,
       CAST(metadata->>'duration_ms' AS FLOAT) as duration_ms
FROM logs
WHERE metadata->>'duration_ms' IS NOT NULL
ORDER BY CAST(metadata->>'duration_ms' AS FLOAT) DESC
LIMIT 10;
```

## Coverage Goals

| Field | Current | Target | Priority |
|-------|---------|--------|----------|
| task_id | 3% | 90% | High |
| context_id | 54% | 95% | High |
| metadata | 9% | 40% | Medium |
| trace_id | 100% | 100% | ✅ Done |

To check coverage:
```sql
-- Task ID coverage
SELECT ROUND(COUNT(CASE WHEN task_id IS NOT NULL THEN 1 END) * 100.0 / COUNT(*), 1) as task_id_coverage
FROM logs WHERE module LIKE 'mcp_%' OR module LIKE 'task_%';

-- Context ID coverage
SELECT ROUND(COUNT(CASE WHEN context_id IS NOT NULL THEN 1 END) * 100.0 / COUNT(*), 1) as context_id_coverage
FROM logs WHERE user_id NOT LIKE 'system%';

-- Metadata coverage
SELECT ROUND(COUNT(CASE WHEN metadata IS NOT NULL AND metadata != '' THEN 1 END) * 100.0 / COUNT(*), 1) as metadata_coverage
FROM logs;
```

## Further Reading

- [LogService README](../crates/modules/log/src/services/README.md)
- [Rust Standards Guide](../instructions/rust.md)
- [Architecture Guide](./ARCHITECTURE.md)
