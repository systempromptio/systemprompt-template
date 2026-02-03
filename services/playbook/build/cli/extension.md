---
title: "Create CLI Extension"
description: "Create a standalone CLI extension binary."
author: "SystemPrompt"
slug: "build-05-cli-extensions-create-cli-extension"
keywords: "cli, extension, plugin, binary"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Create CLI Extension

Create a standalone CLI extension binary. Reference: `extensions/cli/discord/` for working example.

> **Help**: `{ "command": "core playbooks show build_create-cli-extension" }`

---

## Core Principle

CLI extensions are standalone binaries discovered via `manifest.yaml`. They run as separate processes launched by `systemprompt plugins run <name>`.

---

## Structure

```
extensions/cli/{name}/
├── Cargo.toml
├── manifest.yaml
└── src/
    └── main.rs
```

---

## manifest.yaml

```yaml
extension:
  type: cli
  name: my-extension
  binary: systemprompt-my-extension
  description: "What this extension does"
  enabled: true
  commands:
    - name: do-something
      description: "Does something useful"
    - name: another-command
      description: "Another useful command"
```

---

## Cargo.toml

```toml
[package]
name = "systemprompt-my-extension"
version = "1.0.0"
edition = "2021"

[[bin]]
name = "systemprompt-my-extension"
path = "src/main.rs"

[dependencies]
clap = { version = "4.4", features = ["derive"] }
tokio = { version = "1.47", features = ["full"] }
anyhow = "1.0"
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

---

## Main Entry Point

File: `src/main.rs`. See `extensions/cli/discord/src/main.rs:1-50` for reference.

```rust
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "systemprompt-my-extension")]
#[command(about = "My CLI extension description")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    DoSomething {
        input: String,
        #[arg(long, short)]
        verbose: bool,
    },
    AnotherCommand {
        #[arg(long)]
        option: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::DoSomething { input, verbose } => {
            do_something(&input, verbose).await?;
        }
        Commands::AnotherCommand { option } => {
            another_command(option).await?;
        }
    }

    Ok(())
}

async fn do_something(input: &str, verbose: bool) -> anyhow::Result<()> {
    println!("Processing: {}", input);
    Ok(())
}

async fn another_command(option: Option<String>) -> anyhow::Result<()> {
    Ok(())
}
```

---

## Environment Variables

The main CLI passes these environment variables:

| Variable | Description |
|----------|-------------|
| `SYSTEMPROMPT_PROFILE` | Path to active profile directory |
| `DATABASE_URL` | Database connection string |

```rust
fn get_profile_path() -> anyhow::Result<std::path::PathBuf> {
    std::env::var("SYSTEMPROMPT_PROFILE")
        .map(std::path::PathBuf::from)
        .map_err(|_| anyhow::anyhow!("SYSTEMPROMPT_PROFILE not set"))
}

fn get_database_url() -> anyhow::Result<String> {
    std::env::var("DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("DATABASE_URL not set"))
}
```

---

## Configuration

Load extension config from profile directory:

```rust
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct MyConfig {
    api_key: String,
    enabled: bool,
}

fn load_config(profile_path: &Path) -> anyhow::Result<MyConfig> {
    let config_path = profile_path.join("extensions/my-extension.yaml");
    let content = std::fs::read_to_string(&config_path)?;
    let config: MyConfig = serde_yaml::from_str(&content)?;
    Ok(config)
}
```

---

## Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyExtensionError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

impl From<MyExtensionError> for anyhow::Error {
    fn from(err: MyExtensionError) -> Self {
        anyhow::anyhow!(err)
    }
}
```

---

## Workspace Registration

Add to workspace `Cargo.toml`:

```toml
[workspace]
members = [
    "extensions/cli/my-extension",
]
```

---

## Checklist

- [ ] `manifest.yaml` with `type: cli`
- [ ] Package name follows `systemprompt-{name}` pattern
- [ ] Binary name matches `manifest.yaml`
- [ ] Located in `extensions/cli/`
- [ ] Uses clap with derive macros
- [ ] Initializes tracing for logging
- [ ] Returns `anyhow::Result<()>`
- [ ] Added to workspace members

---

## Code Quality

| Metric | Limit |
|--------|-------|
| File length | 300 lines |
| Function length | 75 lines |
| No `unwrap()` | Use `?` or `ok_or_else()` |
| No inline comments | Code documents itself |

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Extension not found | Check `manifest.yaml` exists |
| Binary not found | Run `cargo build -p systemprompt-{name}` |
| Environment error | Verify CLI passes variables |

---

## Quick Reference

| Task | Command |
|------|---------|
| Build | `cargo build -p systemprompt-{name}` |
| Run directly | `cargo run -p systemprompt-{name} -- <args>` |
| Run via CLI | `systemprompt plugins run {name} <args>` |
| List extensions | `systemprompt plugins list --type cli` |
| Lint | `cargo clippy -p systemprompt-{name} -- -D warnings` |

---

## Related

-> See [Rust Standards](../06-standards/rust-standards.md) for code style
-> See [CLI Extension](../../documentation/extensions/domains/cli.md) for domain reference