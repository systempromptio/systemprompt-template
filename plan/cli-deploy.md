# CLI Deploy Flow

How registry tokens are integrated into the 'just tenant => cloud' deployment flow.

## Overview

The CLI fetches registry tokens **on-demand** at deploy time. Tokens are never stored locally - they are fetched fresh for each deployment operation.

## Current Implementation Status: COMPLETE

The CLI deploy flow is fully implemented in systemprompt-template.

## Flow Diagram

```
User                   CLI                    Management API         Fly Registry
  │                     │                           │                      │
  │── just deploy ─────>│                           │                      │
  │                     │                           │                      │
  │                     │── Load credentials ──────>│                      │
  │                     │   (.systemprompt/         │                      │
  │                     │    credentials.json)      │                      │
  │                     │                           │                      │
  │                     │── Load profile ──────────>│                      │
  │                     │   (SYSTEMPROMPT_PROFILE)  │                      │
  │                     │                           │                      │
  │                     │── Get tenant_id from ────>│                      │
  │                     │   profile.cloud.tenant_id │                      │
  │                     │                           │                      │
  │                     │── Build Docker image ────>│                      │
  │                     │                           │                      │
  │                     │── GET /registry-token ───>│                      │
  │                     │<── { registry, user, ────│                      │
  │                     │     password, repo }      │                      │
  │                     │                           │                      │
  │                     │── docker login ──────────────────────────────────>│
  │                     │── docker push ───────────────────────────────────>│
  │                     │                           │                      │
  │                     │── POST /deploy ──────────>│                      │
  │                     │<── { status, url } ──────│                      │
  │                     │                           │                      │
  │<── Success ─────────│                           │                      │
```

## Key Files

### API Client

**File**: `systemprompt-template/core/crates/infra/cloud/src/api_client/client.rs:186-191`

```rust
pub async fn get_registry_token(&self, tenant_id: &str) -> Result<RegistryToken> {
    let response: ApiResponse<RegistryToken> = self
        .get(&ApiPaths::tenant_registry_token(tenant_id))
        .await?;
    Ok(response.data)
}
```

### Deploy Command

**File**: `systemprompt-template/core/crates/entry/cli/src/cloud/deploy.rs:151-166`

```rust
let api_client = CloudApiClient::new(&creds.api_url, &creds.api_token);

if !skip_push {
    let spinner = CliService::spinner("Pushing to registry...");
    let registry_token = api_client.get_registry_token(tenant_id).await?;
    docker_login(
        &registry_token.registry,
        &registry_token.username,
        &registry_token.password,
    )?;
    docker_push(&image)?;
    spinner.finish_and_clear();
    CliService::success("Image pushed");
}
```

### Docker Utilities

**File**: `systemprompt-template/core/crates/entry/cli/src/common/docker.rs`

```rust
pub fn docker_login(registry: &str, username: &str, password: &str) -> Result<()> {
    let mut child = Command::new("docker")
        .args(["login", registry, "-u", username, "--password-stdin"])
        .stdin(Stdio::piped())
        .spawn()?;

    // Password passed via stdin (secure, not visible in process list)
    child.stdin.as_mut().unwrap().write_all(password.as_bytes())?;
    child.wait()?;
    Ok(())
}
```

## Security Properties

| Property | Implementation |
|----------|----------------|
| On-demand fetch | Token fetched at `deploy.rs:155` each deploy |
| No local storage | Token used immediately, not persisted |
| Secure transmission | HTTPS to Management API |
| Secure docker login | Password via stdin, not command line |
| Authenticated requests | OAuth token in Authorization header |

## Authentication Prerequisites

Before `just deploy` works:

1. **Login**: `just login` - Creates `.systemprompt/credentials.json`
2. **Tenant**: `just tenant` - Creates/selects tenant
3. **Profile**: Cloud profile with `cloud.enabled: true` and `cloud.tenant_id` set

## Error Handling

| Error | Cause | Resolution |
|-------|-------|------------|
| "Deployment requires cloud credentials" | No credentials.json | Run `just login` |
| "No tenant configured" | Missing tenant_id in profile | Run `just tenant` |
| "Registry service not available" | Fly.io disabled on backend | Contact admin |
| "No cloud infrastructure provisioned" | Tenant not provisioned | Wait for provisioning |

## Optional Improvements

### 1. Token Refresh Warning

```rust
if creds.expires_within(Duration::minutes(10)) {
    CliService::warning("Token expires soon. Consider 'just login'");
}
```

### 2. Retry Logic

```rust
async fn get_registry_token_with_retry(
    client: &CloudApiClient,
    tenant_id: &str,
) -> Result<RegistryToken> {
    for attempt in 0..3 {
        match client.get_registry_token(tenant_id).await {
            Ok(token) => return Ok(token),
            Err(e) if is_transient(&e) => {
                tokio::time::sleep(Duration::from_millis(100 * 2u64.pow(attempt))).await;
            }
            Err(e) => return Err(e),
        }
    }
    Err(anyhow!("Failed after 3 retries"))
}
```

## No Code Changes Required

The CLI deploy flow is complete. This document serves as reference for the existing implementation.
