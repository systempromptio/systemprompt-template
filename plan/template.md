# Template Justfile Cleanup

Massively simplify the justfile to only essential commands.

## Current: 272 lines, ~40 commands

## Target: ~50 lines, 3 commands

```just
# SystemPrompt Template
set dotenv-load

CLI := env_var_or_default("CARGO_TARGET_DIR", "target") + "/debug/systemprompt"

default:
    @just --list

# ============================================================================
# ESSENTIALS
# ============================================================================

# Start local development server
start:
    {{CLI}} services start --skip-web --skip-migrate

# Deploy to cloud (assumes artifacts are built)
deploy:
    #!/usr/bin/env bash
    set -e

    # Validate staged artifacts exist
    if [ ! -f "infrastructure/build-context/release/systemprompt" ]; then
        echo "❌ Release binary not found. Build with:"
        echo "   cargo build --release --manifest-path=core/Cargo.toml --bin systemprompt"
        echo "   mkdir -p infrastructure/build-context/release"
        echo "   cp core/target/release/systemprompt infrastructure/build-context/release/"
        exit 1
    fi

    if [ ! -d "core/web/dist" ]; then
        echo "❌ Web assets not found. Build with:"
        echo "   npm run build --prefix core/web"
        exit 1
    fi

    echo "📦 Building Docker image..."
    TAG="deploy-$(date +%s)-$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
    APP_ID=$(jq -r '.app_id // empty' ~/.systemprompt/credentials.json 2>/dev/null)

    if [ -z "$APP_ID" ]; then
        echo "❌ Not logged in. Run: {{CLI}} cloud login"
        exit 1
    fi

    IMAGE="registry.fly.io/${APP_ID}:${TAG}"
    docker build -f infrastructure/docker/app.Dockerfile -t "${IMAGE}" .

    echo "🚀 Deploying..."
    {{CLI}} cloud deploy --skip-build --tag "${TAG}"

# CLI passthrough (interactive TUI or any subcommand)
systemprompt *ARGS:
    {{CLI}} {{ARGS}}
```

## What Gets Removed

| Removed Command | Use Instead |
|----------------|-------------|
| `setup` | `systemprompt services db migrate && npm run build --prefix core/web` |
| `build`, `build-web`, `build-release` | `cargo build ...` directly |
| `mcp-build-submodules`, `mcp-update` | `systemprompt agents mcp ...` |
| `stop`, `status`, `restart`, `cleanup` | `systemprompt services ...` |
| `db-migrate`, `db` | `systemprompt services db ...` |
| `logs`, `ai-trace` | `systemprompt logs ...` |
| `sync-content`, `sync-skills` | `systemprompt cloud sync ...` |
| `core-update`, `core-sync`, `core-pin`, `core-version` | `cd core && git ...` |
| `test`, `lint`, `fmt`, `clean` | `cargo test/clippy/fmt/clean` directly |
| `config` | `systemprompt cloud config` |
| `login`, `logout`, `cloud-setup`, `cloud-deploy`, `cloud-status`, `cloud-config` | `systemprompt cloud ...` |
| `ghcr-push` | Remove (not needed for Fly.io) |

## Usage

```bash
# Local development
just start                    # Start services
just systemprompt logs stream # View logs
just systemprompt services stop # Stop services

# Deploy to cloud
cargo build --release --manifest-path=core/Cargo.toml --bin systemprompt
npm run build --prefix core/web
mkdir -p infrastructure/build-context/release
cp core/target/release/systemprompt infrastructure/build-context/release/
just deploy

# Any CLI command
just systemprompt cloud login
just systemprompt cloud config
just systemprompt services status
```

## Rationale

1. **`just systemprompt *ARGS`** is a universal passthrough - any CLI command works
2. **`just start`** is the most common dev command - deserves shortcut
3. **`just deploy`** encapsulates the deploy pipeline with validation
4. Everything else: use the CLI directly via `just systemprompt <command>`
