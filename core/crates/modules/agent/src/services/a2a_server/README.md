# A2A Server Service

## Overview

The A2A Server implements a fully compliant A2A (Agent-to-Agent) protocol server that exposes agent capabilities via HTTP/JSON-RPC. It handles incoming requests, manages authentication, processes messages through the AI service, and streams responses via Server-Sent Events (SSE).

## Architecture

```
a2a_server/
├── server.rs                  # HTTP server setup and lifecycle management
│
├── handlers/                  # Request handling layer
│   ├── mod.rs
│   ├── state.rs              # Shared handler state
│   ├── request.rs            # Main A2A JSON-RPC request handler
│   └── card.rs               # Agent card discovery endpoint
│
├── processing/                # Business logic layer
│   ├── mod.rs
│   ├── message.rs            # Message processing orchestration
│   ├── ai_executor.rs        # AI service integration (tools, agentic loops)
│   └── artifact.rs           # Transform tool results into A2A artifacts
│
├── streaming/                 # Server-Sent Events layer
│   ├── mod.rs
│   └── messages.rs           # Real-time message streaming via SSE
│
├── auth/                      # Authentication & authorization
│   ├── mod.rs
│   ├── types.rs              # OAuth state and user types
│   ├── validation.rs         # JWT token validation
│   └── middleware.rs         # Axum OAuth middleware
│
├── config/                    # Configuration management
│   ├── mod.rs
│   ├── types.rs              # Config type aliases
│   └── agent.rs              # Agent config database operations
│
├── builders/                  # Data construction utilities
│   ├── mod.rs
│   └── task.rs               # Task builder pattern + helpers
│
└── errors/                    # Error handling
    ├── mod.rs
    └── jsonrpc.rs            # JSON-RPC 2.0 error builder

```

## Module Responsibilities

### 1. Server (`server.rs`)
- HTTP server initialization and lifecycle
- Router configuration
- Axum middleware setup
- Graceful shutdown handling
- CORS configuration

**Key API:**
```rust
let server = Server::new(db_pool, app_context, agent_id, port).await?;
server.run().await?;
```

### 2. Handlers (`handlers/`)
**State** (`state.rs`):
- Shared state across all handlers
- Contains: db_pool, config, oauth_state, app_context, ai_service, log

**Request** (`request.rs`):
- Main entry point for A2A JSON-RPC requests
- Request parsing and validation
- OAuth enforcement (conditional)
- Routes to streaming or non-streaming paths
- Methods supported: `message/send`, `message/stream`, `tasks/get`, `tasks/cancel`

**Card** (`card.rs`):
- Serves agent card at `/.well-known/agent-card.json`
- Returns agent capabilities and metadata

### 3. Processing (`processing/`)
**Message** (`message.rs`):
- Consolidates message handling (streaming and non-streaming)
- Orchestrates AI processing pipeline
- Persists tasks to database
- Injects analytics context into messages

**AI Executor** (`ai_executor.rs`):
- `process_with_agentic_tools()` - Multi-turn tool execution with AI control
- `process_with_tools()` - Single-turn tool execution
- `process_without_tools()` - Direct text generation streaming

**Artifact** (`artifact.rs`):
- Transforms tool results into A2A artifacts
- Handles MCP output schemas
- Creates error artifacts for failed tool calls

### 4. Streaming (`streaming/`)
**Messages** (`messages.rs`):
- Real-time SSE for message processing
- Streams: `task.started`, `message`, `tool_call`, `tool_result`, `complete`
- Uses `MessageProcessor` for AI execution
- Persists completed tasks to database

### 5. Auth (`auth/`)
**Types** (`types.rs`):
- `AgentOAuthState` - OAuth configuration and database
- `AgentAuthenticatedUser` - Authenticated user context

**Validation** (`validation.rs`):
- JWT token validation
- User existence and active status verification
- A2A permission checking
- Token generation utilities

**Middleware** (`middleware.rs`):
- Axum middleware for OAuth enforcement
- Extracts and validates Bearer tokens
- Injects user context into request extensions

### 6. Config (`config/`)
**Types** (`types.rs`):
- Type aliases for agent config

**Agent** (`agent.rs`):
- Load/save agent config from database
- Update agent card and metadata

### 7. Builders (`builders/`)
**Task** (`task.rs`):
- `TaskBuilder` - Fluent builder pattern for tasks
- Helper functions:
  - `build_completed_task()` - Success with artifacts
  - `build_canceled_task()` - Canceled task
  - `build_mock_task()` - Placeholder task
  - `build_multiturn_task()` - Multi-turn with tool history

### 8. Errors (`errors/`)
**JSON-RPC** (`jsonrpc.rs`):
- Builder pattern for JSON-RPC 2.0 errors
- Integrated logging
- Standard error codes: `-32600` (Invalid Request), `-32601` (Method Not Found), etc.
- Helper functions: `unauthorized_response()`, `forbidden_response()`

## Request Flow

### Non-Streaming Message
```
Client → handle_agent_request()
  ↓
  Parse JSON-RPC request
  ↓
  OAuth validation (if required)
  ↓
  handle_non_streaming_request()
  ↓
  MessageProcessor::handle_message()
  ↓
  process_message_stream() (collect all events)
  ↓
  AI Executor (with/without tools)
  ↓
  Build artifacts
  ↓
  Persist task to DB
  ↓
  Return JSON-RPC response
```

### Streaming Message
```
Client → handle_agent_request()
  ↓
  Parse JSON-RPC request
  ↓
  OAuth validation (if required)
  ↓
  handle_streaming_request()
  ↓
  create_sse_stream()
  ↓
  MessageProcessor::process_message_stream()
  ↓
  Stream events: Text, ToolCallStarted, ToolResult, Complete
  ↓
  Client receives SSE events
```

## Configuration

Server requires:
- **Database Pool**: SQLite pool for agent/task storage
- **AppContext**: Shared application context with logging
- **Agent ID**: UUID of the agent to serve
- **Port**: HTTP port to bind (e.g., 9000-9010)

Optional:
- **OAuth**: Enable via `AgentOAuthConfig`
- **Web Client**: Serve static files from `web/dist/`

## Authentication

OAuth2 Bearer token authentication (optional):
1. Extract token from `Authorization: Bearer <token>` header
2. Validate JWT signature and expiration
3. Verify `aud` contains `"a2a"`
4. Check user exists and is active
5. Verify A2A permissions (`admin` role or `a2a` permission)

## Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/` | POST | Main A2A JSON-RPC endpoint |
| `/.well-known/agent-card.json` | GET | Agent card discovery |
| `/api/a2a/card` | GET | Alternative agent card endpoint |

## A2A Methods Supported

| Method | Parameters | Response | Streaming |
|--------|-----------|----------|-----------|
| `message/send` | `{message}` | Task | No |
| `message/stream` | `{message}` | Task (via SSE) | Yes |
| `tasks/get` | `{id}` | Task | No |
| `tasks/cancel` | `{id}` | Task | No |

## Dependencies

**Internal:**
- `crate::repository` - Agent and task repositories
- `crate::models::a2a` - A2A protocol types
- `crate::services::mcp` - MCP artifact transformation
- `systemprompt_core_ai` - AI service and tool execution
- `systemprompt_core_logging` - Structured logging
- `systemprompt_core_oauth` - JWT validation
- `systemprompt_core_system` - App context and config

**External:**
- `axum` - Web framework
- `tokio` - Async runtime
- `serde_json` - JSON serialization
- `tower_http` - CORS middleware
- `futures` - Stream utilities

## Example Usage

```rust
use systemprompt_core_system::AppContext;
use crate::services::a2a_server::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app_context = AppContext::new().await?;
    let db_pool = app_context.db_pool().clone();
    let agent_id = "550e8400-e29b-41d4-a716-446655440000";
    let port = 9000;

    let server = Server::new(db_pool, app_context, Some(agent_id.to_string()), port).await?;
    server.run().await?;

    Ok(())
}
```

## Testing

```bash
# Start server
just a2a start --agent-id <UUID>

# Test agent card
curl http://localhost:9000/.well-known/agent-card.json

# Send message (requires admin token)
ADMIN_TOKEN=$(just admin-token)
curl -X POST http://localhost:9000/ \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "message/send",
    "params": {
      "message": {
        "role": "user",
        "parts": [{"kind": "text", "text": "Hello"}],
        "messageId": "msg_1",
        "kind": "message"
      }
    },
    "id": 1
  }'
```

## Performance Characteristics

- **Concurrency**: Handles multiple requests via Tokio runtime
- **Streaming**: SSE keeps connections alive for real-time updates
- **Database**: Connection pooling via SQLx
- **Tool Execution**: Parallel tool calls when possible
- **Memory**: Bounded channel sizes (100 items) for streaming

## Security Considerations

1. **Authentication**: OAuth2 Bearer tokens (optional but recommended)
2. **Authorization**: Role-based access control (RBAC)
3. **Input Validation**: JSON-RPC 2.0 spec compliance
4. **Rate Limiting**: Not implemented (add via Axum middleware if needed)
5. **CORS**: Permissive by default (tighten for production)

## Error Handling

All errors follow JSON-RPC 2.0 spec:
- `-32700` Parse error (invalid JSON)
- `-32600` Invalid Request
- `-32601` Method not found
- `-32602` Invalid params
- `-32603` Internal error

Custom errors:
- `401 Unauthorized` - Missing/invalid token
- `403 Forbidden` - Insufficient permissions
- `404 Not Found` - Agent card not found

## Logging

Module prefixes for log filtering:
- `a2a_server` - Server lifecycle
- `a2a_request` - Request handling
- `a2a_card` - Agent card requests
- `a2a_auth` - Authentication/authorization
- `a2a_oauth` - OAuth validation
- `message_processor` - Message processing
- `ai_executor` - AI execution
- `sse_messages` - SSE message streaming

Use `just log` during development to monitor all events.
