---
title: "CLI Extensions"
description: "Build standalone CLI extensions for custom commands, external integrations, and utility tools that agents can execute via subprocess."
author: "SystemPrompt Team"
slug: "extensions/domains/cli"
keywords: "cli, extensions, commands, integrations, standalone"
image: "/files/images/docs/extensions-cli.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# CLI Extensions

CLI extensions are standalone binaries that provide custom command-line tools. Like MCP servers, they run as separate processes, not compiled into the main binary. Unlike MCP servers, CLI extensions are invoked directly via shell execution rather than through the MCP protocol.

## CLI vs MCP: When to Use Which

Both CLI and MCP extensions are standalone binaries, but they serve different purposes:

| Aspect | CLI Extension | MCP Extension |
|--------|---------------|---------------|
| **Invocation** | Shell execution (`./binary args`) | MCP protocol over HTTP |
| **Communication** | stdout/stderr, exit codes | JSON-RPC, structured responses |
| **Use case** | One-off commands, scripts | Persistent tools for AI agents |
| **State** | Stateless per invocation | Can maintain session state |
| **Discovery** | Known binary path | Tool listing via protocol |

**Choose CLI extensions when:**
- Agents execute commands via subprocess
- You need shell-compatible tools for scripts
- The operation is a one-shot command with text output
- Integration with existing shell workflows

**Choose MCP extensions when:**
- AI agents need structured tool access
- Tools require persistent state or sessions
- You want protocol-level discovery and schemas
- Claude Desktop or similar clients are the primary consumers

For detailed MCP server implementation, see [MCP Extensions](/documentation/extensions/domains/mcp).

## Project Structure

```
extensions/cli/discord/
├── Cargo.toml
└── src/
    ├── main.rs         # Entry point with clap
    ├── config.rs       # Configuration loading
    └── discord.rs      # Discord API service
```

## Entry Point

CLI extensions use clap for argument parsing:

```rust
mod config;
mod discord;

use clap::{Parser, Subcommand};
use colored::Colorize;

use crate::config::DiscordConfigValidated;
use crate::discord::DiscordService;

#[derive(Parser)]
#[command(name = "systemprompt-discord")]
#[command(about = "Send messages via Discord")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Send {
        message: String,

        #[arg(long, short)]
        channel: Option<String>,

        #[arg(long, short)]
        user: Option<String>,
    },

    Test,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Send { message, channel, user } => {
            send_message(&message, channel, user).await?;
        }
        Commands::Test => {
            test_connection().await?;
        }
    }

    Ok(())
}
```

## Command Implementation

Implement each subcommand with clear output:

```rust
async fn send_message(
    message: &str,
    channel: Option<String>,
    user: Option<String>,
) -> anyhow::Result<()> {
    let config = DiscordConfigValidated::load_from_default_paths()?;
    let service = DiscordService::new(config)?;

    let result = if let Some(channel_id) = channel {
        print_info(&format!("Sending to channel {}...", channel_id));
        service.send_channel_message(&channel_id, message).await?
    } else if let Some(user_id) = user {
        print_info(&format!("Sending DM to user {}...", user_id));
        service.send_dm(&user_id, message).await?
    } else {
        print_info("Sending to default target...");
        service.send_to_default(message).await?
    };

    print_success(&format!(
        "Message sent! (ID: {}, Channel: {})",
        result.id, result.channel_id
    ));

    Ok(())
}

async fn test_connection() -> anyhow::Result<()> {
    print_info("Loading Discord configuration...");
    let config = DiscordConfigValidated::load_from_default_paths()?;
    print_success("Configuration loaded");

    print_info("Testing Discord API connection...");
    let service = DiscordService::new(config)?;

    match service.test_connection().await {
        Ok(bot_name) => {
            print_success(&format!("Connected as: {}", bot_name));
            Ok(())
        }
        Err(e) => {
            print_error(&format!("Connection failed: {}", e));
            Err(e)
        }
    }
}

fn print_success(message: &str) {
    println!("{} {}", "OK".green(), message);
}

fn print_error(message: &str) {
    eprintln!("{} {}", "ERR".red(), message);
}

fn print_info(message: &str) {
    println!("{} {}", "->".blue(), message);
}
```

## Configuration

Load configuration from files or environment variables:

```rust
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct DiscordConfig {
    pub bot_token: String,
    pub default_channel: Option<String>,
    pub default_user: Option<String>,
}

pub struct DiscordConfigValidated {
    pub bot_token: String,
    pub default_channel: Option<String>,
    pub default_user: Option<String>,
}

impl DiscordConfigValidated {
    pub fn load_from_default_paths() -> anyhow::Result<Self> {
        // Try environment variables first
        if let Ok(token) = std::env::var("DISCORD_BOT_TOKEN") {
            return Ok(Self {
                bot_token: token,
                default_channel: std::env::var("DISCORD_CHANNEL").ok(),
                default_user: std::env::var("DISCORD_USER").ok(),
            });
        }

        // Fall back to config file
        let config_path = Self::config_path()?;
        let content = std::fs::read_to_string(&config_path)?;
        let config: DiscordConfig = serde_yaml::from_str(&content)?;

        Ok(Self {
            bot_token: config.bot_token,
            default_channel: config.default_channel,
            default_user: config.default_user,
        })
    }

    fn config_path() -> anyhow::Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".config/systemprompt/discord.yaml"))
    }
}
```

## Service Implementation

Implement the external service integration:

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct DiscordService {
    client: Client,
    config: DiscordConfigValidated,
}

#[derive(Serialize)]
struct CreateMessage {
    content: String,
}

#[derive(Deserialize)]
pub struct MessageResponse {
    pub id: String,
    pub channel_id: String,
}

impl DiscordService {
    pub fn new(config: DiscordConfigValidated) -> anyhow::Result<Self> {
        let client = Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    "Authorization",
                    format!("Bot {}", config.bot_token).parse()?,
                );
                headers
            })
            .build()?;

        Ok(Self { client, config })
    }

    pub async fn send_channel_message(
        &self,
        channel_id: &str,
        content: &str,
    ) -> anyhow::Result<MessageResponse> {
        let url = format!(
            "https://discord.com/api/v10/channels/{}/messages",
            channel_id
        );

        let response = self.client
            .post(&url)
            .json(&CreateMessage { content: content.to_string() })
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn test_connection(&self) -> anyhow::Result<String> {
        let response: serde_json::Value = self.client
            .get("https://discord.com/api/v10/users/@me")
            .send()
            .await?
            .json()
            .await?;

        Ok(response["username"].as_str().unwrap_or("Unknown").to_string())
    }
}
```

## Cargo Configuration

```toml
[package]
name = "systemprompt-discord"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "systemprompt-discord"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
anyhow = "1"
colored = "2"
dirs = "5"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

## Building

```bash
# Build CLI extension
cargo build --release -p systemprompt-discord

# Run directly
./target/release/systemprompt-discord test

# Run with arguments
./target/release/systemprompt-discord send "Hello from CLI" --channel 123456789
```

## Agent Integration

Agents can invoke CLI extensions via subprocess:

```bash
# From an agent or script
systemprompt-discord send "Deployment complete" --channel "$NOTIFY_CHANNEL"
```

Exit codes indicate success (0) or failure (non-zero), and stdout/stderr provide human-readable output.

## Integration Patterns

CLI extensions are useful for:

- **Notifications** - Send alerts to Slack, Discord, email
- **Data sync** - Import/export between systems
- **External APIs** - Call third-party services
- **Automation** - CI/CD integration, scheduled tasks
- **Admin tools** - Database utilities, debugging commands