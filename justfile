# foodles.com
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
# BUILD & CHECK
# ══════════════════════════════════════════════════════════════════════════════

# Build (Windows) - always uses offline mode
[windows]
build *FLAGS:
    $env:SQLX_OFFLINE="true"; cargo build --workspace {{FLAGS}}

# Build (Unix) - tries database, falls back to offline
[unix]
build *FLAGS:
    #!/usr/bin/env bash
    set -euo pipefail
    export SYSTEMPROMPT_PROFILE="${SYSTEMPROMPT_PROFILE:-}"
    SECRETS_FILE="{{justfile_directory()}}/.systemprompt/profiles/local/secrets.json"
    USE_OFFLINE=false
    if [ -f "$SECRETS_FILE" ]; then
        DB_URL=$(sed -n 's/.*"database_url"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$SECRETS_FILE" 2>/dev/null | head -1)
        if [ -n "$DB_URL" ] && [ "$DB_URL" != "null" ]; then
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

# Clippy (Windows) - always uses offline mode
[windows]
clippy *FLAGS:
    $env:SQLX_OFFLINE="true"; cargo clippy --workspace {{FLAGS}} -- -D warnings

# Clippy (Unix) - tries database, falls back to offline
[unix]
clippy *FLAGS:
    #!/usr/bin/env bash
    set -euo pipefail
    SECRETS_FILE="{{justfile_directory()}}/.systemprompt/profiles/local/secrets.json"
    USE_OFFLINE=false
    if [ -f "$SECRETS_FILE" ]; then
        DB_URL=$(sed -n 's/.*"database_url"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$SECRETS_FILE" 2>/dev/null | head -1)
        if [ -n "$DB_URL" ] && [ "$DB_URL" != "null" ]; then
            if pg_isready -d "$DB_URL" -t 2 >/dev/null 2>&1; then
                export DATABASE_URL="$DB_URL"
            else
                USE_OFFLINE=true
            fi
        else
            USE_OFFLINE=true
        fi
    else
        USE_OFFLINE=true
    fi
    if [ "$USE_OFFLINE" = "true" ]; then
        SQLX_OFFLINE=true cargo clippy --workspace {{FLAGS}} -- -D warnings
    else
        cargo clippy --workspace {{FLAGS}} -- -D warnings
    fi

# Prepare SQLx offline query cache (requires running database)
prepare:
    #!/usr/bin/env bash
    set -euo pipefail
    SECRETS_FILE="{{justfile_directory()}}/.systemprompt/profiles/local/secrets.json"
    if [ ! -f "$SECRETS_FILE" ]; then
        echo "Error: No local profile secrets found at $SECRETS_FILE"
        echo "Run 'just db-up' first to start the database"
        exit 1
    fi
    DB_URL=$(jq -r '.database_url // empty' "$SECRETS_FILE" 2>/dev/null)
    if [ -z "$DB_URL" ] || [ "$DB_URL" = "null" ]; then
        echo "Error: No database_url in secrets"
        exit 1
    fi
    if ! pg_isready -d "$DB_URL" -t 2 >/dev/null 2>&1; then
        echo "Error: Database not reachable at $DB_URL"
        echo "Run 'just db-up' first to start the database"
        exit 1
    fi
    echo "Preparing SQLx offline cache..."
    export DATABASE_URL="$DB_URL"
    # Workspace-level prepare (catches lib crates)
    cargo sqlx prepare --workspace
    # Per-crate prepare for binary/extension crates that cargo sqlx skips
    EXTENSION_DIRS="extensions/cli/activity extensions/cli/slack extensions/web extensions/marketplace extensions/mcp/marketplace extensions/mcp/systemprompt"
    for dir in $EXTENSION_DIRS; do
        if [ -f "{{justfile_directory()}}/$dir/Cargo.toml" ]; then
            echo "  Preparing $dir..."
            (cd "{{justfile_directory()}}/$dir" && cargo sqlx prepare 2>&1 | tail -1) || true
            if ls "{{justfile_directory()}}/$dir/.sqlx/"*.json >/dev/null 2>&1; then
                cp "{{justfile_directory()}}/$dir/.sqlx/"*.json "{{justfile_directory()}}/.sqlx/"
            fi
        fi
    done
    echo "SQLx cache prepared successfully ($(ls {{justfile_directory()}}/.sqlx/ | wc -l) queries cached)"

# ══════════════════════════════════════════════════════════════════════════════
# SERVICES & DATABASE
# ══════════════════════════════════════════════════════════════════════════════

# Start server (always uses local profile)
start:
    {{CLI}} infra services start --profile local

# Start server with release binary
start-release:
    {{CLI_RELEASE}} infra services start --profile local

# Run migrations
migrate:
    {{CLI}} infra db migrate

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
# AUTH & TENANT & PROFILE
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

# Tenant operations (interactive menu)
# Builds everything first since cloud tenant creation deploys immediately
tenant:
    {{CLI_RELEASE}} cloud tenant

# List all tenants
tenants:
    {{CLI}} cloud tenant list

# Profile operations (interactive menu)
profile:
    {{CLI}} cloud profile

# List all profiles
profiles:
    {{CLI}} cloud profile list

# ══════════════════════════════════════════════════════════════════════════════
# SYNC
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
# DEPLOY
# ══════════════════════════════════════════════════════════════════════════════

# Deploy to cloud
# Note: publish_pipeline runs automatically on server startup with correct profile URLs
deploy *FLAGS:
    just build --release
    {{CLI_RELEASE}} cloud deploy {{FLAGS}}

# Check deployment status
status:
    {{CLI}} cloud status

# ══════════════════════════════════════════════════════════════════════════════
# MCP & BUILD ALL
# ══════════════════════════════════════════════════════════════════════════════

# Build all MCP servers (reads from manifest.yaml files)
build-mcp:
    DATABASE_URL="$(just _db-url)" {{CLI}} build mcp --release

# Build everything for deployment (Rust binary + MCP servers + web assets)
build-all:
    just build --release
    just build-mcp
    just web-build
    {{CLI_RELEASE}} infra jobs run publish_pipeline
    @echo "All components built"

# ══════════════════════════════════════════════════════════════════════════════
# WEB ASSETS & PUBLISHING
# ══════════════════════════════════════════════════════════════════════════════

# Copy web assets to dist (CSS, JS, images)
web-assets:
    {{CLI}} infra jobs run copy_extension_assets

# Publish: compile templates, bundle CSS/JS, copy assets, prerender content
publish:
    {{CLI}} infra jobs run publish_pipeline

# Build web assets only (templates + CSS + JS + copy to dist)
web-build:
    {{CLI}} infra jobs run bundle_admin_css
    {{CLI}} infra jobs run bundle_css
    {{CLI}} infra jobs run copy_extension_assets

# ══════════════════════════════════════════════════════════════════════════════
# DOCKER
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

# ══════════════════════════════════════════════════════════════════════════════
# ADMIN & PLUGINS
# ══════════════════════════════════════════════════════════════════════════════

# Generate WebAuthn setup token for admin user
webauthn-admin EMAIL:
    {{CLI}} admin users webauthn generate-setup-token --email "{{EMAIL}}"

# Generate plugin output
marketplace:
    {{CLI}} core plugins generate

# Update Anthropic official plugins from vendor submodule and reimport
update-anthropic-plugins:
    git submodule update --remote vendor/knowledge-work-plugins
    {{CLI}} infra jobs run import_anthropic_plugins

# ══════════════════════════════════════════════════════════════════════════════
# BENCHMARKS
# ══════════════════════════════════════════════════════════════════════════════

# Benchmark governance endpoint (requires hey: curl -sL https://hey-release.s3.us-east-2.amazonaws.com/hey_linux_amd64 -o /tmp/hey && chmod +x /tmp/hey)
benchmark REQUESTS="200" CONCURRENCY="100":
    #!/usr/bin/env bash
    set -e
    HEY="/tmp/hey"
    if [[ ! -x "$HEY" ]]; then
        echo "Installing hey..."
        curl -sL https://hey-release.s3.us-east-2.amazonaws.com/hey_linux_amd64 -o "$HEY" && chmod +x "$HEY"
    fi
    TOKEN_FILE="demo/.token"
    if [[ ! -f "$TOKEN_FILE" ]]; then
        echo "ERROR: No token. Run: ./demo/00-preflight.sh" >&2
        exit 1
    fi
    TOKEN=$(cat "$TOKEN_FILE")
    echo "Governance endpoint: {{REQUESTS}} requests, {{CONCURRENCY}} concurrent"
    echo ""
    "$HEY" -n {{REQUESTS}} -c {{CONCURRENCY}} -m POST \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"hook_event_name":"PreToolUse","tool_name":"Read","agent_id":"developer_agent","session_id":"bench","tool_input":{"file_path":"/src/main.rs"}}' \
        "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo"
