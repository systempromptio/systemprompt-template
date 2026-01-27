# Setup Playbook

Bootstrap a fresh SystemPrompt project.

## Prerequisites

- Rust 1.75+
- Docker

## Steps

```bash
# 1. Build CLI (offline mode - no database yet)
SQLX_OFFLINE=true cargo build --release --manifest-path core/crates/entry/cli/Cargo.toml

# 2. Login (MANUAL - opens browser)
./core/target/release/systemprompt cloud auth login

# 3. Create tenant (interactive - sets up database)
./core/target/release/systemprompt cloud tenant

# 4. Create profile (interactive - configures environment)
./core/target/release/systemprompt cloud profile

# 5. Migrate database
./core/target/release/systemprompt infra db migrate

# 6. Start services
./core/target/release/systemprompt infra services start --all
```

Server available at `http://127.0.0.1:8080`.

## Verification

```bash
./core/target/release/systemprompt infra services status
./core/target/release/systemprompt infra db status
```
