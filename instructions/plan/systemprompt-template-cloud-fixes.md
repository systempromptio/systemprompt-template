# Cloud Deployment Architecture

This document outlines the complete checkout and provisioning flow across all repositories.

---

## Complete Checkout & Provisioning Flow

### Overview

When a user creates a cloud tenant, the flow involves:
1. **CLI** creates checkout session and opens browser
2. **Paddle** processes payment and sends webhook
3. **Management API** provisions tenant and streams events via SSE
4. **CLI** receives SSE events and displays progress

### Flow Diagram

```
┌─────────┐     ┌─────────┐     ┌─────────────┐     ┌──────────────┐     ┌─────────┐
│   CLI   │     │ Browser │     │   Paddle    │     │ Management   │     │   GCP   │
│         │     │         │     │             │     │    API       │     │         │
└────┬────┘     └────┬────┘     └──────┬──────┘     └──────┬───────┘     └────┬────┘
     │               │                 │                   │                  │
     │ 1. Create checkout session      │                   │                  │
     │─────────────────────────────────────────────────────>│                  │
     │               │                 │                   │                  │
     │<──────────────────────────────────────checkout_url──│                  │
     │               │                 │                   │                  │
     │ 2. Open browser with checkout_url                   │                  │
     │──────────────>│                 │                   │                  │
     │               │                 │                   │                  │
     │               │ 3. User completes payment           │                  │
     │               │────────────────>│                   │                  │
     │               │                 │                   │                  │
     │               │<───redirect to /checkout/complete───│                  │
     │               │                 │                   │                  │
     │               │ 4. Redirect to CLI localhost        │                  │
     │               │<────────────────────────────────────│                  │
     │               │  ?status=pending&checkout_session_id│                  │
     │               │                 │                   │                  │
     │<──────────────│                 │                   │                  │
     │ 5. Return waiting HTML          │                   │                  │
     │──────────────>│                 │                   │                  │
     │               │                 │                   │                  │
     │ 6. Subscribe to SSE             │                   │                  │
     │────────────────────────────GET /checkout/{id}/events│                  │
     │               │                 │                   │                  │
     │               │                 │ 7. Webhook: subscription.created     │
     │               │                 │──────────────────>│                  │
     │               │                 │                   │                  │
     │<──────────────SSE: SubscriptionCreated──────────────│                  │
     │               │                 │                   │                  │
     │               │                 │                   │ 8. Create tenant │
     │               │                 │                   │ in database      │
     │               │                 │                   │                  │
     │<──────────────SSE: TenantCreated────────────────────│                  │
     │               │                 │                   │                  │
     │               │                 │                   │ 9. Create DB     │
     │               │                 │                   │ and credentials  │
     │               │                 │                   │                  │
     │<──────────────SSE: DatabaseCreated──────────────────│                  │
     │               │                 │                   │                  │
     │               │                 │                   │ 10. Store secrets│
     │               │                 │                   │                  │
     │<──────────────SSE: SecretsStored────────────────────│                  │
     │               │                 │                   │                  │
     │               │                 │                   │ 11. Start VM     │
     │               │                 │                   │──────────────────>│
     │<──────────────SSE: VmProvisioningStarted────────────│                  │
     │               │                 │                   │                  │
     │<──────────────SSE: VmProvisioningProgress───────────│<─────────────────│
     │               │                 │                   │                  │
     │<──────────────SSE: VmProvisioned────────────────────│                  │
     │               │                 │                   │                  │
     │               │                 │                   │ 12. Set secrets  │
     │               │                 │                   │──────────────────>│
     │<──────────────SSE: SecretsConfigured────────────────│                  │
     │               │                 │                   │                  │
     │<──────────────SSE: TenantReady──────────────────────│                  │
     │               │                 │                   │                  │
     │ 13. Display success             │                   │                  │
     │               │                 │                   │                  │
```

---

## Provisioning Event Types

### Event Enum (Backend)

```rust
// File: crates/management-api/src/models/provisioning_events.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvisioningEventType {
    // Checkout flow events
    SubscriptionCreated,    // Paddle webhook received, starting provisioning
    TenantCreated,          // Tenant record created in management DB
    DatabaseCreated,        // Tenant database and user created with credentials
    SecretsStored,          // Credentials stored in tenant_secrets table

    // VM provisioning events
    VmProvisioningStarted,  // Cloud VM provisioning started
    VmProvisioningProgress, // Progress update during VM creation
    VmProvisioned,          // VM successfully created
    SecretsConfigured,      // Environment secrets configured on VM

    // Terminal events
    TenantReady,            // Provisioning complete, tenant is ready
    ProvisioningFailed,     // Error occurred (with message)
}
```

### Event Structs

```rust
// For checkout flows (has checkout_session_id for correlation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutEvent {
    pub checkout_session_id: String,
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub event_type: ProvisioningEventType,
    pub status: String,
    pub message: Option<String>,
    pub app_url: Option<String>,
}

// For retry-provision flows (no checkout_session_id)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningEvent {
    pub tenant_id: Uuid,
    pub event_type: ProvisioningEventType,
    pub status: String,
    pub message: Option<String>,
    pub app_url: Option<String>,
}
```

### Event Messages

| Event Type | Message |
|------------|---------|
| `SubscriptionCreated` | "Payment confirmed, creating tenant..." |
| `TenantCreated` | "Tenant '{name}' created" |
| `DatabaseCreated` | "Database and credentials created" |
| `SecretsStored` | "Credentials stored securely" |
| `VmProvisioningStarted` | "Starting cloud VM provisioning..." |
| `VmProvisioningProgress` | "{step}: {details}" |
| `VmProvisioned` | "Cloud VM created successfully" |
| `SecretsConfigured` | "Environment secrets configured on VM" |
| `TenantReady` | "Your tenant is ready!" |
| `ProvisioningFailed` | "{error_message}" |

---

## SSE Endpoints

### Checkout Events (for new subscriptions)

```
GET /api/v1/checkout/{checkout_session_id}/events
```

- Streams `CheckoutEvent` objects
- Filtered by `checkout_session_id`
- Used during initial checkout flow
- Closes after `TenantReady` or `ProvisioningFailed`

### Tenant Events (for retry-provision)

```
GET /api/v1/tenants/{tenant_id}/events
```

- Streams `ProvisioningEvent` objects
- Filtered by `tenant_id`
- Used when retrying failed provisioning
- Closes after `TenantReady` or `ProvisioningFailed`

---

## CLI Callback Handler Architecture

### Non-Blocking Design

The CLI callback handler must return HTTP response immediately and handle SSE in background:

```rust
// File: systemprompt-core/crates/infra/cloud/src/checkout/client.rs

async fn callback_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CallbackParams>,
) -> Html<String> {
    // Case 1: Error from checkout
    if let Some(error) = &params.error {
        send_result(&state.tx, Err(anyhow!("Checkout error: {}", error))).await;
        return Html(state.error_html.clone());
    }

    // Case 2: Completed (tenant already exists)
    if let (Some(transaction_id), Some(tenant_id)) = (&params.transaction_id, &params.tenant_id) {
        if params.status.as_deref() == Some("completed") {
            let result = Ok(CheckoutCallbackResult { transaction_id, tenant_id });
            send_result(&state.tx, result).await;
            return Html(state.success_html.replace("{{TENANT_ID}}", &tenant_id));
        }
    }

    // Case 3: Pending (need to wait for webhook/provisioning)
    if params.status.as_deref() == Some("pending") {
        if let Some(checkout_session_id) = &params.checkout_session_id {
            CliService::info("Payment confirmed, waiting for provisioning...");

            // Spawn background task for SSE subscription
            let api_client = Arc::clone(&state.api_client);
            let tx = Arc::clone(&state.tx);
            let transaction_id = params.transaction_id.clone()
                .unwrap_or_else(|| checkout_session_id.clone());

            tokio::spawn(async move {
                match wait_for_checkout_provisioning(&api_client, &checkout_session_id).await {
                    Ok(event) => {
                        let result = Ok(CheckoutCallbackResult {
                            transaction_id,
                            tenant_id: event.tenant_id,
                        });
                        send_result(&tx, result).await;
                    }
                    Err(e) => {
                        send_result(&tx, Err(e)).await;
                    }
                }
            });

            // Return immediately - browser shows waiting page
            return Html(state.waiting_html.clone());
        }
    }

    // Case 4: Missing required params
    send_result(&state.tx, Err(anyhow!("Missing required callback params"))).await;
    Html(state.error_html.clone())
}
```

### HTML Templates

```rust
// File: systemprompt-core/crates/entry/cli/src/cloud/checkout/templates.rs

pub struct CheckoutTemplates {
    pub success_html: &'static str,   // Shown when tenant is ready
    pub error_html: &'static str,     // Shown on errors
    pub waiting_html: &'static str,   // Shown during provisioning (new)
}
```

---

## Backend Event Broadcasting

### AppState with Broadcast Channels

```rust
// File: crates/management-api/src/api/mod.rs

pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub fly_provisioner: Option<Arc<FlyProvisioner>>,
    pub oauth_service: Option<Arc<OAuthService>>,
    pub provisioning_events: broadcast::Sender<ProvisioningEvent>,
    pub checkout_events: broadcast::Sender<CheckoutEvent>,
}
```

### Sending Events During Provisioning

```rust
// File: crates/management-api/src/api/webhooks/provisioning.rs

fn send_event(
    state: &AppState,
    checkout_session_id: &Option<String>,
    tenant_id: TenantId,
    tenant_name: &str,
    event_type: ProvisioningEventType,
    status: &str,
    message: Option<&str>,
    app_url: Option<String>,
) {
    if let Some(session_id) = checkout_session_id {
        // Checkout flow - send to checkout_events channel
        let _ = state.checkout_events.send(CheckoutEvent {
            checkout_session_id: session_id.clone(),
            tenant_id: tenant_id.as_uuid(),
            tenant_name: tenant_name.to_string(),
            event_type,
            status: status.to_string(),
            message: message.map(String::from),
            app_url,
        });
    } else {
        // Retry-provision flow - send to provisioning_events channel
        let _ = state.provisioning_events.send(ProvisioningEvent {
            tenant_id: tenant_id.as_uuid(),
            event_type,
            status: status.to_string(),
            message: message.map(String::from),
            app_url,
        });
    }
}
```

---

## Paddle Webhook Custom Data

### Checkout Request

```rust
// When creating checkout session, include checkout_session_id
let checkout_session_id = uuid::Uuid::new_v4().to_string();

let custom_data = json!({
    "region": region,
    "redirect_uri": redirect_uri,
    "checkout_session_id": checkout_session_id,
});
```

### Webhook Handler

```rust
// Extract checkout_session_id from webhook custom_data
let checkout_session_id = webhook.custom_data
    .as_ref()
    .and_then(|cd| cd.get("checkout_session_id"))
    .and_then(|v| v.as_str())
    .map(String::from);

// Pass to provisioning functions
provision_fly_infrastructure(state, tenant_id, config, checkout_session_id).await?;
```

---

## Expected CLI Output

```
$ just tenant
✔ Tenant operation · Create
✔ Tenant type · Cloud
✔ Select a plan · basic
✔ Select a region · US East (Virginia)
⠁ Creating checkout session...
ℹ Starting checkout callback server on http://127.0.0.1:8766
ℹ Opening Paddle checkout in your browser...
ℹ URL: https://sandbox-pay.paddle.io/hsc_01abc123...
ℹ Waiting for checkout completion...
ℹ (timeout in 300 seconds)
ℹ Payment confirmed, waiting for provisioning...

[SUBSCRIPTION_CREATED] Payment confirmed, creating tenant...
[TENANT_CREATED] Tenant 'happy-fox-123' created
[DATABASE_CREATED] Database and credentials created
[SECRETS_STORED] Credentials stored securely
[VM_PROVISIONING_STARTED] Starting cloud VM provisioning...
[VM_PROVISIONING_PROGRESS] Creating compute instance...
[VM_PROVISIONING_PROGRESS] Configuring networking...
[VM_PROVISIONED] Cloud VM created successfully
[SECRETS_CONFIGURED] Environment secrets configured on VM
[TENANT_READY] Your tenant is ready!

✓ Checkout complete! Tenant ID: abc-123-def
⠹ Provisioning cloud infrastructure...
✓ Infrastructure provisioned successfully
⠹ Syncing new tenant...
⠹ Fetching database credentials...
✓ Database credentials retrieved

✓ Tenant created successfully
  Name: happy-fox-123
  App URL: https://happy-fox-123.systemprompt.app
```

---

## Implementation Status

### Completed

- [x] `ProvisioningEventType` enum with all event types
- [x] `CheckoutEvent` struct for checkout flows
- [x] `ProvisioningEvent` struct for retry flows
- [x] SSE endpoint: `GET /api/v1/checkout/{checkout_session_id}/events`
- [x] SSE endpoint: `GET /api/v1/tenants/{tenant_id}/events`
- [x] `checkout_session_id` in Paddle custom_data
- [x] Event broadcasts during provisioning
- [x] CLI non-blocking callback handler
- [x] CLI SSE subscription for checkout events
- [x] `WAITING_HTML` template

### Backend Files Modified

| File | Changes |
|------|---------|
| `models/provisioning_events.rs` | Added event types, CheckoutEvent, ProvisioningEvent |
| `api/mod.rs` | Added checkout_events broadcast channel to AppState |
| `api/webhooks/paddle.rs` | Added checkout_events SSE endpoint |
| `api/webhooks/provisioning.rs` | Added send_event helper, event broadcasts |
| `api/webhooks/handlers.rs` | Pass checkout_session_id to provisioning |
| `api/checkout.rs` | Generate checkout_session_id |
| `routes.rs` | Added checkout events route |
| `services/paddle/webhook.rs` | Extract checkout_session_id from custom_data |

### CLI Files Modified

| File | Changes |
|------|---------|
| `api_client/types.rs` | Added CheckoutEvent, new event types |
| `api_client/client.rs` | Added subscribe_checkout_events method |
| `checkout/client.rs` | Non-blocking callback handler, spawn SSE task |
| `checkout/templates.rs` | Added WAITING_HTML |
| `tenant_ops/create.rs` | Pass waiting_html to CheckoutTemplates |

---

## Remaining Gaps

### Repository: systemprompt-db (Management API)

| Endpoint | Status | Priority |
|----------|--------|----------|
| GET `/api/v1/tenants/{id}/logs` | Implemented | - |
| POST `/api/v1/tenants/{id}/restart` | Implemented | - |
| POST `/api/v1/tenants/{id}/retry-provision` | Implemented | - |

### Repository: systemprompt-template

| Component | Status | Priority |
|-----------|--------|----------|
| MCP Extensions in Docker | Missing | High |
| Node.js in Docker | Missing | High |
| Build MCP script | Missing | Medium |

### Repository: systemprompt-core

| Component | Status | Priority |
|-----------|--------|----------|
| `logs` CLI command | Implemented | - |
| `restart` CLI command | Implemented | - |
| `retry-provision` CLI command | Missing | Medium |

---

## Testing

### Test Checkout Flow

```bash
# 1. Start management API
cd /var/www/html/systemprompt-db
just dev

# 2. Run CLI checkout
cd /var/www/html/systemprompt-template
just tenant
# Select: Create > Cloud > basic > US East

# 3. Complete Paddle checkout in browser

# 4. Observe SSE events in CLI terminal
```

### Test SSE Endpoint Directly

```bash
# Subscribe to checkout events
curl -N -H "Authorization: Bearer $TOKEN" \
  "https://api-sandbox.systemprompt.io/api/v1/checkout/{checkout_session_id}/events"

# Subscribe to tenant events
curl -N -H "Authorization: Bearer $TOKEN" \
  "https://api-sandbox.systemprompt.io/api/v1/tenants/{tenant_id}/events"
```
