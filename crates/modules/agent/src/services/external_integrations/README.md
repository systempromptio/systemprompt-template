# External Integrations

Unified service layer for third-party integrations including MCP servers and webhook endpoints. Provides reliable, secure, and maintainable integration points for A2A agents.

## Architecture Overview

The external integrations module follows a clean, modular architecture with clear separation of concerns:

```
external_integrations/
├── README.md                    # This file
├── mod.rs                       # Module exports
│
├── mcp/                         # MCP server integration
│   ├── README.md               # MCP-specific documentation
│   ├── orchestration/          # Skill loading coordination
│   │   └── loader.rs           # McpSkillLoader
│   ├── client/                 # MCP protocol client
│   │   └── adapter.rs          # McpClientAdapter
│   ├── converter/              # Data transformation
│   │   └── tool_to_skill.rs    # Tool → Skill conversion
│   ├── service/                # Service state management
│   │   └── state_manager.rs    # ServiceStateManager
│   └── models/                 # Domain models
│       └── mod.rs
│
└── webhook/                     # Webhook delivery
    ├── README.md               # Webhook-specific documentation
    ├── mod.rs
    └── service.rs              # WebhookService
```

## Core Principles

### 1. Single Responsibility
Each component has one clear purpose:
- **orchestration/** - Coordinates multi-step processes
- **client/** - Handles protocol communication
- **converter/** - Transforms data between formats
- **service/** - Manages service state and configuration

### 2. Repository Pattern Compliance
All database operations go through repositories:
- ✅ SQL queries in `repository/service_repository.rs`
- ❌ No inline SQL in service layer
- Follows CLAUDE.md architectural requirements

### 3. Clear Naming
Folder names immediately convey purpose:
- `orchestration/` not `core/` - Explains what it orchestrates
- `converter/` not `utils/` - Explains data transformation role
- `client/` not `connection/` - Standard protocol client terminology

### 4. Comprehensive Documentation
Every major component has README explaining:
- Purpose and responsibilities
- Architecture and data flow
- Usage examples
- Integration points

## Integration Types

### MCP (Model Context Protocol)

**Purpose**: Dynamic skill loading from MCP servers for agent capability extension

**Key Features**:
- On-demand skill loading (no persistence)
- Timeout protection (5s default)
- Service state tracking
- Tool → Skill transformation

**Use Cases**:
- Database tool integration
- File system access
- API integrations
- Custom tool execution

**See**: [mcp/README.md](./mcp/README.md)

### Webhook

**Purpose**: Outbound event notifications to external HTTP endpoints

**Key Features**:
- HMAC-SHA256 signature validation
- Automatic retries with exponential backoff
- Endpoint management (CRUD)
- Delivery tracking

**Use Cases**:
- Task completion notifications
- Agent lifecycle events
- Error alerting
- External system synchronization

**See**: [webhook/README.md](./webhook/README.md)

## Data Flow

### MCP Skill Loading Flow

```
1. Agent metadata defines MCP servers
   agent_metadata.mcp_servers: ["database-tools", "file-manager"]
   ↓
2. ServiceStateManager queries service state
   repository/service_repository.rs
   ↓
3. McpClientAdapter connects to running servers
   localhost:5001, localhost:5002
   ↓
4. SkillConverter transforms tools to skills
   MCP Tool → A2A AgentSkill
   ↓
5. McpSkillLoader returns combined skills
   All skills from all assigned servers
```

### Webhook Delivery Flow

```
1. Event triggered (task.completed, agent.started)
   ↓
2. WebhookService looks up registered endpoints
   Filter by event type
   ↓
3. Generate HMAC signature
   HMAC-SHA256(payload, secret)
   ↓
4. HTTP POST with signature header
   X-Signature: sha256=abc123...
   ↓
5. Retry on failure (max 3 attempts)
   Exponential backoff: 1s, 2s, 4s
```

## Usage Examples

### MCP Skill Loading

```rust
use crate::services::external_integrations::mcp::McpSkillLoader;

let loader = McpSkillLoader::new(db_pool);

// Load all skills for an agent
let skills = loader.load_agent_skills("echo-agent").await?;

// Load skills from specific server
let server_skills = loader.load_server_skills("database-tools").await?;

// For task processor skill matching
let all_skills = loader.get_all_agent_skills_map().await?;
```

### Webhook Event Delivery

```rust
use crate::services::external_integrations::webhook::WebhookService;

let service = WebhookService::new();

// Register endpoint
let endpoint = WebhookEndpoint {
    id: String::new(),
    url: "https://api.example.com/webhooks".to_string(),
    secret: "shared-secret".to_string(),
    active: true,
    events: vec!["task.completed".to_string()],
    headers: HashMap::new(),
};

let endpoint_id = service.register_endpoint(endpoint).await?;

// Send event
let request = WebhookRequest {
    event_type: "task.completed".to_string(),
    payload: json!({"task_id": "task-123", "status": "completed"}),
    timestamp: chrono::Utc::now().timestamp(),
    headers: HashMap::new(),
};

let response = service.handle_webhook(&endpoint_id, request).await?;
```

## Repository Pattern

### Before (Architectural Violation)

```rust
// ❌ WRONG - Inline SQL in service layer
pub async fn get_mcp_service(&self, name: &str) -> Result<Option<McpServiceState>> {
    let row = sqlx::query(
        "SELECT name, host, port, status FROM services WHERE protocol = 'mcp' AND name = ?"
    )
    .bind(name)
    .fetch_optional(self.db_pool.pool())
    .await?;
    // ...
}
```

### After (Compliant)

```rust
// ✅ CORRECT - Repository handles database access
// repository/service_repository.rs
impl ServiceRepository {
    const GET_MCP_SERVICE: &'static str = "SELECT name, host, port...";

    pub async fn get_mcp_service(&self, name: &str) -> Result<Option<McpServiceState>> {
        // SQL query here
    }
}

// service/state_manager.rs
impl ServiceStateManager {
    pub async fn get_mcp_service(&self, name: &str) -> Result<Option<McpServiceState>> {
        self.service_repo.get_mcp_service(name).await
    }
}
```

## Integration Points

### A2A Protocol

External integrations enable A2A protocol features:

**Task Processing**:
- MCP skills → Capability matching → Task routing
- Webhook notifications → External task orchestration

**Agent Discovery**:
- MCP servers → Dynamic skill loading → Agent capabilities
- Service state → Health monitoring → Agent availability

**Event Streaming**:
- Webhook delivery → External event processing
- HMAC signatures → Secure event validation

### CLI Commands

```bash
# MCP operations
systemprompt-a2a mcp list echo-agent           # List agent's MCP skills
systemprompt-a2a mcp servers                   # List running MCP servers

# Webhook operations
systemprompt-a2a webhook register \
    --url "https://api.example.com/hooks" \
    --events "task.completed,task.failed" \
    --secret "my-secret"

systemprompt-a2a webhook list                  # List registered webhooks
systemprompt-a2a webhook test <endpoint-id>    # Test webhook delivery
```

### REST API

```bash
# Get agent skills (includes MCP skills)
curl http://localhost:8080/api/v1/agents/echo-agent/skills

# Trigger webhook (via event system)
curl -X POST http://localhost:8080/api/v1/events \
    -d '{"event_type": "task.completed", "task_id": "task-123"}'
```

## Error Handling

### MCP Errors

| Error | Handling | Impact |
|-------|----------|--------|
| Server not found | Skip with warning | Continue with other servers |
| Server not running | Skip with warning | Agent has reduced capabilities |
| Connection timeout (5s) | Skip with warning | Prevents hanging |
| No tools returned | Empty skill list | Valid state (server may be empty) |

### Webhook Errors

| Error | Handling | Impact |
|-------|----------|--------|
| Endpoint not found | Return 404 | Event delivery failed |
| Endpoint inactive | Return 404 | Event ignored |
| Network timeout | Retry (max 3) | Delayed delivery |
| HTTP 5xx | Retry (max 3) | Eventual delivery |
| HTTP 4xx | No retry | Permanent failure |

## Performance Characteristics

### MCP Skill Loading

- **Timeout**: 5 seconds per server (prevents hanging)
- **Concurrency**: Serial loading (controlled resource usage)
- **Caching**: No caching (always fresh from MCP servers)
- **Failure Isolation**: Failed servers don't block others

### Webhook Delivery

- **Timeout**: 30 seconds per attempt
- **Retries**: 3 attempts with exponential backoff
- **Total Time**: ~1 minute maximum per webhook
- **Async**: Non-blocking delivery (doesn't block task processing)

## Security

### MCP

- **Service Authentication**: Port 5002 requires admin tokens
- **Network Isolation**: MCP servers run on localhost only
- **Timeout Protection**: Prevents resource exhaustion
- **Error Sanitization**: No sensitive data in error messages

### Webhook

- **HMAC-SHA256**: Cryptographic signature validation
- **Secret Management**: Per-endpoint secrets
- **HTTPS Only**: Best practice (enforced in production)
- **Signature Validation**: Constant-time comparison

## Testing

### Unit Tests

```bash
# Test MCP skill loading
cargo test --lib external_integrations::mcp::orchestration::loader

# Test webhook delivery
cargo test --lib external_integrations::webhook::service
```

### Integration Tests

```bash
# Test with running MCP servers
./tests/integration/mcp/test_skill_loading.sh

# Test webhook delivery
./tests/integration/webhook/test_delivery.sh
```

## Future Enhancements

### Planned Features

1. **OAuth Integration**: Third-party OAuth provider support
2. **Webhook Persistence**: Database-backed webhook configurations
3. **Delivery Analytics**: Track webhook success/failure rates
4. **MCP Caching**: Optional skill caching with TTL
5. **Circuit Breaker**: Automatic server health management
6. **Dead Letter Queue**: Failed webhook retry queue

### Performance Improvements

- Concurrent MCP server loading
- Connection pooling for webhooks
- Batch webhook delivery
- Tool result caching

## Contributing

When adding new integrations:

1. **Create dedicated folder** (e.g., `oauth/`, `grpc/`)
2. **Add README.md** explaining purpose and usage
3. **Follow repository pattern** (no inline SQL)
4. **Use clear naming** (orchestration, client, converter patterns)
5. **Add comprehensive tests**
6. **Update this README** with new integration type

## Migration Notes

### Recent Changes

**v1.0 → v2.0 (Current)**:
- ✅ Moved `skill_loader.rs` → `orchestration/loader.rs` (no more loose files)
- ✅ Extracted SQL to `repository/service_repository.rs` (repository pattern compliance)
- ✅ Removed `types.rs` (eliminated redundant re-export)
- ✅ Added comprehensive documentation (README in each module)

**Import Changes**:
```rust
// Old
use crate::services::external_integrations::mcp::skill_loader::McpSkillLoader;

// New
use crate::services::external_integrations::mcp::McpSkillLoader;  // Re-exported from mod.rs
```

All exports remain the same, only internal organization changed.
