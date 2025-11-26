# SystemPrompt Template - Development Commands

default:
    @just --list

# ============================================================================
# SETUP
# ============================================================================

# First-time setup (run this after cloning)
setup:
    ./infrastructure/scripts/setup-dev.sh

# ============================================================================
# BUILD & RUN
# ============================================================================

# Build debug binaries
build:
    ./infrastructure/scripts/build.sh debug

# Build release binaries
build-release:
    ./infrastructure/scripts/build.sh release

# Start the API server
start:
    #!/usr/bin/env bash
    if [ -f .env.local ]; then set -a; source .env.local; set +a; fi
    if [ -f .env.secrets ]; then set -a; source .env.secrets; set +a; fi
    ./core/target/debug/systemprompt serve api --foreground

# Start with verbose logging
start-debug:
    #!/usr/bin/env bash
    export RUST_LOG=debug
    if [ -f .env.local ]; then set -a; source .env.local; set +a; fi
    if [ -f .env.secrets ]; then set -a; source .env.secrets; set +a; fi
    ./core/target/debug/systemprompt serve api --foreground

# ============================================================================
# DATABASE
# ============================================================================

# Run database migrations
db-migrate:
    #!/usr/bin/env bash
    if [ -f .env.local ]; then set -a; source .env.local; set +a; fi
    if [ -f .env.secrets ]; then set -a; source .env.secrets; set +a; fi
    ./core/target/debug/systemprompt db migrate

# ============================================================================
# CORE MANAGEMENT
# ============================================================================

# Sync core subtree from upstream (READ-ONLY updates only)
core-sync:
    ./infrastructure/scripts/core-sync.sh

# Show current core version
core-status:
    @echo "Core subtree status:"
    @git log --oneline -1 -- core/

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
