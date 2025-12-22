# SystemPrompt Template
set dotenv-load

CLI := "target/debug/systemprompt"

default:
    @just --list

# Build workspace (use --release for release build)
build *FLAGS:
    #!/usr/bin/env bash
    set -e
    cargo build --manifest-path=core/Cargo.toml {{FLAGS}}
    if [[ "{{FLAGS}}" == *"--release"* ]]; then
        mkdir -p target/release
        cp core/target/release/systemprompt target/release/
    fi

# Start local development server
start:
    {{CLI}} services start

# Deploy to cloud
deploy:
    {{CLI}} cloud deploy

# CLI passthrough
systemprompt *ARGS:
    {{CLI}} {{ARGS}}
