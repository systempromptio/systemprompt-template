# Webhook Service

Manages outbound webhook delivery for external event notifications with HMAC signature validation, retry logic, and delivery tracking.

## Architecture Overview

The webhook service enables agents to send event notifications to external HTTP endpoints with enterprise-grade reliability and security features.

### Key Features

- **HMAC-SHA256 Signatures**: Cryptographic signature validation for webhook authenticity
- **Automatic Retries**: Configurable retry logic with exponential backoff
- **Endpoint Management**: CRUD operations for webhook endpoints
- **Delivery Tracking**: Monitor delivery status and failures
- **Security**: Secret-based request signing and validation

## Components

### WebhookService (`service.rs`)

Main orchestrator that coordinates all webhook operations:

**Endpoint Management**:
- `register_endpoint(endpoint)` - Register new webhook endpoint
- `update_endpoint(endpoint)` - Update existing endpoint
- `get_endpoint(id)` - Retrieve endpoint configuration
- `list_endpoints()` - List all registered endpoints
- `remove_endpoint(id)` - Unregister webhook endpoint

**Event Delivery**:
- `handle_webhook(endpoint_id, request)` - Process incoming webhook event
- `deliver_webhook(url, payload, secret)` - Send HTTP POST with HMAC signature
- `test_endpoint(endpoint_id)` - Test endpoint connectivity

**Security**:
- `validate_signature(payload, signature, secret)` - HMAC-SHA256 validation
- `generate_signature(payload, secret)` - Create request signature

## Data Models

### WebhookEndpoint

```rust
pub struct WebhookEndpoint {
    pub id: String,              // Unique endpoint identifier
    pub url: String,             // Target HTTP URL
    pub secret: String,          // HMAC signing secret
    pub active: bool,            // Enable/disable endpoint
    pub events: Vec<String>,     // Event types to receive
    pub headers: HashMap<String, String>,  // Custom HTTP headers
}
```

### WebhookRequest

```rust
pub struct WebhookRequest {
    pub event_type: String,      // Event identifier
    pub payload: Value,          // JSON event data
    pub timestamp: i64,          // Event timestamp
    pub headers: HashMap<String, String>,  // Request headers
}
```

### WebhookResponse

```rust
pub struct WebhookResponse {
    pub status: u16,             // HTTP status code
    pub body: Option<String>,    // Response body
    pub delivered: bool,         // Delivery success flag
    pub error: Option<String>,   // Error message if failed
}
```

## Usage Examples

### Register Webhook Endpoint

```rust
use crate::services::external_integrations::webhook::WebhookService;

let service = WebhookService::new();

let endpoint = WebhookEndpoint {
    id: String::new(),  // Auto-generated
    url: "https://api.example.com/webhooks".to_string(),
    secret: "your-secret-key".to_string(),
    active: true,
    events: vec!["task.completed".to_string(), "agent.started".to_string()],
    headers: HashMap::new(),
};

let endpoint_id = service.register_endpoint(endpoint).await?;
```

### Send Webhook Event

```rust
let request = WebhookRequest {
    event_type: "task.completed".to_string(),
    payload: json!({
        "task_id": "task-123",
        "status": "completed",
        "result": "success"
    }),
    timestamp: chrono::Utc::now().timestamp(),
    headers: HashMap::new(),
};

let response = service.handle_webhook(&endpoint_id, request).await?;

if response.delivered {
    println!("Webhook delivered successfully: status {}", response.status);
} else {
    println!("Webhook delivery failed: {:?}", response.error);
}
```

### Validate Incoming Webhook (Verification)

```rust
// When receiving webhook callbacks from external services
let payload = r#"{"event": "data"}"#;
let signature = "sha256=abc123...";  // From X-Signature header
let secret = "shared-secret";

if service.validate_signature(payload, signature, secret) {
    println!("Valid webhook signature");
} else {
    println!("Invalid signature - potential tampering");
}
```

## Security Model

### HMAC Signature Flow

**Outbound Webhooks** (we send to external service):
1. Generate signature: `HMAC-SHA256(payload, secret)`
2. Include in `X-Signature` header: `sha256=<hex_digest>`
3. External service validates using shared secret

**Inbound Webhooks** (we receive from external service):
1. Extract signature from `X-Signature` header
2. Compute expected signature from payload
3. Compare signatures (constant-time comparison)
4. Reject if mismatch

### Example Signature Generation

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;
mac.update(payload.as_bytes());
let result = mac.finalize();
let signature = format!("sha256={}", hex::encode(result.into_bytes()));
```

## Retry Logic

**Retry Configuration**:
- Max retries: 3
- Backoff: Exponential (1s, 2s, 4s)
- Timeout: 30 seconds per attempt
- Total max time: ~1 minute

**Retry Triggers**:
- Network errors (connection timeout, DNS failure)
- HTTP 5xx errors (server errors)
- HTTP 429 (rate limiting)

**No Retry**:
- HTTP 4xx errors (client errors - bad request, unauthorized)
- HTTP 2xx success
- Endpoint marked inactive

## Error Handling

### Common Error Scenarios

| Error | Cause | Resolution |
|-------|-------|------------|
| `Endpoint not found` | Invalid endpoint_id | Verify endpoint exists |
| `Endpoint inactive` | Endpoint disabled | Enable endpoint or remove |
| `Invalid signature` | Wrong secret or tampered payload | Verify shared secret |
| `Connection timeout` | Network or DNS issues | Check URL accessibility |
| `HTTP 4xx` | Bad request/auth | Verify payload and credentials |
| `HTTP 5xx` | External service down | Automatic retry |

## Integration Points

### A2A Protocol Events

Webhooks can be triggered by A2A protocol events:
- `task.submitted` - New task received
- `task.working` - Task execution started
- `task.completed` - Task finished successfully
- `task.failed` - Task execution failed
- `agent.started` - Agent process started
- `agent.stopped` - Agent process stopped

### CLI Commands

```bash
# Register webhook
systemprompt-a2a webhook register \
    --url "https://api.example.com/hooks" \
    --events "task.completed,task.failed" \
    --secret "my-secret"

# List webhooks
systemprompt-a2a webhook list

# Test webhook
systemprompt-a2a webhook test <endpoint-id>

# Remove webhook
systemprompt-a2a webhook remove <endpoint-id>
```

## Performance Considerations

- **Non-blocking**: Webhook delivery runs asynchronously
- **Timeout Protection**: 30s timeout prevents hanging
- **Connection Pooling**: Reuses HTTP connections
- **Memory Efficient**: Streams large payloads
- **Failure Isolation**: Failed webhooks don't block task processing

## Best Practices

1. **Use HTTPS**: Always use HTTPS URLs for webhook endpoints
2. **Rotate Secrets**: Periodically rotate HMAC secrets
3. **Validate Events**: Only register for events you need
4. **Monitor Failures**: Track delivery failures and investigate
5. **Idempotency**: Design webhook handlers to be idempotent
6. **Timeouts**: Set reasonable timeouts on receiving side
7. **Logging**: Log all webhook deliveries for audit trails

## Future Enhancements

- Database persistence for webhook configurations
- Delivery history and analytics
- Custom retry policies per endpoint
- Batch webhook delivery
- Webhook event filtering/transformation
- Dead letter queue for failed deliveries
