set dotenv-load

# CLI binary selection (release preferred, then debug)
CLI := if path_exists("target/release/systemprompt") == "true" {
    "target/release/systemprompt"
} else if path_exists("target/debug/systemprompt") == "true" {
    "target/debug/systemprompt"
} else {
    "echo 'ERROR: No CLI binary found. Run: just build'"
}

CLI_RELEASE := "target/release/systemprompt"

# Default: pass args to CLI
default *ARGS:
    {{CLI}} {{ARGS}}

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

# Service commands
start:
    {{CLI}} infra services start --profile local

migrate:
    {{CLI}} infra db migrate

# Auth commands
login ENV="production":
    {{CLI}} cloud auth login {{ENV}}

logout:
    {{CLI}} cloud auth logout

whoami:
    {{CLI}} cloud auth whoami

tenant:
    {{CLI}} cloud tenant
    {{CLI}} core skills sync --direction to-db -y

profile:
    {{CLI}} cloud profile

profiles:
    {{CLI}} cloud profile list

# Deploy
deploy *FLAGS:
    just build --release
    {{CLI_RELEASE}} cloud deploy {{FLAGS}}

# Docker commands
docker-build TAG="local":
    docker build -f .systemprompt/Dockerfile -t systemprompt-template:{{TAG}} .

docker-run TAG="local":
    docker run -p 8080:8080 --env-file .env systemprompt-template:{{TAG}}

# Admin commands
webauthn-admin EMAIL:
    {{CLI}} admin users webauthn generate-setup-token --email "{{EMAIL}}"
