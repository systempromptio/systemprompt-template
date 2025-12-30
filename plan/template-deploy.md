# Template Deploy Command

Full deploy workflow: build Docker image, push to Fly registry, deploy via Management API.

## Overview

`just deploy` in systemprompt-template performs a complete deployment:

1. Build all crates (Rust binary + MCP servers)
2. Build Docker image
3. Fetch registry token (on-demand)
4. Push to Fly registry (per-tenant)
5. Deploy via Management API

**Note**: GHCR (GitHub Container Registry) targets have been removed. All deployments go directly to Fly registry per-tenant.

## Integrated Build Approach

**Decision**: `just deploy` automatically runs `build-all` before deployment.

```
just deploy
    │
    ├── just build-all
    │   ├── cargo build --release (systemprompt binary)
    │   └── build-mcp (infrastructure, admin, system-tools)
    │
    └── systemprompt cloud deploy
        ├── Build Docker image
        ├── Fetch registry token
        ├── docker push
        └── POST /deploy
```

## Justfile Configuration

**File**: `systemprompt-template/justfile`

```just
# Deploy to cloud (builds everything first)
deploy *FLAGS:
    just build-all
    {{CLI}} cloud deploy {{FLAGS}}
```

## Full Workflow

### Step 1: Build All Crates

```bash
# Build main binary
DATABASE_URL="..." cargo build --release --manifest-path=core/Cargo.toml

# Build MCP servers
cargo build --release -p system-tools
(cd extensions/mcp/infrastructure && cargo build --release)
(cd extensions/mcp/admin && cargo build --release)
```

**Outputs**:
- `target/release/systemprompt`
- `target/release/system-tools`
- `target/release/systemprompt-infrastructure`
- `target/release/systemprompt-admin`

### Step 2: Build Docker Image

**Dockerfile**: `.systemprompt/Dockerfile`

```dockerfile
FROM debian:bookworm-slim

# Copy pre-built binaries
COPY target/release/systemprompt /app/bin/
COPY target/release/system-tools /app/bin/mcp/
COPY target/release/systemprompt-infrastructure /app/bin/mcp/
COPY target/release/systemprompt-admin /app/bin/mcp/

# Copy web frontend
COPY core/web/dist/ /app/web/

# Copy services config
COPY services/ /app/services/
```

**Tag Format**: `registry.fly.io/{app_id}:deploy-{timestamp}-{git_sha}`

Example: `registry.fly.io/sp-ten-abc123:deploy-1705329622-a1b2c3d`

### Step 3: Fetch Registry Token

```rust
let registry_token = api_client.get_registry_token(tenant_id).await?;
// Returns: { registry, username, password, repository }
```

### Step 4: Docker Push

```bash
docker login registry.fly.io -u x --password-stdin
docker push registry.fly.io/{app_id}:{tag}
```

### Step 5: Deploy via API

```rust
let response = api_client.deploy(tenant_id, &image).await?;
// POST /api/v1/tenants/{id}/deploy
// Body: { "image": "registry.fly.io/{app_id}:{tag}" }
```

**Backend Action**: Updates Fly machine with new image, restarts.

## Deploy Command Flags

| Flag | Description |
|------|-------------|
| `--skip-push` | Skip docker push (re-deploy existing image) |
| `--tag <tag>` | Use custom tag instead of auto-generated |

## Example Usage

```bash
# Full deploy (build + push + deploy)
just deploy

# Skip build, re-deploy existing image
just deploy --skip-push --tag deploy-1705329622-a1b2c3d

# Custom tag
just deploy --tag v1.0.0
```

## Sequence Diagram

```
Developer              systemprompt-template        Management API       Fly.io
    │                         │                           │                │
    │── just deploy ─────────>│                           │                │
    │                         │                           │                │
    │                         │── cargo build --release ──│                │
    │                         │── build-mcp ──────────────│                │
    │                         │                           │                │
    │                         │── docker build ───────────│                │
    │                         │   -f .systemprompt/       │                │
    │                         │   Dockerfile              │                │
    │                         │                           │                │
    │                         │── GET /registry-token ───>│                │
    │                         │<── { token } ────────────│                │
    │                         │                           │                │
    │                         │── docker login ───────────────────────────>│
    │                         │── docker push ────────────────────────────>│
    │                         │                           │                │
    │                         │── POST /deploy ──────────>│                │
    │                         │                           │── Update ─────>│
    │                         │                           │   machine      │
    │                         │<── { status, url } ──────│                │
    │                         │                           │                │
    │<── Deployed! ───────────│                           │                │
    │    URL: https://...     │                           │                │
```

## Files Modified

### systemprompt-template/justfile

Change from:
```just
deploy *FLAGS:
    {{CLI}} cloud deploy {{FLAGS}}
```

To:
```just
deploy *FLAGS:
    just build-all
    {{CLI}} cloud deploy {{FLAGS}}
```

## Error Handling

| Phase | Error | Resolution |
|-------|-------|------------|
| Build | Compilation error | Fix code, re-run |
| Build | Missing DATABASE_URL | Ensure local tenant exists |
| Docker | Binary not found | Run `just build --release` |
| Docker | Web dist not found | Run `npm run build` |
| Push | Auth failure | Check credentials, re-login |
| Deploy | Image not found | Ensure push completed |
| Deploy | Machine update failed | Check Fly.io status |

## Rollback

To rollback to a previous deployment:

```bash
# Find previous tag from deployment history
just deploy --skip-push --tag deploy-{previous-timestamp}-{sha}
```

The Management API updates the Fly machine to use the specified image.
