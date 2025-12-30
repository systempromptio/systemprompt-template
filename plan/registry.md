# Registry Token System

How tenants get a token for the Fly.io registry associated with their VM.

## Overview

Each tenant deploys their own Docker image to `registry.fly.io/{tenant-app-name}:{tag}`. The Management API provides registry credentials via the `/api/v1/tenants/{id}/registry-token` endpoint.

## Current Implementation

**Endpoint**: `GET /api/v1/tenants/{id}/registry-token`

**File**: `crates/management-api/src/api/tenants/handlers.rs:269-303`

```rust
let response = RegistryTokenResponse {
    registry: "registry.fly.io".to_string(),
    username: "x".to_string(),
    password: provisioner.config().api_token.clone(),  // ORG-LEVEL TOKEN
    repository: app_name.clone(),
};
```

**Security Concern**: The current implementation returns `FLY_API_TOKEN` which is an organization-level token with access to ALL Fly.io apps in the organization.

## Recommended Improvement: Per-Tenant Scoped Tokens

### Approach

Generate a per-app deploy token during tenant provisioning using `fly tokens create deploy`. This token is scoped to only the tenant's specific Fly app.

### Token Generation

During provisioning (after app creation):

```bash
fly tokens create deploy -a sp-{tenant_id_slug} -x 8760h
```

This creates a token that can only:
- Push images to the app's registry
- Deploy to the app's machines
- Cannot access other apps

### Database Schema

Add column to `tenants` table:

```sql
ALTER TABLE management.tenants
ADD COLUMN fly_deploy_token_encrypted TEXT;
```

### Provisioner Changes

**File**: `crates/management-api/src/services/fly/provisioner.rs`

After `create_app()`:

```rust
// Generate scoped deploy token
let deploy_token = self.generate_deploy_token(app_name).await?;
let encrypted = encrypt_token(&deploy_token, &self.config.token_key)?;

// Store in database
db::update_tenant_deploy_token(&pool, tenant_id, &encrypted).await?;
```

### Handler Changes

**File**: `crates/management-api/src/api/tenants/handlers.rs`

```rust
// Prefer tenant-specific token, fall back to org token for legacy
let password = if let Some(encrypted_token) = &tenant.fly_deploy_token_encrypted {
    decrypt_token(encrypted_token, &state.config.deploy_token_key)?
} else {
    // Fallback for legacy tenants
    tracing::warn!(tenant_id = %id, "Using org-level token (legacy tenant)");
    provisioner.config().api_token.clone()
};
```

## Token Lifecycle

```
PROVISIONING                      RUNTIME                     ROTATION
─────────────                     ───────                     ────────
1. Create Fly App                 6. CLI: GET /registry-token
   (sp-{tenant_id})                  │
       │                             ▼
       ▼                          7. Decrypt stored token
2. fly tokens create deploy          │
   -a {app} -x 8760h                 ▼
       │                          8. Return to CLI
       ▼                             │
3. Encrypt token (AES-256-GCM)       ▼
       │                          9. docker login + push
       ▼                             │
4. Store in DB                       ▼
   fly_deploy_token_encrypted    10. POST /deploy
       │
       ▼
5. Complete provisioning         11. Admin can regenerate
                                     POST /admin/tenants/{id}/
                                     generate-deploy-token
```

## Files to Modify

1. **Migration**: New file `migrations/NNNNNN_tenant_deploy_tokens.sql`
2. **Model**: `crates/management-api/src/models/tenant.rs` - Add field
3. **Provisioner**: `crates/management-api/src/services/fly/provisioner.rs` - Generate token
4. **Handler**: `crates/management-api/src/api/tenants/handlers.rs` - Use stored token
5. **Config**: `crates/management-api/src/services/fly/config.rs` - Add encryption key

## Security Notes

- Tokens encrypted at rest (AES-256-GCM)
- Encryption key in environment: `DEPLOY_TOKEN_ENCRYPTION_KEY`
- One-year expiry, refresh mechanism needed
- Fallback to org token for legacy tenants (with warning)
- Audit logging on all token requests
