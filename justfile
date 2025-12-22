# SystemPrompt Template
set dotenv-load

CLI := "target/debug/systemprompt"
RELEASE_DIR := "target/release"

default:
    @just --list

# Build workspace (use --release for release build)
build *FLAGS:
    #!/usr/bin/env bash
    set -e

    # Build core CLI
    cargo build --manifest-path=core/Cargo.toml {{FLAGS}}

    # Build MCP extensions (if any exist)
    for ext in extensions/mcp/*/; do
        if [ -f "$ext/Cargo.toml" ]; then
            echo "Building MCP extension: $ext"
            cargo build --manifest-path="$ext/Cargo.toml" {{FLAGS}}
        fi
    done

    # Collect MCP binaries to central target directory
    if [[ "{{FLAGS}}" == *"--release"* ]]; then
        TARGET_DIR="target/release"
        BUILD_TYPE="release"
    else
        TARGET_DIR="target/debug"
        BUILD_TYPE="debug"
    fi

    mkdir -p "$TARGET_DIR"
    for ext in extensions/mcp/*/; do
        if [ -d "$ext/target/$BUILD_TYPE" ]; then
            for bin in "$ext/target/$BUILD_TYPE/"*; do
                if [ -f "$bin" ] && [ -x "$bin" ] && [[ ! "$bin" == *.d ]] && [[ ! "$bin" == *.rlib ]]; then
                    cp "$bin" "$TARGET_DIR/" 2>/dev/null || true
                fi
            done
        fi
    done

# Start local development server
start:
    {{CLI}} services start

# Deploy to cloud
deploy:
    {{CLI}} cloud deploy

# CLI passthrough
systemprompt *ARGS:
    {{CLI}} {{ARGS}}
