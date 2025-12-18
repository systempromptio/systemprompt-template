# systemprompt-db: Deploy Endpoint Implementation

## Overview

Add a deployment endpoint to systemprompt-db that allows users to deploy new Docker images to their tenant's Fly.io machine, abstracting away all Fly.io details.

## API Endpoint

### `POST /api/v1/tenants/{id}/deploy`

**Authentication**: Bearer JWT token (existing auth system)

**Request Body**:
```json
{
  "image": "registry.fly.io/sp-abc123:deploy-1702841234"
}
```

**Success Response** (200):
```json
{
  "data": {
    "status": "deployed",
    "message": "Deployed image: registry.fly.io/sp-abc123:deploy-1702841234",
    "app_url": "https://abc123.systemprompt.io",
    "machine_id": "abc123def456",
    "deployed_at": "2025-12-17T12:00:00Z"
  },
  "meta": {
    "request_id": "uuid",
    "timestamp": "2025-12-17T12:00:00Z"
  }
}
```

**Error Responses**:
- 401: Invalid or expired token
- 403: User doesn't own this tenant
- 400: Tenant has no Fly.io infrastructure
- 503: Fly.io service unavailable

## Implementation

### File: `crates/api-types/src/tenant.rs`

Add request/response types:

```rust
/// Request to deploy a new image to a tenant
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeployRequest {
    /// Docker image to deploy
    /// Example: "registry.fly.io/sp-abc123:deploy-1702841234"
    pub image: String,
}

/// Response from a deployment
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeployResponse {
    /// Deployment status: "deploying", "deployed", "failed"
    pub status: String,
    /// Human-readable message
    pub message: String,
    /// Public URL of the deployed app
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_url: Option<String>,
    /// Fly.io machine ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine_id: Option<String>,
    /// Timestamp of deployment
    pub deployed_at: chrono::DateTime<chrono::Utc>,
}
```

### File: `crates/management-api/src/api/tenants.rs`

Add deploy handler:

```rust
use crate::models::{DeployRequest, DeployResponse};

/// Deploy a new image to a tenant's Fly.io machine
///
/// This endpoint updates the Docker image running on the tenant's
/// Fly.io machine and restarts it with the new image.
#[utoipa::path(
    post,
    path = "/api/v1/tenants/{id}/deploy",
    request_body = DeployRequest,
    responses(
        (status = 200, description = "Deployment successful", body = SingleResponse<DeployResponse>),
        (status = 400, description = "Bad request - tenant has no infrastructure"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not tenant owner"),
        (status = 503, description = "Fly.io service unavailable"),
    ),
    params(
        ("id" = TenantId, Path, description = "Tenant UUID")
    ),
    security(("bearer_auth" = []))
)]
pub async fn deploy(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<TenantId>,
    Json(req): Json<DeployRequest>,
) -> ApiResult<Json<SingleResponse<DeployResponse>>> {
    // 1. Verify ownership
    let customer = get_customer(&state, user.user_id()).await?;
    let tenant = db::get_tenant(&state.db, id).await?;
    verify_ownership(&tenant, &customer.id)?;

    // 2. Check Fly.io provisioner is available
    let provisioner = state
        .fly_provisioner
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("Deployment service not available"))?;

    // 3. Get tenant's Fly.io app and machine
    let app_name = tenant
        .fly_app_name
        .as_ref()
        .ok_or_else(|| ApiError::bad_request("Tenant has no cloud infrastructure provisioned"))?;

    let machine_id = tenant
        .fly_machine_id
        .as_ref()
        .ok_or_else(|| ApiError::bad_request("Tenant has no running machine"))?;

    // 4. Get current machine config
    let machine = provisioner
        .client()
        .get_machine(app_name, machine_id)
        .await
        .map_err(|e| ApiError::internal_error(format!("Failed to get machine: {e}")))?;

    // 5. Update image in config
    let mut config = machine.config;
    config.image = req.image.clone();

    // 6. Update machine (triggers restart with new image)
    let updated_machine = provisioner
        .client()
        .update_machine(app_name, machine_id, config)
        .await
        .map_err(|e| ApiError::internal_error(format!("Failed to deploy: {e}")))?;

    // 7. Build response
    let app_url = tenant.fly_hostname.as_ref().map(|h| format!("https://{h}"));

    let response = DeployResponse {
        status: "deployed".to_string(),
        message: format!("Deployed image: {}", req.image),
        app_url,
        machine_id: Some(updated_machine.id),
        deployed_at: chrono::Utc::now(),
    };

    tracing::info!(
        tenant_id = %id,
        image = %req.image,
        machine_id = %machine_id,
        "Tenant deployment completed"
    );

    Ok(Json(SingleResponse::new(response)))
}
```

### File: `crates/management-api/src/main.rs`

Add route to router:

```rust
// In the tenants routes section, add:
.route("/api/v1/tenants/:id/deploy", post(tenants::deploy))
```

### File: `crates/management-api/src/services/fly/models.rs`

Verify `MachineConfig` has mutable `image` field (it does - line 576 of client.rs uses it).

## Fly.io Registry Integration

The deploy endpoint accepts images from Fly.io's registry:
- Format: `registry.fly.io/{app-name}:{tag}`
- Each tenant app has its own registry namespace
- Images are pushed using `flyctl deploy --image` or `flyctl auth docker` + `docker push`

### Registry Authentication Flow

For the template to push images:
1. Template's deploy script gets a Fly.io auth token from systemprompt-db
2. Uses token to authenticate with registry.fly.io
3. Pushes image to tenant's app registry
4. Calls deploy endpoint with image tag

**Alternative approach (simpler)**:
1. systemprompt-db exposes an endpoint to get registry credentials
2. Or: systemprompt-db accepts image upload directly (more complex)

### Recommended: Add Registry Token Endpoint

```rust
// GET /api/v1/tenants/{id}/registry-token
// Returns a temporary token for pushing to registry.fly.io

pub async fn get_registry_token(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<TenantId>,
) -> ApiResult<Json<RegistryTokenResponse>> {
    // Verify ownership
    let customer = get_customer(&state, user.user_id()).await?;
    let tenant = db::get_tenant(&state.db, id).await?;
    verify_ownership(&tenant, &customer.id)?;

    // The Fly.io API token can be used for registry auth
    // Or generate a scoped deploy token
    let token = state.fly_provisioner
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("Not available"))?
        .client()
        .config
        .api_token
        .clone();

    Ok(Json(RegistryTokenResponse {
        registry: "registry.fly.io".to_string(),
        username: "x", // Fly.io uses "x" as username
        password: token,
        repository: tenant.fly_app_name.unwrap_or_default(),
    }))
}
```

## Database Changes

None required. The deploy endpoint uses existing tenant data:
- `fly_app_name`: Target Fly.io app
- `fly_machine_id`: Target machine to update

Optionally, track deployment history in a new table:

```sql
CREATE TABLE management.deployments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES management.tenants(id),
    image TEXT NOT NULL,
    status TEXT NOT NULL,  -- 'pending', 'deployed', 'failed'
    deployed_by UUID NOT NULL REFERENCES management.users(id),
    deployed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    error_message TEXT
);

CREATE INDEX idx_deployments_tenant ON management.deployments(tenant_id);
```

## OpenAPI Documentation

Add to utoipa OpenAPI spec:
- `DeployRequest` schema
- `DeployResponse` schema
- `/api/v1/tenants/{id}/deploy` endpoint
- `/api/v1/tenants/{id}/registry-token` endpoint

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_deploy_updates_machine_image() {
    // Setup test tenant with mock Fly.io client
    // Call deploy endpoint
    // Verify update_machine was called with correct image
}

#[tokio::test]
async fn test_deploy_requires_ownership() {
    // Create tenant owned by user A
    // Try to deploy as user B
    // Expect 403 Forbidden
}

#[tokio::test]
async fn test_deploy_requires_infrastructure() {
    // Create tenant without fly_app_name
    // Try to deploy
    // Expect 400 Bad Request
}
```

### Integration Tests

```bash
# Create tenant
TENANT_ID=$(curl -X POST .../tenants/free | jq -r '.data.tenant.id')

# Build and push image
docker build -t registry.fly.io/sp-$TENANT_ID:test .
docker push registry.fly.io/sp-$TENANT_ID:test

# Deploy
curl -X POST .../tenants/$TENANT_ID/deploy \
  -d '{"image": "registry.fly.io/sp-'$TENANT_ID':test"}'

# Verify
curl .../tenants/$TENANT_ID/status
```

## Summary of Changes

| File | Change |
|------|--------|
| `crates/api-types/src/tenant.rs` | Add `DeployRequest`, `DeployResponse` |
| `crates/management-api/src/api/tenants.rs` | Add `deploy()` handler |
| `crates/management-api/src/main.rs` | Add route |
| `crates/management-api/src/api/tenants.rs` | Add `get_registry_token()` (optional) |

## Security Considerations

1. **Ownership verification**: Only tenant owner can deploy
2. **Image validation**: Consider validating image format/source
3. **Rate limiting**: Apply rate limits to prevent abuse
4. **Audit logging**: Log all deployments with user/tenant/image
5. **Registry token scope**: Token should only allow push to tenant's app
