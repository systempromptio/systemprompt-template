# SystemPrompt Template - Development Commands
# All commands use the core CLI via the configured CARGO_TARGET_DIR

set dotenv-load

# CLI binary paths - all binaries in CARGO_TARGET_DIR
CLI := env_var_or_default("CARGO_TARGET_DIR", "target") + "/debug/systemprompt"
CLI_RELEASE := env_var_or_default("CARGO_TARGET_DIR", "target") + "/release/systemprompt"

default:
    @just --list

# ============================================================================
# SETUP
# ============================================================================

# First-time setup (clone with --recursive, then run this)
setup:
    #!/usr/bin/env bash
    set -e
    export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:?CARGO_TARGET_DIR must be set}"
    export SYSTEMPROMPT_CORE_PATH="${SYSTEMPROMPT_CORE_PATH:?SYSTEMPROMPT_CORE_PATH must be set}"
    echo "🔧 Building core binary..."
    cargo build --manifest-path=core/Cargo.toml --bin systemprompt
    echo "🔧 Building workspace..."
    cargo build --workspace
    echo "📊 Running database migrations..."
    {{CLI}} services db migrate
    echo "🌐 Building web assets..."
    {{CLI}} build web build
    echo ""
    echo "✅ Setup complete! Run 'just start' to start the server."

# ============================================================================
# BUILD & RUN
# ============================================================================

# Build debug binaries (add --web to also build web assets)
build *FLAGS:
    #!/usr/bin/env bash
    set -e
    cargo build --manifest-path=core/Cargo.toml --bin systemprompt
    cargo build --workspace
    just mcp-build-submodules
    if [[ "{{FLAGS}}" == *"--web"* ]]; then
        just build-web
    fi

# Build MCP server submodules (systemprompt-admin, systemprompt-infrastructure)
mcp-build-submodules:
    #!/usr/bin/env bash
    set -e
    export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:?CARGO_TARGET_DIR must be set}"
    for dir in services/mcp/systemprompt-*/; do
        if [ -f "$dir/Cargo.toml" ] && [ -e "$dir/.git" ]; then
            name=$(basename "$dir")
            echo "🔧 Building MCP submodule: $name"
            cargo build --manifest-path="$dir/Cargo.toml" --bin "$name"
        fi
    done
    echo "✅ MCP submodules built"

# Update MCP server submodules to latest
mcp-update:
    #!/usr/bin/env bash
    set -e
    for dir in services/mcp/systemprompt-*/; do
        if [ -e "$dir/.git" ]; then
            name=$(basename "$dir")
            echo "📥 Updating MCP submodule: $name"
            (cd "$dir" && git fetch origin && git checkout origin/main)
        fi
    done
    echo "✅ MCP submodules updated. Run 'just build' to rebuild."

# Build web assets
build-web:
    {{CLI}} build web build

# Build release binaries
build-release:
    cargo build --manifest-path=core/Cargo.toml --bin systemprompt --release
    cargo build --workspace --release

# Build release web assets
build-release-web:
    {{CLI}} build web build --prod

# Open interactive TUI
systemprompt:
    {{CLI}}

# Start all services (API, agents, MCP servers)
start:
    {{CLI}} services start --skip-web --skip-migrate

# Start with verbose logging
start-debug:
    {{CLI}} services start --debug --skip-migrate

# Stop all services
stop:
    {{CLI}} services stop

# Show status of all services
status:
    {{CLI}} services status

# Restart services
restart:
    {{CLI}} services restart

# Clean up orphaned processes
cleanup:
    {{CLI}} services cleanup

# ============================================================================
# DATABASE
# ============================================================================

# Run database migrations
db-migrate:
    {{CLI}} services db migrate

# Database operations
db *ARGS:
    {{CLI}} services db {{ARGS}}

# ============================================================================
# LOGS & TRACING
# ============================================================================

# Stream logs
logs:
    {{CLI}} logs stream

# AI task trace
ai-trace TASK_ID *ARGS:
    {{CLI}} logs trace ai {{TASK_ID}} {{ARGS}}


# ============================================================================
# CONTENT & SYNC
# ============================================================================

# Sync content from disk to database
sync-content:
    {{CLI}} cloud sync content

# Sync skills
sync-skills:
    {{CLI}} cloud sync skills

# ============================================================================
# CORE MANAGEMENT
# ============================================================================

# Update core submodule to latest
core-update:
    cd core && git fetch origin && git checkout origin/main
    @echo "Core updated. Run 'cargo update' to update Cargo dependencies."

# Update both submodule and Cargo deps
core-sync:
    #!/usr/bin/env bash
    set -e
    echo "Updating core submodule..."
    cd core && git fetch origin && git checkout -- . && git clean -fd && git checkout origin/main
    cd ..
    echo "Updating Cargo dependencies..."
    cargo update
    echo ""
    echo "✅ Core synced. Run 'just build' to rebuild."

# Pin core to specific version (e.g., just core-pin v0.1.0)
core-pin version:
    cd core && git fetch origin && git checkout {{version}}
    @echo "Core pinned to {{version}}. Run 'cargo update' to update dependencies."

# Show current core version
core-version:
    @cd core && git describe --tags --always

# ============================================================================
# UTILITIES
# ============================================================================

# Run tests
test:
    cargo test --workspace

# Run clippy lints
lint:
    cargo clippy --workspace -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Clean build artifacts
clean:
    cargo clean

# Run SystemPrompt Cloud configuration wizard
config:
    {{CLI}} cloud config

# ============================================================================
# CLOUD DEPLOYMENT
# ============================================================================

# Login to SystemPrompt Cloud (environment: production or sandbox)
login environment="production":
    {{CLI}} cloud login {{environment}}

# Logout from SystemPrompt Cloud
logout:
    {{CLI}} cloud logout

# Link this project to a cloud tenant
cloud-setup:
    {{CLI}} cloud setup

# Deploy to SystemPrompt Cloud
cloud-deploy:
    {{CLI}} cloud deploy

# Check cloud deployment status
cloud-status:
    {{CLI}} cloud status

# Show cloud configuration
cloud-config:
    {{CLI}} cloud config

# ============================================================================
# CONTAINER REGISTRY
# ============================================================================

# Build and push template image to GitHub Container Registry
ghcr-push:
    #!/usr/bin/env bash
    set -e
    echo "🔧 Building release binary..."
    cargo build --release --manifest-path=core/Cargo.toml --bin systemprompt
    # Build workspace only if it has actual crate members (skip for empty template)
    # Check if members array contains actual crate paths (not just comments)
    if grep -Pzo 'members\s*=\s*\[\s*#[^\]]*\]' Cargo.toml >/dev/null 2>&1 || grep -q 'members = \[\s*\]' Cargo.toml 2>/dev/null; then
        echo "ℹ️  Skipping empty workspace build (no crate members)"
    else
        echo "🔧 Building workspace crates..."
        cargo build --workspace --release
    fi
    echo "🌐 Building web assets..."
    SYSTEMPROMPT_WEB_CONFIG_PATH="$(pwd)/services/web/config.yml" \
    SYSTEMPROMPT_WEB_METADATA_PATH="$(pwd)/services/web/metadata.yml" \
    npm run build --prefix core/web
    echo "📦 Staging artifacts..."
    mkdir -p infrastructure/build-context/release
    cp {{CLI_RELEASE}} infrastructure/build-context/release/
    echo "🐳 Building Docker image..."
    docker build -f infrastructure/docker/app.Dockerfile -t ghcr.io/systempromptio/systemprompt-template:latest .
    echo "🚀 Pushing to GitHub Container Registry..."
    docker push ghcr.io/systempromptio/systemprompt-template:latest
    echo "✅ Done! Image pushed to ghcr.io/systempromptio/systemprompt-template:latest"
