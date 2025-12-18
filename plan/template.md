# systemprompt-template: Cloud Deployment Scripts

## Overview

Add scripts and commands to systemprompt-template that enable users to deploy to "SystemPrompt Cloud" without knowing about Fly.io. Uses Fly.io Registry for images.

## User Experience

### One-time Setup
```bash
just cloud-setup
# Opens browser for OAuth authentication
# Creates or links to a tenant
# Saves credentials to .env.secrets
```

### Deploy
```bash
just release     # Build + push + deploy (full workflow)
# OR
just cloud-build # Build image only
just cloud-push  # Push to Fly.io registry
just deploy      # Deploy (call API with image)
```

### Check Status
```bash
just cloud-status  # Show tenant status and URL
just cloud-logs    # View application logs
```

## File Structure

```
systemprompt-template/
├── infrastructure/
│   └── scripts/
│       ├── setup-cloud.sh      # One-time setup
│       ├── deploy.sh           # Deploy to cloud
│       ├── push-image.sh       # Push to Fly.io registry
│       └── cloud-status.sh     # Check status
├── justfile                    # Add cloud commands
└── .env.secrets               # Generated (gitignored)
```

## Implementation

### File: `infrastructure/scripts/setup-cloud.sh`

```bash
#!/bin/bash
set -euo pipefail

#
# SystemPrompt Cloud Setup
# Links this repository to a tenant in SystemPrompt Cloud
#

API_BASE_URL="${SYSTEMPROMPT_API_URL:-https://api.systemprompt.io}"

echo "╔════════════════════════════════════════════╗"
echo "║     SystemPrompt Cloud Setup               ║"
echo "╚════════════════════════════════════════════╝"
echo ""

# Check prerequisites
command -v curl >/dev/null 2>&1 || { echo "Error: curl required"; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "Error: jq required"; exit 1; }
command -v docker >/dev/null 2>&1 || { echo "Error: docker required"; exit 1; }

# Check for existing configuration
if [ -f ".env.secrets" ]; then
    if grep -q "TENANT_ID" .env.secrets && grep -q "SYSTEMPROMPT_API_TOKEN" .env.secrets; then
        source .env.secrets
        echo "Found existing configuration:"
        echo "  Tenant ID: ${TENANT_ID:-not set}"
        echo ""
        read -p "Reconfigure? (y/N): " RECONFIG
        if [ "${RECONFIG:-n}" != "y" ]; then
            echo "Keeping existing configuration."
            echo "Run 'just deploy' to deploy."
            exit 0
        fi
    fi
fi

# Step 1: OAuth Authentication
echo ""
echo "Step 1: Authentication"
echo "─────────────────────────────────────────────"
echo "Opening browser for login..."

AUTH_URL="${API_BASE_URL}/auth/login?redirect=cli"

# Try to open browser
if command -v open >/dev/null 2>&1; then
    open "$AUTH_URL"
elif command -v xdg-open >/dev/null 2>&1; then
    xdg-open "$AUTH_URL"
else
    echo "Please open this URL in your browser:"
    echo "  $AUTH_URL"
fi

echo ""
echo "After logging in, copy the API token shown."
echo ""
read -p "Paste your API token: " SYSTEMPROMPT_API_TOKEN

if [ -z "$SYSTEMPROMPT_API_TOKEN" ]; then
    echo "Error: Token required"
    exit 1
fi

# Verify token
echo ""
echo "Verifying token..."
RESPONSE=$(curl -s -w "\n%{http_code}" \
    -H "Authorization: Bearer $SYSTEMPROMPT_API_TOKEN" \
    "${API_BASE_URL}/api/v1/auth/me")

HTTP_CODE=$(echo "$RESPONSE" | tail -1)
BODY=$(echo "$RESPONSE" | head -n -1)

if [ "$HTTP_CODE" != "200" ]; then
    echo "Error: Invalid token (HTTP $HTTP_CODE)"
    echo "$BODY" | jq -r '.error.message // .' 2>/dev/null || echo "$BODY"
    exit 1
fi

USER_EMAIL=$(echo "$BODY" | jq -r '.data.email // "unknown"')
echo "✓ Authenticated as: $USER_EMAIL"

# Step 2: Select or Create Tenant
echo ""
echo "Step 2: Select Tenant"
echo "─────────────────────────────────────────────"

TENANTS_RESPONSE=$(curl -s \
    -H "Authorization: Bearer $SYSTEMPROMPT_API_TOKEN" \
    "${API_BASE_URL}/api/v1/tenants")

TENANT_COUNT=$(echo "$TENANTS_RESPONSE" | jq '.data | length')

TENANT_ID=""

if [ "$TENANT_COUNT" -gt 0 ]; then
    echo "Your existing tenants:"
    echo ""
    echo "$TENANTS_RESPONSE" | jq -r '.data[] | "  [\(.id | .[0:8])...] \(.name) → \(.fly_hostname // "provisioning...")"'
    echo ""
    echo "Enter tenant ID to use, or 'new' to create a new one."
    read -p "Choice: " TENANT_CHOICE

    if [ "$TENANT_CHOICE" != "new" ] && [ -n "$TENANT_CHOICE" ]; then
        # Validate tenant ID exists
        TENANT_EXISTS=$(echo "$TENANTS_RESPONSE" | jq -r --arg id "$TENANT_CHOICE" '.data[] | select(.id | startswith($id)) | .id')
        if [ -n "$TENANT_EXISTS" ]; then
            TENANT_ID="$TENANT_EXISTS"
            echo "✓ Selected tenant: $TENANT_ID"
        else
            echo "Tenant not found, creating new one..."
        fi
    fi
fi

if [ -z "$TENANT_ID" ]; then
    # Create new tenant
    REPO_NAME=$(basename -s .git "$(git config --get remote.origin.url 2>/dev/null)" || echo "my-site")
    read -p "Tenant name [$REPO_NAME]: " TENANT_NAME
    TENANT_NAME="${TENANT_NAME:-$REPO_NAME}"

    # Sanitize name: lowercase, alphanumeric + underscore only
    TENANT_NAME=$(echo "$TENANT_NAME" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9_]/_/g')

    echo "Creating tenant '$TENANT_NAME'..."

    CREATE_RESPONSE=$(curl -s -X POST \
        -H "Authorization: Bearer $SYSTEMPROMPT_API_TOKEN" \
        -H "Content-Type: application/json" \
        -d "{\"name\": \"${TENANT_NAME}\", \"region\": \"iad\"}" \
        "${API_BASE_URL}/api/v1/tenants/free")

    TENANT_ID=$(echo "$CREATE_RESPONSE" | jq -r '.data.tenant.id // empty')

    if [ -z "$TENANT_ID" ]; then
        echo "Error: Failed to create tenant"
        echo "$CREATE_RESPONSE" | jq -r '.error.message // .' 2>/dev/null || echo "$CREATE_RESPONSE"
        exit 1
    fi

    FLY_APP_NAME=$(echo "$CREATE_RESPONSE" | jq -r '.data.tenant.fly_app_name // empty')
    FLY_HOSTNAME=$(echo "$CREATE_RESPONSE" | jq -r '.data.tenant.fly_hostname // empty')

    echo "✓ Created tenant: $TENANT_ID"
    echo "  App name: $FLY_APP_NAME"
    echo "  Hostname: $FLY_HOSTNAME"
fi

# Get full tenant details
TENANT_DETAILS=$(curl -s \
    -H "Authorization: Bearer $SYSTEMPROMPT_API_TOKEN" \
    "${API_BASE_URL}/api/v1/tenants/${TENANT_ID}")

FLY_APP_NAME=$(echo "$TENANT_DETAILS" | jq -r '.data.fly_app_name // empty')
FLY_HOSTNAME=$(echo "$TENANT_DETAILS" | jq -r '.data.fly_hostname // empty')

# Step 3: Update tenant metadata with repository
echo ""
echo "Step 3: Link Repository"
echo "─────────────────────────────────────────────"

REPO_URL=$(git config --get remote.origin.url 2>/dev/null || echo "")
if [ -n "$REPO_URL" ]; then
    curl -s -X PATCH \
        -H "Authorization: Bearer $SYSTEMPROMPT_API_TOKEN" \
        -H "Content-Type: application/json" \
        -d "{\"metadata\": {\"repository\": \"${REPO_URL}\"}}" \
        "${API_BASE_URL}/api/v1/tenants/${TENANT_ID}" > /dev/null
    echo "✓ Linked repository: $REPO_URL"
else
    echo "⚠ No git remote found, skipping repository link"
fi

# Step 4: Save configuration
echo ""
echo "Step 4: Save Configuration"
echo "─────────────────────────────────────────────"

cat > .env.secrets << EOF
# SystemPrompt Cloud Configuration
# Generated by setup-cloud.sh on $(date -Iseconds)
# DO NOT COMMIT THIS FILE

# API Authentication
SYSTEMPROMPT_API_TOKEN=${SYSTEMPROMPT_API_TOKEN}
SYSTEMPROMPT_API_URL=${API_BASE_URL}

# Tenant Configuration
TENANT_ID=${TENANT_ID}
FLY_APP_NAME=${FLY_APP_NAME}
FLY_HOSTNAME=${FLY_HOSTNAME}

# Registry (for docker push)
FLY_REGISTRY=registry.fly.io
FLY_IMAGE_REPO=registry.fly.io/${FLY_APP_NAME}
EOF

echo "✓ Saved to .env.secrets"

# Ensure .env.secrets is in .gitignore
if ! grep -q "^\.env\.secrets$" .gitignore 2>/dev/null; then
    echo ".env.secrets" >> .gitignore
    echo "✓ Added .env.secrets to .gitignore"
fi

# Step 5: Docker registry login
echo ""
echo "Step 5: Docker Registry Setup"
echo "─────────────────────────────────────────────"
echo "Logging into Fly.io registry..."

# Get registry token from API (or use API token directly)
# Fly.io uses the API token for registry auth with username "x"
echo "$SYSTEMPROMPT_API_TOKEN" | docker login registry.fly.io -u x --password-stdin

if [ $? -eq 0 ]; then
    echo "✓ Docker logged into registry.fly.io"
else
    echo "⚠ Docker login failed - you may need to run 'flyctl auth docker' manually"
fi

# Done
echo ""
echo "╔════════════════════════════════════════════╗"
echo "║     Setup Complete!                        ║"
echo "╚════════════════════════════════════════════╝"
echo ""
echo "Your site will be available at:"
echo "  https://${FLY_HOSTNAME:-$FLY_APP_NAME.fly.dev}"
echo ""
echo "Next steps:"
echo "  just release    # Build, push, and deploy"
echo ""
echo "Or step by step:"
echo "  just cloud-build   # Build Docker image"
echo "  just cloud-push    # Push to registry"
echo "  just deploy        # Deploy to cloud"
```

### File: `infrastructure/scripts/push-image.sh`

```bash
#!/bin/bash
set -euo pipefail

#
# Push Docker image to Fly.io registry
#

# Load configuration
if [ ! -f .env.secrets ]; then
    echo "Error: .env.secrets not found"
    echo "Run 'just cloud-setup' first"
    exit 1
fi

source .env.secrets

if [ -z "${FLY_APP_NAME:-}" ]; then
    echo "Error: FLY_APP_NAME not set in .env.secrets"
    exit 1
fi

# Generate tag based on timestamp and git sha
TIMESTAMP=$(date +%s)
GIT_SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
TAG="deploy-${TIMESTAMP}-${GIT_SHA}"

IMAGE_NAME="${FLY_REGISTRY:-registry.fly.io}/${FLY_APP_NAME}"
FULL_IMAGE="${IMAGE_NAME}:${TAG}"
LATEST_IMAGE="${IMAGE_NAME}:latest"

echo "Pushing image to Fly.io registry..."
echo "  Image: $FULL_IMAGE"

# Tag the local image
LOCAL_IMAGE="${LOCAL_IMAGE:-systemprompt-template:latest}"
docker tag "$LOCAL_IMAGE" "$FULL_IMAGE"
docker tag "$LOCAL_IMAGE" "$LATEST_IMAGE"

# Push both tags
docker push "$FULL_IMAGE"
docker push "$LATEST_IMAGE"

echo ""
echo "✓ Pushed: $FULL_IMAGE"
echo "✓ Pushed: $LATEST_IMAGE"

# Output the image name for use in deploy
echo ""
echo "IMAGE=$FULL_IMAGE" > .env.image
echo "Image saved to .env.image"
```

### File: `infrastructure/scripts/deploy.sh`

```bash
#!/bin/bash
set -euo pipefail

#
# Deploy to SystemPrompt Cloud
# Calls systemprompt-db API to update the tenant's Docker image
#

# Load configuration
if [ ! -f .env.secrets ]; then
    echo "Error: .env.secrets not found"
    echo "Run 'just cloud-setup' first"
    exit 1
fi

source .env.secrets

# Get image to deploy
if [ -n "${1:-}" ]; then
    IMAGE="$1"
elif [ -f .env.image ]; then
    source .env.image
else
    # Default to latest
    IMAGE="${FLY_IMAGE_REPO:-registry.fly.io/${FLY_APP_NAME}}:latest"
fi

echo "╔════════════════════════════════════════════╗"
echo "║     Deploying to SystemPrompt Cloud        ║"
echo "╚════════════════════════════════════════════╝"
echo ""
echo "  Tenant: ${TENANT_ID}"
echo "  Image:  ${IMAGE}"
echo ""

# Call deploy API
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST \
    -H "Authorization: Bearer ${SYSTEMPROMPT_API_TOKEN}" \
    -H "Content-Type: application/json" \
    -d "{\"image\": \"${IMAGE}\"}" \
    "${SYSTEMPROMPT_API_URL}/api/v1/tenants/${TENANT_ID}/deploy")

HTTP_CODE=$(echo "$RESPONSE" | tail -1)
BODY=$(echo "$RESPONSE" | head -n -1)

if [ "$HTTP_CODE" -eq 200 ]; then
    echo "✓ Deployment successful!"
    echo ""

    STATUS=$(echo "$BODY" | jq -r '.data.status // .status // "unknown"')
    MESSAGE=$(echo "$BODY" | jq -r '.data.message // .message // ""')
    APP_URL=$(echo "$BODY" | jq -r '.data.app_url // .app_url // empty')

    echo "  Status: $STATUS"
    [ -n "$MESSAGE" ] && echo "  Message: $MESSAGE"
    [ -n "$APP_URL" ] && echo "  URL: $APP_URL"
else
    echo "✗ Deployment failed (HTTP $HTTP_CODE)"
    echo ""
    echo "$BODY" | jq -r '.error.message // .' 2>/dev/null || echo "$BODY"
    exit 1
fi
```

### File: `infrastructure/scripts/cloud-status.sh`

```bash
#!/bin/bash
set -euo pipefail

#
# Check SystemPrompt Cloud deployment status
#

if [ ! -f .env.secrets ]; then
    echo "Error: .env.secrets not found"
    echo "Run 'just cloud-setup' first"
    exit 1
fi

source .env.secrets

echo "SystemPrompt Cloud Status"
echo "═════════════════════════════════════════════"
echo ""

# Get tenant status
RESPONSE=$(curl -s \
    -H "Authorization: Bearer ${SYSTEMPROMPT_API_TOKEN}" \
    "${SYSTEMPROMPT_API_URL}/api/v1/tenants/${TENANT_ID}/status")

STATUS=$(echo "$RESPONSE" | jq -r '.data.status // .status // "unknown"')
APP_URL=$(echo "$RESPONSE" | jq -r '.data.app_url // .app_url // empty')

echo "Tenant ID: ${TENANT_ID}"
echo "Status:    ${STATUS}"
[ -n "$APP_URL" ] && echo "URL:       ${APP_URL}"

# Get tenant details
DETAILS=$(curl -s \
    -H "Authorization: Bearer ${SYSTEMPROMPT_API_TOKEN}" \
    "${SYSTEMPROMPT_API_URL}/api/v1/tenants/${TENANT_ID}")

FLY_STATUS=$(echo "$DETAILS" | jq -r '.data.fly_status // empty')
HOSTNAME=$(echo "$DETAILS" | jq -r '.data.fly_hostname // empty')

[ -n "$FLY_STATUS" ] && echo "Machine:   ${FLY_STATUS}"
[ -n "$HOSTNAME" ] && echo "Hostname:  ${HOSTNAME}"
```

### File: `justfile` (additions)

Add these commands to the existing justfile:

```just
# ═══════════════════════════════════════════════════════════════════
# Cloud Deployment Commands
# ═══════════════════════════════════════════════════════════════════

# One-time setup: link this repo to SystemPrompt Cloud
cloud-setup:
    @chmod +x infrastructure/scripts/setup-cloud.sh
    @./infrastructure/scripts/setup-cloud.sh

# Build Docker image for deployment
cloud-build:
    @echo "Building Docker image..."
    docker build -f infrastructure/docker/app.Dockerfile -t systemprompt-template:latest .

# Push image to Fly.io registry
cloud-push:
    @chmod +x infrastructure/scripts/push-image.sh
    @./infrastructure/scripts/push-image.sh

# Deploy to SystemPrompt Cloud (uses latest pushed image)
deploy image="":
    @chmod +x infrastructure/scripts/deploy.sh
    @./infrastructure/scripts/deploy.sh "{{image}}"

# Full release: build, push, and deploy
release: cloud-build cloud-push deploy
    @echo ""
    @echo "Release complete!"

# Check cloud deployment status
cloud-status:
    @chmod +x infrastructure/scripts/cloud-status.sh
    @./infrastructure/scripts/cloud-status.sh

# View cloud application logs (requires flyctl)
cloud-logs:
    @if [ -f .env.secrets ]; then \
        source .env.secrets && flyctl logs -a "$$FLY_APP_NAME"; \
    else \
        echo "Run 'just cloud-setup' first"; \
    fi
```

## Configuration Files

### File: `.env.secrets` (generated by setup)

```bash
# SystemPrompt Cloud Configuration
# Generated by setup-cloud.sh
# DO NOT COMMIT THIS FILE

# API Authentication
SYSTEMPROMPT_API_TOKEN=eyJhbGciOiJIUzI1NiIs...
SYSTEMPROMPT_API_URL=https://api.systemprompt.io

# Tenant Configuration
TENANT_ID=12345678-1234-1234-1234-123456789abc
FLY_APP_NAME=sp-abc123
FLY_HOSTNAME=abc123.systemprompt.io

# Registry
FLY_REGISTRY=registry.fly.io
FLY_IMAGE_REPO=registry.fly.io/sp-abc123
```

### File: `.env.image` (generated by push)

```bash
IMAGE=registry.fly.io/sp-abc123:deploy-1702841234-abc1234
```

## Dockerfile Updates

The existing `infrastructure/docker/app.Dockerfile` should work as-is. No changes needed.

## .gitignore Updates

Ensure these are in `.gitignore`:

```
.env.secrets
.env.image
.env.local
```

## Reusability

This system is reusable because:

1. **No hardcoded values**: All configuration comes from `.env.secrets`
2. **Per-repo setup**: Each blog runs `just cloud-setup` independently
3. **API abstraction**: Scripts talk to systemprompt-db, not Fly.io directly

### To use in another blog:

```bash
# 1. Copy the scripts (or they're part of the template)
cp -r infrastructure/scripts/setup-cloud.sh ../other-blog/infrastructure/scripts/
cp -r infrastructure/scripts/deploy.sh ../other-blog/infrastructure/scripts/
# etc.

# 2. Add justfile commands (or they're part of the template)

# 3. Run setup in the new repo
cd ../other-blog
just cloud-setup

# 4. Deploy
just release
```

## Summary of Files

| File | Purpose |
|------|--------|
| `infrastructure/scripts/setup-cloud.sh` | One-time setup, OAuth, tenant creation |
| `infrastructure/scripts/push-image.sh` | Push Docker image to Fly.io registry |
| `infrastructure/scripts/deploy.sh` | Call systemprompt-db deploy API |
| `infrastructure/scripts/cloud-status.sh` | Check deployment status |
| `justfile` | Add cloud-* commands |
| `.env.secrets` | Generated credentials (gitignored) |
| `.env.image` | Generated image tag (gitignored) |

## Dependencies

- `curl` - API calls
- `jq` - JSON parsing
- `docker` - Build and push images
- `flyctl` (optional) - For `cloud-logs` command only

## Security Notes

1. **Never commit `.env.secrets`** - Contains API token
2. **Token expiry**: JWT expires in 1 week, user must re-run `cloud-setup` periodically
3. **Registry auth**: Docker credentials are stored in `~/.docker/config.json`
