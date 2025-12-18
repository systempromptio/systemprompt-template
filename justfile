# SystemPrompt Template - Development Commands
# All commands use the core CLI (./core/target/debug/systemprompt)

set dotenv-load

default:
    @just --list

# ============================================================================
# SETUP
# ============================================================================

# First-time setup (clone with --recursive, then run this)
setup:
    #!/usr/bin/env bash
    set -e
    echo "🔧 Building core binary..."
    cargo build --manifest-path=core/Cargo.toml --bin systemprompt
    echo "🔧 Building workspace..."
    cargo build --workspace
    echo "📊 Running database migrations..."
    ./core/target/debug/systemprompt db migrate
    echo "🌐 Building web assets..."
    ./core/target/debug/systemprompt web build
    echo ""
    echo "✅ Setup complete! Run 'just start' to start the server."

# ============================================================================
# BUILD & RUN
# ============================================================================

# Build debug binaries (Rust only)
build:
    cargo build --manifest-path=core/Cargo.toml --bin systemprompt
    cargo build --workspace

# Build web assets
build-web:
    ./core/target/debug/systemprompt web build

# Build release binaries
build-release:
    cargo build --manifest-path=core/Cargo.toml --bin systemprompt --release
    cargo build --workspace --release

# Build release web assets
build-release-web:
    ./core/target/debug/systemprompt web build --prod

# Open interactive TUI (starts services in background first)
systemprompt:
    ./core/target/debug/systemprompt interactive

# Start all services (API, agents, MCP servers)
start:
    ./core/target/debug/systemprompt start --skip-web

# Start with verbose logging
start-debug:
    ./core/target/debug/systemprompt start --debug

# Stop all services
stop:
    ./core/target/debug/systemprompt stop

# Show status of all services
status:
    ./core/target/debug/systemprompt status

# Restart services
restart:
    ./core/target/debug/systemprompt restart

# Clean up orphaned processes
cleanup:
    ./core/target/debug/systemprompt cleanup-services

# ============================================================================
# DATABASE
# ============================================================================

# Run database migrations
db-migrate:
    ./core/target/debug/systemprompt db migrate

# Database operations
db *ARGS:
    ./core/target/debug/systemprompt db {{ARGS}}

ai-trace TASK_ID *ARGS:
    core/target/debug/systemprompt ai-trace {{TASK_ID}} {{ARGS}}


# ============================================================================
# CONTENT & SYNC
# ============================================================================

# Sync content from disk to database
sync-content:
    ./core/target/debug/systemprompt sync content

# Sync skills
sync-skills:
    ./core/target/debug/systemprompt sync skills

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

# Show system configuration
config:
    ./core/target/debug/systemprompt config env

# Stream logs
logs:
    ./core/target/debug/systemprompt logs

# ============================================================================
# CLOUD DEPLOYMENT
# ============================================================================

# Login to SystemPrompt Cloud (environment: production or sandbox)
login environment="production":
    ./core/target/debug/systemprompt cloud login {{environment}}

# Logout from SystemPrompt Cloud
logout:
    ./core/target/debug/systemprompt cloud logout

# Link this project to a cloud tenant
cloud-setup:
    ./core/target/debug/systemprompt cloud setup

# Deploy to SystemPrompt Cloud
cloud-deploy:
    ./core/target/debug/systemprompt cloud deploy

# Check cloud deployment status
cloud-status:
    ./core/target/debug/systemprompt cloud status

# Show cloud configuration
cloud-config:
    ./core/target/debug/systemprompt cloud config

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
    cp core/target/release/systemprompt infrastructure/build-context/release/
    echo "🐳 Building Docker image..."
    docker build -f infrastructure/docker/app.Dockerfile -t ghcr.io/systempromptio/systemprompt-template:latest .
    echo "🚀 Pushing to GitHub Container Registry..."
    docker push ghcr.io/systempromptio/systemprompt-template:latest
    echo "✅ Done! Image pushed to ghcr.io/systempromptio/systemprompt-template:latest"
