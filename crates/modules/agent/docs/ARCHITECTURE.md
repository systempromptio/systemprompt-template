# Agent Module Architecture

**Date**: 2025-10-17
**Status**: Current Architecture (Post-Refactoring)

---

## Overview

The Agent module implements the Agent-to-Agent (A2A) protocol specification, providing a complete framework for:
- **Agent Registration & Discovery** - Config-driven agent registry
- **Message Processing** - Streaming and non-streaming message handling
- **Tool Integration** - MCP (Model Context Protocol) tool execution
- **Artifact Management** - Creation, persistence, and broadcasting of artifacts
- **Event Broadcasting** - Real-time SSE notifications for UI updates

---

## Service Boundaries

### MessageService
**Location**: `crates/modules/agent/src/services/message_service.rs`

**Responsibility**: Single source of truth for all message persistence operations.

**Key Features**:
- ✅ **Transaction-safe sequence calculation** - No race conditions
- ✅ **Atomic operations** - All sequence numbers calculated within transactions
- ✅ **Logging** - All operations logged with structured metadata

**Usage Example**:
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

### EventService
**Location**: `crates/modules/agent/src/services/event_service.rs`

**Responsibility**: Centralized event broadcasting for all artifact/message/task events.

**Key Features**:
- ✅ **Non-blocking** - Uses `tokio::spawn()` internally
- ✅ **Error handling** - Catches and logs errors without panicking
- ✅ **Fire-and-forget** - Database persistence is source of truth

### ArtifactPublishingService  
**Location**: `crates/modules/agent/src/services/artifact_publishing_service.rs`

**Responsibility**: Publishes artifacts from A2A agents or direct MCP tool calls.

**Flow**:
```
publish_from_mcp()
├─> MessageService::create_tool_execution_message() (synthetic message)
├─> ArtifactRepository::create_artifact() (persist artifact)
└─> EventService::broadcast_artifact_created() (notify UI)
```

---

## Message Persistence Flow

### A2A Protocol Messages

```
Client → Agent Request
  ↓
MessageProcessor::handle_message()
  ↓
TaskRepository::create_task_simple()
  ↓
AI Processing + Tool Execution
  ↓
TaskRepository::update_task_and_save_messages() [ATOMIC]
  ├─> UPDATE agent_tasks (status = completed)
  ├─> persist_message (user, seq=0)
  └─> persist_message (agent, seq=1)
  ↓
ArtifactPublishingService::publish_from_a2a()
```

### Direct MCP Tool Calls

```
Client → MCP Tool Call
  ↓
ensure_task_exists()
  ↓
ArtifactPublishingService::publish_from_mcp()
  ├─> MessageService::create_tool_execution_message() [ATOMIC]
  ├─> ArtifactRepository::create_artifact()
  └─> EventService::broadcast_artifact_created()
```

---

## Sequence Number Calculation

**CRITICAL**: All sequence numbers calculated dynamically within transactions.

```rust
// ✅ CORRECT - Transaction-safe
let mut tx = db_pool.begin_transaction().await?;
let seq = TaskRepository::get_next_sequence_number_in_tx(&mut *tx, task_id).await?;
task_repo.persist_message_with_tx(&mut *tx, message, ..., seq, ...).await?;
tx.commit().await?;

// ❌ WRONG - Race condition
let seq = 0;  // Hardcoded!
```

---

## See Also

- [MESSAGE-PERSISTENCE.md](./MESSAGE-PERSISTENCE.md) - Message persistence patterns
- [../../../plan/a2a/refactor.md](../../../plan/a2a/refactor.md) - Refactoring plan

**Last Updated**: 2025-10-17
