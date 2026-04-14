# systemprompt-template
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
tenant:
    {{CLI}} cloud tenant

# Set up a local-only profile + Docker Postgres (no cloud, no login required).
# Pass keys as positional args, or leave blank to be prompted interactively:
#   just setup-local sk-ant-... sk-... AIza...
setup-local ANTHROPIC_KEY="" OPENAI_KEY="" GEMINI_KEY="":
    #!/usr/bin/env bash
    set -euo pipefail
    ROOT="{{justfile_directory()}}"
    PROFILE_DIR="$ROOT/.systemprompt/profiles/local"
    DOCKER_DIR="$ROOT/.systemprompt/docker"
    ANTHROPIC_KEY="{{ANTHROPIC_KEY}}"
    OPENAI_KEY="{{OPENAI_KEY}}"
    GEMINI_KEY="{{GEMINI_KEY}}"
    if [ -z "$ANTHROPIC_KEY" ] && [ -z "$OPENAI_KEY" ] && [ -z "$GEMINI_KEY" ]; then
        echo ""
        echo "================================================================"
        echo "  setup-local needs at least one AI provider API key"
        echo "================================================================"
        echo ""
        echo "  The marketplace MCP server (skill-manager) will not start"
        echo "  without at least one of: Anthropic, OpenAI, or Gemini."
        echo "  Keys are written into the local profile at:"
        echo "    .systemprompt/profiles/local/secrets.json"
        echo ""
        echo "  Press Enter to skip a provider."
        echo ""
        if [ ! -t 0 ]; then
            echo "  ERROR: not running on a TTY — pass keys as recipe args:"
            echo "    just setup-local <anthropic_key> <openai_key> <gemini_key>"
            exit 1
        fi
        read -r -p "  Anthropic API key: " ANTHROPIC_KEY || true
        read -r -p "  OpenAI API key:    " OPENAI_KEY || true
        read -r -p "  Gemini API key:    " GEMINI_KEY || true
        if [ -z "$ANTHROPIC_KEY" ] && [ -z "$OPENAI_KEY" ] && [ -z "$GEMINI_KEY" ]; then
            echo ""
            echo "  ERROR: no key provided. Aborting."
            exit 1
        fi
    fi
    if [ ! -x target/debug/systemprompt ] && [ ! -x target/release/systemprompt ]; then
        echo "Building debug binary..."
        just build
    fi
    mkdir -p "$PROFILE_DIR" "$DOCKER_DIR"
    if [ ! -f "$DOCKER_DIR/local.yaml" ]; then
        echo "Writing Docker compose for local Postgres..."
        cat > "$DOCKER_DIR/local.yaml" <<'YAML'
    services:
      postgres:
        image: postgres:18-alpine
        container_name: systemprompt-postgres-local
        restart: unless-stopped
        environment:
          POSTGRES_USER: systemprompt
          POSTGRES_PASSWORD: 123
          POSTGRES_DB: systemprompt
        ports:
          - "5432:5432"
        volumes:
          - postgres_data:/var/lib/postgresql
        healthcheck:
          test: ["CMD-SHELL", "pg_isready -U systemprompt -d systemprompt"]
          interval: 5s
          timeout: 5s
          retries: 5
    volumes:
      postgres_data:
        name: systemprompt-postgres-local-data
    YAML
    fi
    if [ ! -f "$PROFILE_DIR/profile.yaml" ]; then
        echo "Writing local profile.yaml..."
        cat > "$PROFILE_DIR/profile.yaml" <<YAML
    name: local
    display_name: Local Development
    target: local
    site:
      name: systemprompt.io
      github_link: null
    database:
      type: postgres
      external_db_access: false
    server:
      host: 127.0.0.1
      port: 8080
      api_server_url: http://localhost:8080
      api_internal_url: http://localhost:8080
      api_external_url: http://localhost:8080
      use_https: false
      cors_allowed_origins:
      - http://localhost:8080
      - http://localhost:5173
    paths:
      system: $ROOT
      services: $ROOT/services
      bin: $ROOT/target/debug
      web_path: null
      storage: $ROOT/storage
      geoip_database: null
    security:
      jwt_issuer: systemprompt-local
      jwt_access_token_expiration: 2592000
      jwt_refresh_token_expiration: 15552000
      jwt_audiences: [web, api, a2a, mcp]
    rate_limits:
      disabled: true
      oauth_public_per_second: 10
      oauth_auth_per_second: 10
      contexts_per_second: 100
      tasks_per_second: 50
      artifacts_per_second: 50
      agent_registry_per_second: 50
      agents_per_second: 20
      mcp_registry_per_second: 50
      mcp_per_second: 200
      stream_per_second: 100
      content_per_second: 50
      burst_multiplier: 3
      tier_multipliers:
        admin: 10.0
        user: 1.0
        a2a: 5.0
        mcp: 5.0
        service: 5.0
        anon: 0.5
    runtime:
      environment: development
      log_level: verbose
      output_format: text
      no_color: false
      non_interactive: false
    cloud:
      tenant_id: local_dev
      validation: warn
    secrets:
      secrets_path: ./secrets.json
      validation: warn
      source: file
    extensions:
      disabled: []
    YAML
    fi
    if [ ! -f "$PROFILE_DIR/secrets.json" ]; then
        echo "Writing local secrets.json..."
        JWT_SECRET=$(head -c 48 /dev/urandom | base64 | tr -d '+/=' | head -c 64)
        json_field() { if [ -n "${1:-}" ]; then printf '"%s"' "$1"; else printf 'null'; fi; }
        ANTHROPIC_JSON=$(json_field "$ANTHROPIC_KEY")
        OPENAI_JSON=$(json_field "$OPENAI_KEY")
        GEMINI_JSON=$(json_field "$GEMINI_KEY")
        cat > "$PROFILE_DIR/secrets.json" <<JSON
    {
      "jwt_secret": "$JWT_SECRET",
      "database_url": "postgres://systemprompt:123@localhost:5432/systemprompt",
      "anthropic": $ANTHROPIC_JSON,
      "openai": $OPENAI_JSON,
      "gemini": $GEMINI_JSON
    }
    JSON
    fi
    mkdir -p "$ROOT/web/dist"
    echo "Starting local Postgres via Docker..."
    just db-up local
    echo "Publishing assets..."
    just publish
    echo ""
    echo "Local setup complete. Run: just start"

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
