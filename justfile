# SystemPrompt Template
set dotenv-load

CLI := "target/debug/systemprompt"
RELEASE_DIR := "target/release"

default:
    @just --list

# ══════════════════════════════════════════════════════════════════════════════
# BUILD & RUN
# ══════════════════════════════════════════════════════════════════════════════

# Build workspace (use --release for release build)
build *FLAGS:
    cargo build --manifest-path=core/Cargo.toml --target-dir=target {{FLAGS}}

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
# Produces: .systemprompt/tenants.json
# ══════════════════════════════════════════════════════════════════════════════

# Create cloud tenant
tenant-create *ARGS:
    {{CLI}} cloud tenant create {{ARGS}}

# List all tenants
tenant-list:
    {{CLI}} cloud tenant list

# Show tenant details
tenant-show *ARGS:
    {{CLI}} cloud tenant show {{ARGS}}

# ══════════════════════════════════════════════════════════════════════════════
# PROJECT — Local setup
# Produces: services/ boilerplate
# ══════════════════════════════════════════════════════════════════════════════

# Initialize new project
init *FLAGS:
    {{CLI}} cloud init {{FLAGS}}

# ══════════════════════════════════════════════════════════════════════════════
# PROFILE — Configuration
# Requires: init (local) or login+tenant (cloud)
# Produces: .systemprompt/profiles/<env>/
# ══════════════════════════════════════════════════════════════════════════════

# Create a new profile
profile-create NAME *ARGS:
    {{CLI}} cloud profile create {{NAME}} {{ARGS}}

# List all profiles
profile-list:
    {{CLI}} cloud profile list

# Show profile configuration
profile-show *ARGS:
    {{CLI}} cloud profile show {{ARGS}}

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
# QUICKSTART
# ══════════════════════════════════════════════════════════════════════════════

# Local development setup (tenant-create starts PostgreSQL automatically)
quickstart-local: init tenant-create (profile-create "local") migrate sync-local
    @echo "Done! Run 'just start' to begin"

# Cloud deployment setup (interactive)
quickstart-cloud: login tenant-create (profile-create "local") migrate sync-local
    @echo "Done! Run 'just deploy' to push to cloud"
