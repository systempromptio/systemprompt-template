# A2A Client Service

Client implementation for communicating with A2A-compatible agents via JSON-RPC 2.0 protocol.

## Architecture

```
a2a_client/
├── client.rs       - Main A2aClient with send/stream/query methods
├── protocol.rs     - JSON-RPC 2.0 request/response handling
├── transport.rs    - HTTP transport implementation
├── streaming.rs    - SSE streaming support
├── error.rs        - Error types and conversions
└── mod.rs          - Module exports
```

## API

### A2aClient

```rust
pub struct A2aClient {
    protocol: ProtocolHandler,
    transport: Arc<dyn Transport>,
    config: ClientConfig,
    log_service: Option<LogService>,
}
```

**Methods:**

- `new(config: ClientConfig) -> ClientResult<Self>` - Create new client
- `with_logger(log_service: LogService) -> Self` - Add logging
- `send_message(message: Message) -> ClientResult<Task>` - Send blocking message
- `send_streaming_message(message: Message) -> ClientResult<SseStream>` - Stream message responses
- `get_task(task_id: &str) -> ClientResult<Task>` - Query task status
- `cancel_task(task_id: &str) -> ClientResult<Task>` - Cancel running task
- `get_authenticated_extended_card() -> ClientResult<AgentCard>` - Get agent card (authenticated)
- `fetch_agent_card() -> ClientResult<AgentCard>` - Fetch .well-known/agent-card.json

### ClientConfig

```rust
pub struct ClientConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub auth_token: Option<String>,
}
```

## Components

### protocol.rs
Handles JSON-RPC 2.0 protocol:
- `ProtocolHandler::create_request()` - Create JSON-RPC request with auto-incremented ID
- `ProtocolHandler::parse_response()` - Parse JSON-RPC response, extract result or error
- Request types: `MessageSendRequest`, `TaskQueryRequest`, `CancelTaskRequest`

### transport.rs
HTTP transport layer:
- `HttpTransport` - Implements `Transport` trait
- Configurable timeout and auth token
- POST requests with JSON bodies

### streaming.rs
SSE event stream processing:
- `SseStream` - Implements `Stream` trait
- `StreamEvent` - Content, Tool, Complete, Error variants
- Parses `data:` lines from SSE stream

### error.rs
Error types using `thiserror`:
- `ClientError::Network` - HTTP/network errors
- `ClientError::Json` - Serialization errors
- `ClientError::Protocol` - JSON-RPC protocol errors
- `ClientError::Agent` - Agent-reported errors
- `ClientError::Stream` - Streaming errors

## Usage

```rust
use crate::services::a2a_client::{A2aClient, ClientConfig};
use crate::models::a2a::message::Message;

let config = ClientConfig {
    base_url: "http://localhost:8080".to_string(),
    timeout: Duration::from_secs(30),
    auth_token: Some("token".to_string()),
};

let client = A2aClient::new(config)?;

let message = Message {
    role: "user".to_string(),
    content: "Hello".to_string(),
};

let task = client.send_message(message).await?;
```

### Streaming

```rust
use futures::StreamExt;

let mut stream = client.send_streaming_message(message).await?;

while let Some(event) = stream.next().await {
    match event? {
        StreamEvent::Content(text) => print!("{}", text),
        StreamEvent::Complete => break,
        _ => {}
    }
}
```

## Dependencies

**Internal:**
- `crate::models::a2a` - A2A protocol types (Message, Task, AgentCard)
- `systemprompt_core_logging::LogService` - Optional logging

**External:**
- `reqwest` - HTTP client
- `tokio` - Async runtime
- `serde` / `serde_json` - JSON serialization
- `futures` - Stream utilities
- `async_trait` - Async traits
- `thiserror` - Error types
