# Cloud Deployment Fixes - All Repositories

This document outlines all gaps between current implementation and the target architecture.

---

## Repository: systemprompt-db (Management API)

### Summary of Gaps

| Endpoint | Status | Priority |
|----------|--------|----------|
| GET `/api/v1/tenants/{id}/logs` | Missing | High |
| POST `/api/v1/tenants/{id}/restart` | Missing | High |
| POST `/api/v1/tenants/{id}/retry-provision` | Missing | Medium |
| POST `/api/v1/tenants` (direct create) | N/A - By Design | - |

### Gap 1: Missing Logs Endpoint

**Current**: No endpoint to fetch tenant logs
**Required**: Endpoint that proxies Fly.io log retrieval

**Implementation**:

```rust
// File: crates/management-api/src/api/tenants/mod.rs

/// GET /api/v1/tenants/{id}/logs
pub async fn logs(
    State(state): State<AppState>,
    Path(id): Path<TenantId>,
    Query(params): Query<LogsParams>,
    auth: AuthContext,
) -> Result<impl IntoResponse, ApiError> {
    let tenant = get_tenant_for_owner(&state, &id, &auth).await?;

    let fly_app = tenant.fly_app_name
        .ok_or(ApiError::NotProvisioned)?;

    let logs = state.fly_client
        .get_logs(&fly_app, params.lines.unwrap_or(100))
        .await?;

    Ok(Json(LogsResponse { logs }))
}

#[derive(Deserialize)]
pub struct LogsParams {
    lines: Option<u32>,
    follow: Option<bool>,
}
```

**Route addition** (`routes.rs`):
```rust
.route("/api/v1/tenants/{id}/logs", get(api::tenants::logs))
```

**Fly client addition** (`services/fly/client/mod.rs`):
```rust
pub async fn get_logs(&self, app_name: &str, lines: u32) -> Result<Vec<LogEntry>> {
    // Use Fly Machines API: GET /apps/{app}/machines/{id}/logs
}
```

---

### Gap 2: Missing Restart Endpoint

**Current**: No way to restart a tenant's Fly machine
**Required**: Endpoint that triggers machine restart

**Implementation**:

```rust
// File: crates/management-api/src/api/tenants/mod.rs

/// POST /api/v1/tenants/{id}/restart
pub async fn restart(
    State(state): State<AppState>,
    Path(id): Path<TenantId>,
    auth: AuthContext,
) -> Result<impl IntoResponse, ApiError> {
    let tenant = get_tenant_for_owner(&state, &id, &auth).await?;

    let (fly_app, machine_id) = match (&tenant.fly_app_name, &tenant.fly_machine_id) {
        (Some(app), Some(machine)) => (app, machine),
        _ => return Err(ApiError::NotProvisioned),
    };

    state.fly_client
        .restart_machine(fly_app, machine_id)
        .await?;

    Ok(Json(json!({ "status": "restarting" })))
}
```

**Route addition**:
```rust
.route("/api/v1/tenants/{id}/restart", post(api::tenants::restart))
```

**Fly client addition**:
```rust
pub async fn restart_machine(&self, app_name: &str, machine_id: &str) -> Result<()> {
    // POST /apps/{app}/machines/{id}/restart
}
```

---

### Gap 3: Missing Retry Provisioning Endpoint

**Current**: No way to retry failed provisioning
**Required**: Endpoint to re-trigger provisioning for failed tenants

**Implementation**:

```rust
// File: crates/management-api/src/api/tenants/mod.rs

/// POST /api/v1/tenants/{id}/retry-provision
pub async fn retry_provision(
    State(state): State<AppState>,
    Path(id): Path<TenantId>,
    auth: AuthContext,
) -> Result<impl IntoResponse, ApiError> {
    let tenant = get_tenant_for_owner(&state, &id, &auth).await?;

    if tenant.fly_status != Some("failed".to_string()) {
        return Err(ApiError::InvalidState("Tenant is not in failed state"));
    }

    // Clean up any partial resources
    cleanup_partial_resources(&state, &tenant).await?;

    // Re-trigger provisioning
    let config = ProvisioningConfig::from_tenant(&tenant)?;
    provision_fly_infrastructure(&state, &tenant, &config).await?;

    Ok(Json(json!({ "status": "provisioning" })))
}
```

**Route addition**:
```rust
.route("/api/v1/tenants/{id}/retry-provision", post(api::tenants::retry_provision))
```

---

### Implementation Checklist - systemprompt-db

- [ ] Add `LogsParams` struct and `logs` handler in `api/tenants/mod.rs`
- [ ] Add `restart` handler in `api/tenants/mod.rs`
- [ ] Add `retry_provision` handler in `api/tenants/mod.rs`
- [ ] Add `get_logs` method to Fly client
- [ ] Add `restart_machine` method to Fly client
- [ ] Add routes in `routes.rs`
- [ ] Add OpenAPI documentation for new endpoints
- [ ] Write tests for new endpoints

---

## Repository: systemprompt-template

### Summary of Gaps

| Component | Status | Priority |
|-----------|--------|----------|
| MCP Extensions in Docker | Missing | High |
| Node.js in Docker | Missing | High |
| Build MCP script | Missing | Medium |
| Extensions env var | Missing | Medium |

### Gap 1: MCP Extensions Not in Docker Image

**Current Dockerfile** (`.systemprompt/Dockerfile`):
```dockerfile
COPY target/release/systemprompt /app/bin/
COPY services /app/services
COPY .systemprompt/profiles /app/services/profiles
COPY .systemprompt/entrypoint.sh /app/entrypoint.sh
COPY core/web/dist /app/web
# ❌ Missing: extensions/mcp/
```

**Fixed Dockerfile**:
```dockerfile
FROM debian:bookworm-slim

# Install runtime dependencies (including Node.js for MCP servers)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libpq5 \
    libssl3 \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 app
WORKDIR /app

RUN mkdir -p /app/bin /app/web /app/services /app/extensions /app/data /app/logs /app/storage

# Copy pre-built Rust binary
COPY target/release/systemprompt /app/bin/

# Copy pre-built React frontend
COPY core/web/dist /app/web

# Copy services configuration
COPY services /app/services

# Copy MCP extensions (pre-built)
COPY extensions/mcp /app/extensions/mcp

# Copy profiles
COPY .systemprompt/profiles /app/services/profiles

# Copy entrypoint
COPY .systemprompt/entrypoint.sh /app/entrypoint.sh

RUN chmod +x /app/bin/* /app/entrypoint.sh && chown -R app:app /app

USER app
EXPOSE 8080

ENV HOST=0.0.0.0 \
    PORT=8080 \
    RUST_LOG=info \
    PATH="/app/bin:$PATH" \
    SYSTEMPROMPT_SERVICES_PATH=/app/services \
    SYSTEMPROMPT_EXTENSIONS_PATH=/app/extensions \
    WEB_DIR=/app/web

HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1

CMD ["/app/bin/systemprompt", "services", "serve", "--foreground"]
```

---

### Gap 2: Missing MCP Build Script

**Add to `justfile`**:

```just
# Build MCP extensions
build-mcp:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Building MCP extensions..."
    for dir in extensions/mcp/*/; do
        if [ -f "$dir/package.json" ]; then
            echo "  Building: $dir"
            (cd "$dir" && npm install --production=false && npm run build)
        fi
    done
    echo "MCP extensions built successfully"

# Full build for deployment
build-all: build-release build-web build-mcp
    @echo "All components built"

# Deploy to cloud tenant (updated)
deploy tenant="": build-all
    {{CLI}} cloud deploy {{tenant}}
```

---

### Implementation Checklist - systemprompt-template

- [ ] Update `.systemprompt/Dockerfile` to include Node.js
- [ ] Update `.systemprompt/Dockerfile` to copy extensions
- [ ] Add `SYSTEMPROMPT_EXTENSIONS_PATH` environment variable
- [ ] Add `build-mcp` recipe to `justfile`
- [ ] Update `deploy` recipe to include `build-mcp`
- [ ] Test Docker build with all components
- [ ] Verify MCP servers work in deployed container

---

## Repository: systemprompt-core

### Summary of Gaps

| Component | Status | Priority |
|-----------|--------|----------|
| `logs` CLI command | Missing | High |
| `restart` CLI command | Missing | High |
| API client for new endpoints | Missing | High |

### Gap 1: Missing Logs CLI Command

**Add to `crates/entry/cli/src/cloud/mod.rs`**:

```rust
#[derive(Subcommand)]
pub enum CloudCommands {
    // ... existing commands

    /// View tenant logs
    Logs {
        /// Tenant ID
        #[arg(long)]
        tenant: Option<String>,

        /// Number of lines to show
        #[arg(long, default_value = "100")]
        lines: u32,

        /// Follow log output
        #[arg(long)]
        follow: bool,
    },
}
```

**Implementation**:

```rust
CloudCommands::Logs { tenant, lines, follow } => {
    let tenant_id = select_tenant_or_use(tenant).await?;
    let client = get_api_client().await?;

    if follow {
        // Stream logs via SSE
        let stream = client.stream_logs(&tenant_id).await?;
        while let Some(line) = stream.next().await {
            println!("{}", line?);
        }
    } else {
        let logs = client.get_logs(&tenant_id, lines).await?;
        for line in logs {
            println!("{}", line);
        }
    }
}
```

---

### Gap 2: Missing Restart CLI Command

**Add to CloudCommands**:

```rust
/// Restart tenant machine
Restart {
    /// Tenant ID
    #[arg(long)]
    tenant: Option<String>,
},
```

**Implementation**:

```rust
CloudCommands::Restart { tenant } => {
    let tenant_id = select_tenant_or_use(tenant).await?;
    let client = get_api_client().await?;

    println!("Restarting tenant {}...", tenant_id);
    client.restart_tenant(&tenant_id).await?;
    println!("Tenant restarted successfully");
}
```

---

### Gap 3: API Client Methods

**Add to `crates/infra/cloud/src/api_client/client.rs`**:

```rust
impl ApiClient {
    /// Get tenant logs
    pub async fn get_logs(&self, tenant_id: &str, lines: u32) -> Result<Vec<String>> {
        let url = format!("{}/api/v1/tenants/{}/logs?lines={}", self.base_url, tenant_id, lines);
        let response = self.client.get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;

        let data: LogsResponse = response.json().await?;
        Ok(data.logs)
    }

    /// Restart tenant machine
    pub async fn restart_tenant(&self, tenant_id: &str) -> Result<()> {
        let url = format!("{}/api/v1/tenants/{}/restart", self.base_url, tenant_id);
        self.client.post(&url)
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Retry failed provisioning
    pub async fn retry_provision(&self, tenant_id: &str) -> Result<()> {
        let url = format!("{}/api/v1/tenants/{}/retry-provision", self.base_url, tenant_id);
        self.client.post(&url)
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
```

---

### Implementation Checklist - systemprompt-core

- [ ] Add `Logs` command to `CloudCommands` enum
- [ ] Add `Restart` command to `CloudCommands` enum
- [ ] Implement `logs` command handler
- [ ] Implement `restart` command handler
- [ ] Add `get_logs` method to API client
- [ ] Add `restart_tenant` method to API client
- [ ] Add `retry_provision` method to API client
- [ ] Update `just logs` alias in template

---

## Documentation Updates Required

### Update cloud-deploy-flow.md

1. **Fix endpoint table** - Mark actual vs planned endpoints
2. **Update rotate-credentials** - Change from PATCH to POST (current implementation)
3. **Add note about tenant creation** - Tenants created via webhook, not direct POST
4. **Document existing endpoints not in runbook**:
   - GET `/api/v1/tenants/{id}/environment`
   - GET `/api/v1/tenants/{id}/registry-token`
   - GET `/api/v1/tenants/{id}/metrics`
   - POST `/api/v1/tenants/{id}/suspend`
   - POST `/api/v1/tenants/{id}/activate`

---

## Priority Order

### Phase 1: High Priority (Essential for MVP)
1. **systemprompt-template**: Add MCP extensions to Docker
2. **systemprompt-db**: Add logs endpoint
3. **systemprompt-core**: Add logs CLI command

### Phase 2: Medium Priority (Enhanced Operations)
1. **systemprompt-db**: Add restart endpoint
2. **systemprompt-db**: Add retry-provision endpoint
3. **systemprompt-core**: Add restart CLI command
4. **systemprompt-template**: Add build-mcp justfile recipe

### Phase 3: Documentation
1. Update cloud-deploy-flow.md with accurate endpoints
2. Add API reference for all endpoints
3. Update runbook with verified commands

---

## Testing Plan

### Integration Tests

```bash
# 1. Build and deploy with MCP extensions
cd /var/www/html/systemprompt-template
just build-all
just deploy --tenant test

# 2. Verify MCP servers available
just logs --tenant test  # Check for MCP initialization

# 3. Test restart
curl -X POST -H "Authorization: Bearer $TOKEN" \
  https://api.systemprompt.io/api/v1/tenants/test/restart

# 4. Test logs
curl -H "Authorization: Bearer $TOKEN" \
  "https://api.systemprompt.io/api/v1/tenants/test/logs?lines=50"

# 5. Force failure and test retry
# (simulate failure, then)
curl -X POST -H "Authorization: Bearer $TOKEN" \
  https://api.systemprompt.io/api/v1/tenants/test/retry-provision
```
