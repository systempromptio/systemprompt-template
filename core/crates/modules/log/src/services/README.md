# Logging Services

This module provides two distinct services for different purposes:

## LogService - Data Persistence & Audit

**Purpose**: Store log entries to database for audit trails, debugging, and system monitoring.

**When to use**:
- Background processes and service operations
- Error tracking and system monitoring
- Audit trails for security and compliance
- Long-running operations that need historical tracking
- Internal system events that should be persisted

**Usage**:
```rust
use systemprompt_core_logging::LogService;

// Simple logging
LogService::log_info("mcp_server", "Server started successfully").await?;
LogService::log_error("database", "Connection failed").await?;
LogService::log_warn("auth", "Invalid token attempt").await?;

// With metadata
LogService::log_with_metadata(
    LogLevel::Info,
    "user_action",
    "User login",
    serde_json::json!({"user_id": "123", "ip": "192.168.1.1"})
).await?;
```

**Key features**:
- Persists to database for historical analysis
- Includes metadata support for structured logging
- Automatic terminal output in development mode
- Module-based categorization

## CliService - User Interface Display

**Purpose**: Display information to users with consistent styling, formatting, and interactivity.

**When to use**:
- User-facing messages and feedback
- Command-line tools and interactive operations
- Tables, progress indicators, and formatted output
- User confirmations and prompts
- Status updates during operations

**Usage**:
```rust
use systemprompt_core_logging::services::cli::CliService;

// Basic messages
CliService::success("Database connection established");
CliService::warning("Configuration file not found, using defaults");
CliService::error("Failed to start service");
CliService::info("Processing 150 records...");

// Sections and organization
CliService::section("MCP Servers");

// Tables
let headers = &["Name", "Status", "Port"];
let rows = vec![
    vec!["Server1".to_string(), "Running".to_string(), "8080".to_string()],
    vec!["Server2".to_string(), "Stopped".to_string(), "8081".to_string()],
];
CliService::table(headers, &rows);

// User interaction
let confirmed = CliService::confirm("Delete all data?")?;
if confirmed {
    CliService::info("Operation cancelled");
}
```

**Key features**:
- Consistent visual styling with icons and colors
- Unicode table formatting
- Interactive prompts and confirmations
- No persistence - immediate display only

## Decision Guide

**Use LogService when**:
- You need to track what happened for debugging later
- Recording system events, errors, or user actions
- Building audit trails or compliance logs
- Background processing that users don't see directly

**Use CliService when**:
- Communicating directly with users
- Displaying status, progress, or results
- Creating interactive command-line experiences
- Showing formatted data (tables, lists, etc.)

## Context Propagation for Task Tracking

**Purpose**: Correlate logs across async boundaries and distributed operations using context IDs.

When working with tasks, webhooks, or distributed processes, propagate context through the request context and update the logger when new IDs become available.

**Pattern 1: Updating logger with task_id after creation**:
```rust
// Create logger first
let mut logger = LogService::new(db_pool.clone(), request_context.log_context());

// Get task_id from operation
let task_id = create_task(...).await?;

// Update logger with task_id - all subsequent logs will have task_id
logger = logger.with_task_id(task_id.as_str());

// Now all logs include task_id correlation
logger.info("module", "Task operation completed").await.ok();
```

**Pattern 2: Webhook handler with task correlation**:
```rust
let log_context = if matches!(event_type.as_str(), "task_completed" | "task_created") {
    // Extract task_id from event for task events
    request_context.log_context().with_task_id(&event.entity_id)
} else {
    request_context.log_context()
};

let logger = LogService::new(db.clone(), log_context);
// All logs now have task_id correlation
```

**Pattern 3: Scheduler jobs with operation tracing**:
```rust
let log_context = LogContext::new()
    .with_session_id("scheduler")
    .with_trace_id(&format!("scheduler-{}", uuid::Uuid::new_v4()))
    .with_user_id("system")
    .with_client_id("scheduler");

let logger = LogService::new(db_pool.clone(), log_context);
// Each job run has unique trace_id for distributed tracing
```

**Key methods for context propagation**:
- `log_context.with_task_id(id)` - Add task ID for task-related logs
- `log_context.with_context_id(id)` - Add context ID for conversation tracking
- `log_context.with_trace_id(id)` - Add trace ID for distributed tracing
- `log_context.with_client_id(id)` - Identify the client/process origin
- `logger.with_task_id(id)` - Update existing logger with task ID
- `logger.with_context_id(id)` - Update existing logger with context ID

**Why context propagation matters**:
- **End-to-end tracing**: Follow a request/task across all modules and services
- **Debugging**: Query all logs for a specific task/context to understand what happened
- **Correlation**: Automatically group related log entries across async operations
- **Metrics**: Extract duration, error rates, and performance from context-based log grouping

**Coverage targets**:
- `task_id`: 90%+ of task-related logs should have task_id
- `context_id`: 95%+ of user-context operations should have context_id
- `trace_id`: 100% - all logs must have trace_id
- `metadata`: 40%+ of important operations should include rich metadata

## Examples in Practice

```rust
// Starting a service - combine both
async fn start_mcp_service() -> Result<()> {
    // Log the operation for audit
    LogService::log_info("mcp_service", "Starting MCP service").await?;

    // Show user what's happening
    CliService::info("Starting MCP service...");

    match start_service().await {
        Ok(_) => {
            LogService::log_info("mcp_service", "MCP service started successfully").await?;
            CliService::success("MCP service started successfully");
        }
        Err(e) => {
            LogService::log_error("mcp_service", format!("Failed to start: {}", e)).await?;
            CliService::error(&format!("Failed to start service: {}", e));
        }
    }

    Ok(())
}

// Database operations
async fn query_database() -> Result<()> {
    let results = execute_query().await?;

    // Log the query for audit (with metadata)
    LogService::log_with_metadata(
        LogLevel::Info,
        "database",
        "Query executed",
        serde_json::json!({"query": "SELECT * FROM users", "rows": results.len()})
    ).await?;

    // Display results to user
    results.display_with_cli();

    Ok(())
}
```