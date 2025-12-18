# GitHub-Based Cloud Deployment System

## Overview

Create a reusable GitHub Actions-based deployment system that:
1. Links template repositories to cloud VMs via systemprompt-db API
2. Enables automated CI/CD deployments to Fly.io
3. Is reusable across all blog/template instances

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         GitHub Repository                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │ Source Code  │  │ .github/     │  │ Repository Secrets       │  │
│  │              │  │ workflows/   │  │ - SYSTEMPROMPT_API_TOKEN │  │
│  │              │  │              │  │ - TENANT_ID              │  │
│  │              │  │              │  │ - FLY_API_TOKEN          │  │
│  └──────────────┘  └──────────────┘  └──────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
           │                                      │
           │ Push to main / Manual trigger        │
           ▼                                      ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    GitHub Actions Workflow                           │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌──────────────┐  │
│  │ Checkout   │→ │ Build      │→ │ Push to    │→ │ Deploy to    │  │
│  │            │  │ Docker     │  │ Registry   │  │ Fly.io       │  │
│  └────────────┘  └────────────┘  └────────────┘  └──────────────┘  │
│                                                          │          │
│                                                          ▼          │
│                                        ┌──────────────────────────┐ │
│                                        │ Update tenant metadata   │ │
│                                        │ via systemprompt-db API  │ │
│                                        └──────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
           │                                      │
           ▼                                      ▼
┌─────────────────────┐              ┌─────────────────────────────────┐
│   Fly.io Cloud      │              │      systemprompt-db API        │
│  ┌───────────────┐  │              │  ┌───────────────────────────┐  │
│  │ Tenant VM     │  │◄────────────►│  │ Tenant Management         │  │
│  │ (fly_app_name)│  │   Secrets    │  │ - Provisioning            │  │
│  └───────────────┘  │              │  │ - Secrets management      │  │
└─────────────────────┘              │  │ - Metadata storage        │  │
                                     │  └───────────────────────────┘  │
                                     └─────────────────────────────────┘
```

## Authentication Flow

1. **Initial Setup (one-time, manual)**:
   - User authenticates with systemprompt-db via GitHub/Google OAuth
   - Receives JWT token
   - Creates tenant via API (if not exists)
   - Stores JWT + TENANT_ID in GitHub repository secrets

2. **CI/CD Authentication**:
   - GitHub Actions uses stored JWT to call systemprompt-db API
   - Uses Fly.io deploy token for direct deployments
   - JWT can be refreshed via a scheduled workflow if needed

## Implementation Steps

### Step 1: Create Reusable Workflow Template

**File**: `.github/workflows/deploy.yml`

This workflow will:
- Trigger on push to main OR manual dispatch
- Build Docker image from source
- Push to GitHub Container Registry (GHCR)
- Deploy to Fly.io using flyctl
- Update tenant metadata via systemprompt-db API

### Step 2: Create Setup Script for New Repositories

**File**: `infrastructure/scripts/setup-cloud.sh`

This script will:
- Authenticate user with systemprompt-db API (OAuth flow)
- Create or link tenant
- Store mapping in tenant metadata (repository URL)
- Output required GitHub secrets for user to configure
- Generate fly.toml configuration

### Step 3: Create fly.toml Template

**File**: `fly.toml.template`

Template configuration for Fly.io deployment that:
- Uses environment variables for customization
- References the correct Docker image
- Configures health checks, scaling, regions

### Step 4: Add Deployment Commands to justfile

Add commands:
- `just cloud-setup` - Run initial cloud setup
- `just cloud-deploy` - Manual deploy trigger
- `just cloud-status` - Check deployment status via API

### Step 5: Create Composite Action for Reusability

**File**: `.github/actions/deploy-to-fly/action.yml`

Reusable action that other repositories can reference:
```yaml
uses: systempromptio/systemprompt-template/.github/actions/deploy-to-fly@main
```

## Files to Create

### 1. `.github/workflows/deploy.yml`
```yaml
name: Deploy to Cloud

on:
  push:
    branches: [main]
  workflow_dispatch:
    inputs:
      environment:
        description: 'Deployment environment'
        required: true
        default: 'production'
        type: choice
        options:
          - production
          - sandbox

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          submodules: recursive

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=sha
            type=raw,value=latest

      - name: Build release binary
        run: |
          # Build steps from justfile
          cargo build --manifest-path=core/Cargo.toml --bin systemprompt --release
          npm run build --prefix core/web
          # Stage artifacts
          mkdir -p infrastructure/build-context/release
          cp core/target/release/systemprompt infrastructure/build-context/release/

      - name: Build and push Docker image
        uses: docker/build-push-action@v5
        with:
          context: .
          file: infrastructure/docker/app.Dockerfile
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

      - name: Setup Fly.io CLI
        uses: superfly/flyctl-actions/setup-flyctl@master

      - name: Deploy to Fly.io
        env:
          FLY_API_TOKEN: ${{ secrets.FLY_API_TOKEN }}
        run: |
          flyctl deploy --image ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:latest

      - name: Update tenant metadata
        if: success()
        env:
          SYSTEMPROMPT_API_TOKEN: ${{ secrets.SYSTEMPROMPT_API_TOKEN }}
          TENANT_ID: ${{ secrets.TENANT_ID }}
          API_BASE_URL: ${{ vars.SYSTEMPROMPT_API_URL || 'https://api.systemprompt.io' }}
        run: |
          curl -X PATCH "$API_BASE_URL/api/v1/tenants/$TENANT_ID" \
            -H "Authorization: Bearer $SYSTEMPROMPT_API_TOKEN" \
            -H "Content-Type: application/json" \
            -d '{
              "metadata": {
                "repository": "${{ github.repository }}",
                "last_deploy": "${{ github.event.head_commit.timestamp }}",
                "commit_sha": "${{ github.sha }}",
                "deployed_by": "${{ github.actor }}"
              }
            }'
```

### 2. `fly.toml`
```toml
app = "" # Set via FLY_APP environment variable or --app flag
primary_region = "iad"

[build]
  image = "ghcr.io/systempromptio/systemprompt-template:latest"

[env]
  HOST = "0.0.0.0"
  PORT = "8080"
  SYSTEMPROMPT_PROFILE = "prod"
  RUST_LOG = "info"

[http_service]
  internal_port = 8080
  force_https = true
  auto_stop_machines = "stop"
  auto_start_machines = true
  min_machines_running = 0
  processes = ["app"]

[[http_service.checks]]
  grace_period = "30s"
  interval = "30s"
  method = "GET"
  timeout = "5s"
  path = "/api/v1/health"

[[vm]]
  memory = "512mb"
  cpu_kind = "shared"
  cpus = 1
```

### 3. `infrastructure/scripts/setup-cloud.sh`
```bash
#!/bin/bash
set -euo pipefail

# Cloud setup script for systemprompt-template
# Links this repository to a systemprompt-db tenant

API_BASE_URL="${SYSTEMPROMPT_API_URL:-https://api.systemprompt.io}"
REPO_URL=$(git config --get remote.origin.url)

echo "=== SystemPrompt Cloud Setup ==="
echo ""
echo "This script will:"
echo "1. Authenticate with systemprompt-db"
echo "2. Create or link a tenant for this repository"
echo "3. Generate required secrets for GitHub Actions"
echo ""

# Check for existing token
if [ -f ".env.secrets" ] && grep -q "SYSTEMPROMPT_API_TOKEN" .env.secrets; then
    echo "Found existing API token in .env.secrets"
    source .env.secrets
else
    echo "Opening browser for authentication..."
    echo "Please log in and copy the JWT token."
    echo ""

    # Open OAuth flow
    open "${API_BASE_URL}/auth/login" 2>/dev/null || \
    xdg-open "${API_BASE_URL}/auth/login" 2>/dev/null || \
    echo "Please open: ${API_BASE_URL}/auth/login"

    echo ""
    read -p "Paste your JWT token: " SYSTEMPROMPT_API_TOKEN
fi

# Check for existing tenant
echo ""
echo "Checking for existing tenants..."

TENANTS=$(curl -s -H "Authorization: Bearer ${SYSTEMPROMPT_API_TOKEN}" \
    "${API_BASE_URL}/api/v1/tenants")

TENANT_COUNT=$(echo "$TENANTS" | jq '.data | length')

if [ "$TENANT_COUNT" -gt 0 ]; then
    echo "Found $TENANT_COUNT existing tenant(s):"
    echo "$TENANTS" | jq -r '.data[] | "  - \(.id): \(.name) (\(.fly_hostname // "no hostname"))"'
    echo ""
    read -p "Use existing tenant? (enter ID or 'new'): " TENANT_CHOICE

    if [ "$TENANT_CHOICE" != "new" ]; then
        TENANT_ID="$TENANT_CHOICE"
    fi
fi

if [ -z "${TENANT_ID:-}" ]; then
    # Create new tenant
    REPO_NAME=$(basename -s .git "$REPO_URL")
    read -p "Tenant name [$REPO_NAME]: " TENANT_NAME
    TENANT_NAME="${TENANT_NAME:-$REPO_NAME}"

    echo "Creating tenant '$TENANT_NAME'..."

    RESPONSE=$(curl -s -X POST "${API_BASE_URL}/api/v1/tenants/free" \
        -H "Authorization: Bearer ${SYSTEMPROMPT_API_TOKEN}" \
        -H "Content-Type: application/json" \
        -d "{\"name\": \"${TENANT_NAME}\", \"region\": \"iad\"}")

    TENANT_ID=$(echo "$RESPONSE" | jq -r '.data.tenant.id')
    FLY_APP_NAME=$(echo "$RESPONSE" | jq -r '.data.tenant.fly_app_name')

    echo "Created tenant: $TENANT_ID"
    echo "Fly app: $FLY_APP_NAME"
fi

# Update tenant metadata with repository URL
echo "Linking repository to tenant..."
curl -s -X PATCH "${API_BASE_URL}/api/v1/tenants/${TENANT_ID}" \
    -H "Authorization: Bearer ${SYSTEMPROMPT_API_TOKEN}" \
    -H "Content-Type: application/json" \
    -d "{\"metadata\": {\"repository\": \"${REPO_URL}\"}}" > /dev/null

# Get tenant details
TENANT=$(curl -s -H "Authorization: Bearer ${SYSTEMPROMPT_API_TOKEN}" \
    "${API_BASE_URL}/api/v1/tenants/${TENANT_ID}")

FLY_APP_NAME=$(echo "$TENANT" | jq -r '.data.fly_app_name')

# Generate fly.toml
echo "Generating fly.toml..."
sed "s/^app = .*/app = \"${FLY_APP_NAME}\"/" fly.toml.template > fly.toml

echo ""
echo "=== Setup Complete ==="
echo ""
echo "Add these secrets to your GitHub repository:"
echo "  Settings > Secrets and variables > Actions > New repository secret"
echo ""
echo "  SYSTEMPROMPT_API_TOKEN: ${SYSTEMPROMPT_API_TOKEN}"
echo "  TENANT_ID: ${TENANT_ID}"
echo "  FLY_API_TOKEN: (get from 'flyctl tokens create deploy')"
echo ""
echo "Then push to main branch to trigger deployment."
```

### 4. Updates to `justfile`
```just
# Cloud deployment commands
cloud-setup:
    ./infrastructure/scripts/setup-cloud.sh

cloud-deploy:
    gh workflow run deploy.yml

cloud-status:
    @echo "Checking deployment status..."
    @if [ -f .env.secrets ]; then \
        source .env.secrets && \
        curl -s -H "Authorization: Bearer $$SYSTEMPROMPT_API_TOKEN" \
            "$${SYSTEMPROMPT_API_URL:-https://api.systemprompt.io}/api/v1/tenants/$$TENANT_ID/status" | jq; \
    else \
        echo "Run 'just cloud-setup' first"; \
    fi

cloud-logs:
    flyctl logs
```

## Required GitHub Secrets

| Secret | Description | Source |
|--------|-------------|--------|
| `SYSTEMPROMPT_API_TOKEN` | JWT from OAuth login | setup-cloud.sh output |
| `TENANT_ID` | UUID of linked tenant | setup-cloud.sh output |
| `FLY_API_TOKEN` | Fly.io deploy token | `flyctl tokens create deploy` |

## Required GitHub Variables (Optional)

| Variable | Description | Default |
|----------|-------------|---------|
| `SYSTEMPROMPT_API_URL` | API base URL | `https://api.systemprompt.io` |

## Reusability for Other Blogs

To use this system in another blog repository:

1. **Copy workflow files**:
   ```bash
   cp -r .github/workflows/deploy.yml ../other-blog/.github/workflows/
   cp fly.toml.template ../other-blog/
   cp infrastructure/scripts/setup-cloud.sh ../other-blog/infrastructure/scripts/
   ```

2. **Run setup**:
   ```bash
   cd ../other-blog
   ./infrastructure/scripts/setup-cloud.sh
   ```

3. **Configure GitHub secrets** as prompted

4. **Push to deploy**

## Token Refresh Strategy

Since JWT tokens expire, add a scheduled workflow to refresh:

```yaml
# .github/workflows/refresh-token.yml
name: Refresh API Token

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly
  workflow_dispatch:

jobs:
  refresh:
    runs-on: ubuntu-latest
    steps:
      - name: Check token validity
        env:
          TOKEN: ${{ secrets.SYSTEMPROMPT_API_TOKEN }}
        run: |
          # Decode JWT and check expiry
          # If expired, notify via issue or Slack
          echo "Token check not yet implemented - manual refresh required"
```

## Security Considerations

1. **JWT Storage**: Stored in GitHub encrypted secrets, never logged
2. **Token Scope**: JWT provides access only to owned tenants
3. **Fly Token**: Scoped deploy token, not org-wide
4. **No secrets in code**: All sensitive values via secrets/environment

## Next Steps After Implementation

1. Test full workflow with this template repository
2. Document token refresh process
3. Consider adding API key auth to systemprompt-db for simpler CI/CD
4. Create GitHub template repository for easy replication
