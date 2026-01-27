# SystemPrompt Template
set dotenv-load

CLI_RELEASE := "target/release/systemprompt"

# Use newest binary (release vs debug, whichever is most recent)
CLI := if path_exists("target/release/systemprompt") == "true" { \
    if path_exists("target/debug/systemprompt") == "true" { \
        `[ target/release/systemprompt -nt target/debug/systemprompt ] && echo target/release/systemprompt || echo target/debug/systemprompt` \
    } else { \
        "target/release/systemprompt" \
    } \
} else if path_exists("target/debug/systemprompt") == "true" { \
    "target/debug/systemprompt" \
} else { \
    "echo 'ERROR: No CLI binary found. Run: just build' && exit 1" \
}

# Default: run CLI with any arguments
default *ARGS:
    {{CLI}} {{ARGS}}

# Run CLI with full session context (profile + auth token)
cli *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    SESSION_FILE="{{justfile_directory()}}/.systemprompt/sessions/index.json"
    if [ -f "$SESSION_FILE" ]; then
        ACTIVE_KEY=$(jq -r '.active_key // "local"' "$SESSION_FILE")
        export SYSTEMPROMPT_PROFILE=$(jq -r ".sessions[\"$ACTIVE_KEY\"].profile_path // empty" "$SESSION_FILE")
        export SYSTEMPROMPT_AUTH_TOKEN=$(jq -r ".sessions[\"$ACTIVE_KEY\"].session_token // empty" "$SESSION_FILE")
    fi
    if [ -z "${SYSTEMPROMPT_PROFILE:-}" ]; then
        export SYSTEMPROMPT_PROFILE="{{justfile_directory()}}/.systemprompt/profiles/local/profile.yaml"
    fi
    exec {{CLI}} {{ARGS}}

# Get DATABASE_URL from profile secrets (for sqlx compile-time checks)
_db-url:
    @if [ -n "$SYSTEMPROMPT_PROFILE" ] && [ -f "$SYSTEMPROMPT_PROFILE" ]; then \
        PROFILE_DIR="$(dirname "$SYSTEMPROMPT_PROFILE")"; \
        SECRETS_PATH="$(yq -r '.secrets.secrets_path // "./secrets.json"' "$SYSTEMPROMPT_PROFILE")"; \
        if [ "${SECRETS_PATH#/}" = "$SECRETS_PATH" ]; then \
            SECRETS_FILE="$PROFILE_DIR/$SECRETS_PATH"; \
        else \
            SECRETS_FILE="$SECRETS_PATH"; \
        fi; \
        if [ -f "$SECRETS_FILE" ]; then \
            jq -r '.database_url' "$SECRETS_FILE"; \
        else \
            echo "postgres://systemprompt:systemprompt@localhost:5432/systemprompt"; \
        fi; \
    else \
        cat .systemprompt/tenants.json 2>/dev/null | jq -r '.tenants[] | select(.tenant_type == "local") | .database_url' | head -1 || echo "postgres://systemprompt:systemprompt@localhost:5432/systemprompt"; \
    fi

# ══════════════════════════════════════════════════════════════════════════════
# BUILD & RUN
# ══════════════════════════════════════════════════════════════════════════════

# Build the project
build *FLAGS:
    #!/usr/bin/env bash
    set -euo pipefail
    export SYSTEMPROMPT_PROFILE="{{env_var_or_default('SYSTEMPROMPT_PROFILE', '')}}"

    # Check if local profile has database access
    SECRETS_FILE="{{justfile_directory()}}/.systemprompt/profiles/local/secrets.json"
    USE_OFFLINE=false

    if [ -f "$SECRETS_FILE" ]; then
        DB_URL=$(jq -r '.database_url // empty' "$SECRETS_FILE" 2>/dev/null)
        if [ -n "$DB_URL" ] && [ "$DB_URL" != "null" ]; then
            # Try to connect to the database (timeout after 2 seconds)
            if pg_isready -d "$DB_URL" -t 2 >/dev/null 2>&1; then
                export DATABASE_URL="$DB_URL"
                echo "Using database: $DB_URL"
            else
                echo "Database not reachable, using offline mode"
                USE_OFFLINE=true
            fi
        else
            echo "No database_url in secrets, using offline mode"
            USE_OFFLINE=true
        fi
    else
        echo "No local profile secrets found, using offline mode"
        USE_OFFLINE=true
    fi

    # Sync DATABASE_URL to MCP extension directories for sqlx compile-time checks
    if [ "$USE_OFFLINE" = "false" ]; then
        for dir in extensions/mcp/*/; do
            if [ -f "$dir/Cargo.toml" ]; then
                echo "DATABASE_URL=$DATABASE_URL" > "$dir/.env"
            fi
        done
    fi

    cargo update systemprompt --quiet 2>/dev/null || true

    if [ "$USE_OFFLINE" = "true" ]; then
        SQLX_OFFLINE=true cargo build --workspace {{FLAGS}}
    else
        cargo build --workspace {{FLAGS}}
    fi

# Start server (always uses local profile)
start:
    {{CLI}} infra services start --profile local

# Run migrations
migrate:
    {{CLI}} infra db migrate

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
# Builds everything first since cloud tenant creation deploys immediately
tenant:
    {{CLI_RELEASE}} cloud tenant

# List all tenants
tenants:
    {{CLI}} cloud tenant list

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
    just build --release
    {{CLI_RELEASE}} cloud deploy {{FLAGS}}

# Check deployment status
status:
    {{CLI}} cloud status

# ══════════════════════════════════════════════════════════════════════════════
# MCP — Build MCP server binaries
# ══════════════════════════════════════════════════════════════════════════════

# Build all MCP servers (reads from manifest.yaml files)
build-mcp:
    DATABASE_URL="$(just _db-url)" {{CLI}} build mcp --release

# Build everything for deployment (Rust binary + MCP servers)
build-all:
    just build --release
    just build-mcp
    just web-build
    @echo "All components built"

# ══════════════════════════════════════════════════════════════════════════════
# DOCKER — Local testing only (deploy pushes directly to Fly registry)
# ══════════════════════════════════════════════════════════════════════════════

# Build Docker image for local testing
docker-build TAG="local":
    docker build -f .systemprompt/Dockerfile -t systemprompt-template:{{TAG}} .

# Run image locally for testing
docker-run TAG="local":
    docker run -p 8080:8080 --env-file .env systemprompt-template:{{TAG}}

# Test build without pushing
docker-test:
    just build-all
    just docker-build test
    @echo "Docker build successful! Image: systemprompt-template:test"

