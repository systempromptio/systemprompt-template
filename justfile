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

# Update core submodule to latest
core-update:
    cd core && git fetch origin && git checkout origin/main
    @echo "Core updated. Run 'cargo update' to update Cargo dependencies too."

# Update both submodule and Cargo deps
core-sync:
    ./infrastructure/scripts/core-sync.sh

# Pin core to specific version (e.g., just core-pin v0.1.0)
core-pin version:
    cd core && git fetch origin && git checkout {{version}}
    @echo "Core pinned to {{version}}. Update Cargo.toml to match if using tags."

# Show current core version
core-version:
    @cd core && git describe --tags --always

# Show current core status (legacy alias)
core-status:
    @echo "Core submodule status:"
    @cd core && git describe --tags --always
    @cd core && git log --oneline -1

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
