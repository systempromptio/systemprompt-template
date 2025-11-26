# A2A Module Refactoring Plan

## Current Issues

### 1. File Organization Problems
- **agent.rs** (285 lines): Mixes core types, builder patterns, and extensions
- **protocol.rs** (394 lines): Combines requests, responses, errors, events, and push notification configs
- Inconsistent file sizes indicating poor responsibility distribution
- Mixed abstraction levels within single files

### 2. Naming Inconsistencies
- Redundant prefixes (`A2aRequest`, `A2aResponse` vs `Task`, `Message`)
- Inconsistent casing and conventions
- Some types have unclear semantic meaning

### 3. Architectural Issues
- Builder patterns mixed with domain models
- Circular dependencies between modules
- Commented/unused code in protocol.rs
- Mixed error handling approaches

## Proposed Structure

```
a2a/
├── mod.rs                    # Clean public API exports
├── core/                     # Core domain entities
│   ├── mod.rs
│   ├── task.rs              # Task, TaskStatus, TaskState
│   ├── message.rs           # Message, MessageRole, Part types
│   ├── artifact.rs          # Artifact
│   └── agent.rs             # AgentCard core type only
├── agent/                    # Agent-related functionality
│   ├── mod.rs
│   ├── card.rs              # AgentCard core definition
│   ├── capabilities.rs      # AgentCapabilities, AgentExtension
│   ├── skills.rs            # AgentSkill
│   ├── provider.rs          # AgentProvider, AgentInterface
│   ├── signature.rs         # AgentCardSignature
│   └── builder.rs           # AgentCardBuilder pattern
├── protocol/                 # Protocol layer
│   ├── mod.rs
│   ├── requests.rs          # Request parameter types
│   ├── responses.rs         # Response types
│   ├── jsonrpc.rs           # JSON-RPC wrapper types
│   └── parsing.rs           # Request parsing logic
├── auth/                     # Authentication
│   ├── mod.rs
│   ├── schemes.rs           # SecurityScheme, OAuth2Flows
│   └── types.rs             # Supporting auth types
├── transport/
│   ├── mod.rs
│   └── protocol.rs          # TransportProtocol enum
├── events/                   # Event types
│   ├── mod.rs
│   ├── task_events.rs       # Task-related events
│   └── notification.rs      # Push notification events
├── errors/                   # Error handling
│   ├── mod.rs
│   ├── protocol_errors.rs   # Protocol-level errors
│   ├── task_errors.rs       # Task-specific errors
│   └── parsing_errors.rs    # Request parsing errors
└── config/                   # Configuration types
    ├── mod.rs
    ├── message_config.rs     # MessageSendConfiguration
    └── notification_config.rs # Push notification configs
```

## Detailed Refactoring Plan

### Phase 1: Create New Module Structure

#### 1.1 Core Domain Models (`core/`)
- **task.rs**: Extract `Task`, `TaskStatus`, `TaskState` from current task.rs
- **message.rs**: Extract `Message`, `MessageRole`, `Part`, `TextPart`, `DataPart`, `FilePart`, `FileWithBytes` from current message.rs
- **artifact.rs**: Keep current artifact.rs as-is, move to core/
- **agent.rs**: Extract only core `AgentCard` struct (no builder, no extensions)

#### 1.2 Agent Functionality (`agent/`)
- **card.rs**: Core AgentCard definition and basic constructors
- **capabilities.rs**: `AgentCapabilities`, `AgentExtension` and related logic
- **skills.rs**: `AgentSkill` and MCP server integration
- **provider.rs**: `AgentProvider`, `AgentInterface`
- **signature.rs**: `AgentCardSignature`
- **builder.rs**: `AgentCardBuilder` pattern implementation

#### 1.3 Protocol Layer (`protocol/`)
- **requests.rs**: All `*Params` types and request parameter structures
- **responses.rs**: All response types and `A2aResponse` enum
- **jsonrpc.rs**: Clean JSON-RPC types (`Request`, `JsonRpcResponse`, `JsonRpcError`, `RequestId`)
- **parsing.rs**: `A2aJsonRpcRequest` and parsing logic (`A2aRequestParams`, `A2aParseError`)

#### 1.4 Authentication (`auth/`)
- **schemes.rs**: `SecurityScheme`, `OAuth2Flows`, `OAuth2Flow`, `ApiKeyLocation`
- **types.rs**: `AgentAuthentication` and supporting types

#### 1.5 Transport (`transport/`)
- **protocol.rs**: `TransportProtocol` enum with clean implementations

#### 1.6 Events (`events/`)
- **task_events.rs**: `TaskStatusUpdateEvent`, `TaskArtifactUpdateEvent`
- **notification.rs**: Push notification event types

#### 1.7 Errors (`errors/`)
- **protocol_errors.rs**: `UnsupportedOperationError`, `PushNotificationNotSupportedError`
- **task_errors.rs**: `TaskNotFoundError`, `TaskNotCancelableError`
- **parsing_errors.rs**: `A2aParseError` and request parsing errors

#### 1.8 Configuration (`config/`)
- **message_config.rs**: `MessageSendConfiguration`
- **notification_config.rs**: All push notification config types

### Phase 2: Clean Implementation Standards

#### 2.1 Naming Conventions
- Remove `A2a` prefixes from internal types
- Use consistent `PascalCase` for types
- Use semantic names that reflect domain concepts
- Consistent field naming (camelCase in serde, snake_case in Rust)

#### 2.2 Import Organization
- Each module declares only necessary dependencies
- No circular imports between core modules
- Clear dependency hierarchy: `core` → `agent` → `protocol` → `errors`
- Use absolute imports from crate root

#### 2.3 Code Quality Standards
- No inline comments (code should be self-documenting)
- Consistent error handling patterns
- Proper use of `Option` and `Result` types
- Clean serde annotations
- No unused or commented code

### Phase 3: Module Interface Design

#### 3.1 Public API (`mod.rs`)
```rust
// Core domain types
pub use core::{Task, TaskStatus, TaskState, Message, MessageRole, Artifact};

// Agent types
pub use agent::{AgentCard, AgentCapabilities, AgentSkill, AgentProvider, AgentInterface, AgentExtension, AgentCardBuilder};

// Protocol types
pub use protocol::{JsonRpcResponse, RequestId};

// Auth types
pub use auth::{SecurityScheme, OAuth2Flows, ApiKeyLocation};

// Transport
pub use transport::TransportProtocol;

// Common error types
pub use errors::{ProtocolError, TaskError, ParseError};
```

#### 3.2 Internal Module Dependencies
- `core/` → No internal dependencies
- `agent/` → Depends on `core/`, `auth/`, `transport/`
- `protocol/` → Depends on `core/`, `agent/`, `auth/`
- `errors/` → Depends on `core/`
- `events/` → Depends on `core/`
- `config/` → Depends on `auth/`

### Phase 4: Implementation Guidelines

#### 4.1 Type Definitions
- Use `#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]` consistently
- Apply `#[serde(rename_all = "camelCase")]` where needed
- Use `Option<T>` with `#[serde(skip_serializing_if = "Option::is_none")]`
- Provide sensible `Default` implementations where appropriate

#### 4.2 Constructor Patterns
- Provide `new()` constructors for required fields only
- Use builder pattern for complex types with many optional fields
- Separate builders into dedicated modules
- Implement semantic constructors (e.g., `from_mcp_server()`)

#### 4.3 Error Handling
- Use `thiserror` for error types with clear messages
- Provide context in error messages
- Use `Result<T, E>` consistently for fallible operations
- Group related errors in dedicated modules

#### 4.4 Documentation
- Use `//!` module-level documentation explaining purpose
- Use `///` for public API documentation
- Focus on semantic meaning rather than implementation details
- Reference A2A specification sections where applicable

### Phase 5: Migration Strategy

1. **Create new module structure** while keeping existing files
2. **Implement one module at a time** starting with `core/`
3. **Update imports progressively** to use new structure
4. **Remove old files** once migration is complete
5. **Update tests** to use new module paths
6. **Run cargo check** after each module to ensure compilation

### Expected Benefits

1. **Reduced Cognitive Load**: Each file has single, clear responsibility
2. **Improved Maintainability**: Changes are localized to relevant modules
3. **Better Testability**: Focused modules are easier to test in isolation
4. **Enhanced Readability**: Clear separation of concerns and consistent naming
5. **Easier Extension**: New features can be added to appropriate modules
6. **Better Documentation**: Module organization reflects domain concepts

### Compilation Verification

After each phase:
```bash
cargo check --package agent
cargo test --package agent
cargo clippy --package agent
```

All changes must maintain backward compatibility of the public API exposed through `mod.rs`.