# SystemPrompt Template - Development Commands
# All commands use the core CLI (./core/target/debug/systemprompt)

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

# Start all services (API, agents, MCP servers)
start:
    ./core/target/debug/systemprompt start

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
