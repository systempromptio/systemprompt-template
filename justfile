# SystemPrompt Template
set dotenv-load

CLI := "target/debug/systemprompt"
RELEASE_DIR := "target/release"

default:
    @just --list

# ══════════════════════════════════════════════════════════════════════════════
# BUILD & RUN
# ══════════════════════════════════════════════════════════════════════════════

# Get DATABASE_URL from local tenant (for sqlx compile-time checks)
_db-url:
    @cat .systemprompt/tenants.json 2>/dev/null | jq -r '.tenants[] | select(.tenant_type == "local") | .database_url' | head -1 || echo "postgres://systemprompt:systemprompt@localhost:5432/systemprompt"

# Build workspace (use --release for release build)
build *FLAGS:
    DATABASE_URL="$(just _db-url)" cargo build --manifest-path=core/Cargo.toml --target-dir=target {{FLAGS}}

# Start server
start:
    {{CLI}} services start

# ══════════════════════════════════════════════════════════════════════════════
# AUTH — Who you are
# Produces: .systemprompt/credentials.json
# ══════════════════════════════════════════════════════════════════════════════

# Authenticate with SystemPrompt Cloud
login ENV="production":
    {{CLI}} cloud auth login {{ENV}}

# Clear saved credentials
logout:
    {{CLI}} cloud auth logout

# Show current user and tenant
whoami:
    {{CLI}} cloud auth whoami

# ══════════════════════════════════════════════════════════════════════════════
# TENANT — Where your app runs in cloud
# Requires: login
# Produces: .systemprompt/tenants.json, .systemprompt/docker/<name>.yaml
# ══════════════════════════════════════════════════════════════════════════════

# Tenant operations (interactive menu)
tenant:
    {{CLI}} cloud tenant

# List all tenants
tenants:
    {{CLI}} cloud tenant list

# ══════════════════════════════════════════════════════════════════════════════
# PROJECT — Local setup
# Produces: services/ boilerplate
# ══════════════════════════════════════════════════════════════════════════════

# Initialize new project
init *FLAGS:
    {{CLI}} cloud init {{FLAGS}}

# ══════════════════════════════════════════════════════════════════════════════
# PROFILE — Configuration
# Requires: tenant
# Produces: .systemprompt/profiles/<name>/
# ══════════════════════════════════════════════════════════════════════════════

# Profile operations (interactive menu)
profile:
    {{CLI}} cloud profile

# List all profiles
profiles:
    {{CLI}} cloud profile list

# ══════════════════════════════════════════════════════════════════════════════
# DATABASE — Local PostgreSQL (per tenant)
# ══════════════════════════════════════════════════════════════════════════════

# Start PostgreSQL for a specific tenant (default: local)
db-up TENANT="local":
    docker compose -f .systemprompt/docker/{{TENANT}}.yaml up -d

# Stop PostgreSQL for a specific tenant
db-down TENANT="local":
    docker compose -f .systemprompt/docker/{{TENANT}}.yaml down

# Show PostgreSQL logs for a specific tenant
db-logs TENANT="local":
    docker compose -f .systemprompt/docker/{{TENANT}}.yaml logs -f

# Reset database (stop, remove volume, start fresh)
db-reset TENANT="local":
    docker compose -f .systemprompt/docker/{{TENANT}}.yaml down -v
    docker compose -f .systemprompt/docker/{{TENANT}}.yaml up -d

# List all tenant databases
db-list:
    @ls -1 .systemprompt/docker/*.yaml 2>/dev/null | xargs -I {} basename {} .yaml || echo "No tenant databases found"

# Run migrations
migrate:
    {{CLI}} db migrate

# ══════════════════════════════════════════════════════════════════════════════
# SYNC — Populate database
# Requires: migrate
# ══════════════════════════════════════════════════════════════════════════════

# Sync content to database
sync-content *ARGS:
    {{CLI}} cloud sync local content {{ARGS}}

# Sync skills to database
sync-skills *ARGS:
    {{CLI}} cloud sync local skills {{ARGS}}

# Sync all local content
sync-local:
    {{CLI}} cloud sync local content
    {{CLI}} cloud sync local skills

# Push to cloud
sync-push *ARGS:
    {{CLI}} cloud sync push {{ARGS}}

# Pull from cloud
sync-pull *ARGS:
    {{CLI}} cloud sync pull {{ARGS}}

# ══════════════════════════════════════════════════════════════════════════════
# DEPLOY — Push to cloud
# Requires: login + tenant + profile with cloud.enabled
# ══════════════════════════════════════════════════════════════════════════════

# Deploy to cloud
deploy *FLAGS:
    {{CLI}} cloud deploy {{FLAGS}}

# Check deployment status
status:
    {{CLI}} cloud status

# ══════════════════════════════════════════════════════════════════════════════
# DOCKER — Build and push base template image to GHCR
# Workflow: just build --release && just docker-build-ghcr && just docker-push
# ══════════════════════════════════════════════════════════════════════════════

# Build Docker image for local testing
docker-build TAG="local":
    docker build -f .systemprompt/Dockerfile -t systemprompt-template:{{TAG}} .

# Build Docker image tagged for GHCR
docker-build-ghcr TAG="latest":
    docker build -f .systemprompt/Dockerfile -t ghcr.io/systempromptio/systemprompt-template:{{TAG}} .

# Push image to GHCR (requires: docker login ghcr.io)
docker-push TAG="latest":
    docker push ghcr.io/systempromptio/systemprompt-template:{{TAG}}

# Build and push in one command
docker-release TAG="latest":
    just build --release
    just docker-build-ghcr {{TAG}}
    just docker-push {{TAG}}
    @echo "Released: ghcr.io/systempromptio/systemprompt-template:{{TAG}}"

# Run image locally for testing
docker-run TAG="local":
    docker run -p 8080:8080 --env-file .env systemprompt-template:{{TAG}}

# Test build without pushing
docker-test:
    just build --release
    just docker-build test
    @echo "Docker build successful! Image: systemprompt-template:test"

# ══════════════════════════════════════════════════════════════════════════════
# QUICKSTART
# ══════════════════════════════════════════════════════════════════════════════

# Full local setup: tenant → profile → migrate → sync
quickstart: tenant profile migrate sync-local
    @echo "Done! Run 'just start' to begin"
