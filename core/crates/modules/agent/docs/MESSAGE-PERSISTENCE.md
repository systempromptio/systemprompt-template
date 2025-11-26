# Message Persistence Guide

**Date**: 2025-10-17  
**Audience**: Developers working on the Agent module

---

## Quick Reference

### Use MessageService for Message Persistence

**✅ CORRECT**:
```rust
use crate::services::MessageService;

let message_service = MessageService::new(db_pool, logger);
message_service.persist_messages(
    &task_id,
    &context_id,
    vec![user_message, agent_message],
    Some(user_id),
    session_id,
    trace_id,
).await?;
```

**❌ WRONG**:
```rust
// Don't create messages with hardcoded sequences
let seq = 0;  // Race condition!
task_repo.persist_message(..., seq).await?;
```

---

## MessageService API

### Method 1: persist_messages

**When to use**: Persisting multiple messages at once (most common)

```rust
pub async fn persist_messages(
    &self,
    task_id: &TaskId,
    context_id: &ContextId,
    messages: Vec<Message>,
    user_id: Option<&str>,
    session_id: &str,
    trace_id: &str,
) -> Result<Vec<i32>>
```

**Returns**: Vector of sequence numbers assigned to each message

**Example**:
```rust
let sequences = message_service.persist_messages(
    &task_id,
    &context_id,
    vec![user_message, agent_message],
    Some("user-123"),
    "session-456",
    "trace-789",
).await?;

// sequences = vec![0, 1] (if first messages in task)
```

### Method 2: persist_message_in_tx

**When to use**: Within an existing transaction (advanced)

```rust
pub async fn persist_message_in_tx(
    &self,
    tx: &mut dyn DatabaseTransaction,
    message: &Message,
    task_id: &TaskId,
    context_id: &ContextId,
    user_id: Option<&str>,
    session_id: &str,
    trace_id: &str,
) -> Result<i32>
```

**Returns**: Sequence number assigned to message

**Example**:
```rust
let mut tx = db_pool.begin_transaction().await?;

let seq1 = message_service.persist_message_in_tx(
    &mut *tx, &msg1, &task_id, &context_id, 
    Some(user_id), session_id, trace_id
).await?;

let seq2 = message_service.persist_message_in_tx(
    &mut *tx, &msg2, &task_id, &context_id,
    Some(user_id), session_id, trace_id
).await?;

tx.commit().await?;
```

### Method 3: create_tool_execution_message

**When to use**: Creating synthetic messages for direct MCP tool calls

```rust
pub async fn create_tool_execution_message(
    &self,
    task_id: &TaskId,
    context_id: &ContextId,
    tool_name: &str,
    tool_args: &serde_json::Value,
    request_context: &RequestContext,
) -> Result<(String, i32)>
```

**Returns**: Tuple of (message_id, sequence_number)

**Example**:
```rust
let (message_id, seq) = message_service.create_tool_execution_message(
    &task_id,
    &context_id,
    "example_chart",
    &json!({"labels": [...], "datasets": [...]}),
    &request_context,
).await?;
```

---

## Common Patterns

### Pattern 1: A2A Message Flow

**Scenario**: User sends message to agent, agent responds

```rust
// 1. Create task
let task_id = task_repo.create_task_simple(...).await?;

// 2. Process AI (omitted for brevity)
let response_text = ai_service.process(...).await?;

// 3. Update task + save messages (ATOMIC)
task_repo.update_task_and_save_messages(
    &completed_task,
    &user_message,
    &agent_message,
    Some(user_id),
    session_id,
    trace_id,
).await?;
```

**Note**: Use `TaskRepository::update_task_and_save_messages()` when task status and messages must be updated together.

### Pattern 2: Direct MCP Tool Call

**Scenario**: User calls MCP tool directly (bypassing agent)

```rust
// 1. Ensure task exists
let task_id = ensure_task_exists(db_pool, &mut request_context, tool_name, &logger).await?;

// 2. Execute tool
let result = execute_tool(...).await?;

// 3. Create artifact
let artifact = create_artifact_from_result(result);

// 4. Publish (creates synthetic message + broadcasts)
publishing_service.publish_from_mcp(
    &artifact,
    &task_id,
    &context_id,
    tool_name,
    &tool_args,
    &request_context,
).await?;
```

**Note**: `publish_from_mcp()` internally calls `MessageService::create_tool_execution_message()`.

---

## Sequence Numbers

### How Sequences Work

- **Sequential integers** starting from 0 for each task
- **Calculated dynamically** within transactions (prevents race conditions)
- **Enforced by database** via UNIQUE constraint on `(task_uuid, sequence_number)`

### Sequence Calculation

```rust
// Internal to MessageService
let seq = TaskRepository::get_next_sequence_number_in_tx(tx, task_id).await?;

// Implementation:
// SELECT MAX(sequence_number) FROM task_messages WHERE task_uuid = ?
// Returns: max_seq + 1, or 0 if no messages exist
```

### Why Dynamic Calculation?

**Problem**: Hardcoded sequences cause race conditions

```rust
// ❌ WRONG - Two concurrent requests both try seq=0
Request A: persist_message(..., seq=0) // Succeeds
Request B: persist_message(..., seq=0) // FAILS (UNIQUE constraint violation)
```

**Solution**: Calculate sequence within transaction

```rust
// ✅ CORRECT - Transaction isolation prevents conflicts
Request A: BEGIN → calc_seq()=0 → persist(seq=0) → COMMIT
Request B: BEGIN → calc_seq()=1 → persist(seq=1) → COMMIT
```

---

## Transaction Patterns

### When to use transactions?

**Use transactions when**:
- Multiple database operations must succeed/fail together
- Sequence number calculation + message insertion
- Task status update + message insertion

**Example**:
```rust
let mut tx = db_pool.begin_transaction().await?;

// Operation 1
tx.execute("UPDATE agent_tasks SET status = ? WHERE uuid = ?", ...).await?;

// Operation 2
let seq = get_next_sequence_number_in_tx(&mut *tx, task_id).await?;

// Operation 3
persist_message_with_tx(&mut *tx, message, ..., seq).await?;

// All or nothing
tx.commit().await?;
```

---

## Error Handling

### Best Practices

**1. Use `.map_err()` for context**:
```rust
message_service.persist_messages(...).await
    .map_err(|e| anyhow!("Failed to save messages for task {}: {}", task_id, e))?;
```

**2. Log errors with structured data**:
```rust
logger.error(
    "message_persistence",
    &format!("Failed to persist message: {}", e),
).await.ok();
```

**3. Rollback automatically**:
```rust
// Transactions rollback automatically on error (Drop impl)
let mut tx = db_pool.begin_transaction().await?;
persist_message(...).await?; // If this fails, tx rolls back automatically
tx.commit().await?; // Only commits if all operations succeed
```

---

## Testing

### Unit Test Example

```rust
#[tokio::test]
async fn test_concurrent_message_persistence() {
    let db_pool = create_test_db().await;
    let logger = LogService::system(db_pool.clone());
    let message_service = MessageService::new(db_pool.clone(), logger);

    // Create task
    let task_id = TaskId::generate();
    create_task_simple(&task_id).await;

    // Spawn 10 concurrent operations
    let handles: Vec<_> = (0..10).map(|i| {
        let service = message_service.clone();
        let task_id = task_id.clone();
        tokio::spawn(async move {
            let msg = create_test_message(i);
            service.persist_messages(
                &task_id,
                &context_id,
                vec![msg],
                Some("user"),
                "session",
                "trace",
            ).await
        })
    }).collect();

    // All should succeed (no UNIQUE constraint errors)
    for handle in handles {
        assert!(handle.await.is_ok());
    }

    // Verify sequences are 0..9
    let messages = task_repo.get_messages_by_task(&task_id).await.unwrap();
    assert_eq!(messages.len(), 10);
}
```

---

## Migration from Old Code

### Before (Deprecated)

```rust
task_repo.save_task_messages(
    task_id.as_str(),
    &user_message,
    &agent_message,
    Some(user_id),
    session_id,
    trace_id,
).await?;
```

### After (Recommended)

```rust
let message_service = MessageService::new(db_pool, logger);
message_service.persist_messages(
    &task_id,
    &context_id,
    vec![user_message, agent_message],
    Some(user_id),
    session_id,
    trace_id,
).await?;
```

---

## Troubleshooting

### "UNIQUE constraint failed: task_messages.task_uuid, task_messages.sequence_number"

**Cause**: Hardcoded sequence numbers or missing transaction

**Solution**: Ensure you're using `MessageService::persist_messages()` or calculating sequences within transactions

### "Message not appearing in UI"

**Cause**: Event not broadcasted

**Solution**: Ensure `EventService::broadcast_message_received()` is called after persistence

### "Messages out of order"

**Cause**: Race condition in sequence calculation

**Solution**: Verify sequence calculation happens within same transaction as persistence

---

## See Also

- [ARCHITECTURE.md](./ARCHITECTURE.md) - Overall architecture
- [../../../plan/a2a/refactor.md](../../../plan/a2a/refactor.md) - Refactoring documentation

**Last Updated**: 2025-10-17
