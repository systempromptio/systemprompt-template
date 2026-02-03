mod config;
mod discord;

use std::env;

use anyhow::Context;
use clap::{Parser, Subcommand};
use colored::Colorize;
use serde_json::json;
use systemprompt::models::{ProfileBootstrap, SecretsBootstrap};
use tracing_subscriber::EnvFilter;

use crate::config::DiscordConfigValidated;
use crate::discord::DiscordService;

fn is_json_output() -> bool {
    env::var("SYSTEMPROMPT_OUTPUT_FORMAT")
        .map(|v| v.eq_ignore_ascii_case("json"))
        .unwrap_or(false)
}

#[derive(Parser)]
#[command(name = "systemprompt-discord")]
#[command(about = "Send messages via Discord")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true, hide = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    Send {
        message: String,

        #[arg(long, short)]
        channel: Option<String>,

        #[arg(long, short)]
        user: Option<String>,

        #[arg(long, short)]
        reply_to: Option<String>,
    },

    Test,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    ProfileBootstrap::init().context("Failed to initialize profile")?;
    SecretsBootstrap::init().context("Failed to initialize secrets")?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Send {
            message,
            channel,
            user,
            reply_to,
        } => {
            send_message(&message, channel, user, reply_to).await?;
        }
        Commands::Test => {
            test_connection().await?;
        }
    }

    Ok(())
}

async fn send_message(
    message: &str,
    channel: Option<String>,
    user: Option<String>,
    reply_to: Option<String>,
) -> anyhow::Result<()> {
    let config = DiscordConfigValidated::load_from_default_paths()?;
    let service = DiscordService::new(config)?;
    let json_output = is_json_output();

    let result = if let Some(channel_id) = channel {
        if let Some(ref message_id) = reply_to {
            if !json_output {
                print_info(&format!(
                    "Replying to message {} in channel {}...",
                    message_id, channel_id
                ));
            }
            service
                .reply_to_message(&channel_id, message_id, message)
                .await?
        } else {
            if !json_output {
                print_info(&format!("Sending to channel {}...", channel_id));
            }
            service.send_channel_message(&channel_id, message).await?
        }
    } else if let Some(user_id) = user {
        if !json_output {
            print_info(&format!("Sending DM to user {}...", user_id));
        }
        service.send_dm(&user_id, message).await?
    } else {
        if !json_output {
            print_info("Sending to default target...");
        }
        service.send_to_default(message).await?
    };

    if json_output {
        let output = json!({
            "data": {
                "message_id": result.id.to_string(),
                "channel_id": result.channel_id.to_string(),
                "status": "sent"
            },
            "artifact_type": "text",
            "title": "Discord Message Sent"
        });
        println!("{}", serde_json::to_string(&output)?);
    } else {
        print_success(&format!(
            "Message sent! (ID: {}, Channel: {})",
            result.id, result.channel_id
        ));
    }

    Ok(())
}

async fn test_connection() -> anyhow::Result<()> {
    let json_output = is_json_output();

    if !json_output {
        print_info("Loading Discord configuration...");
    }
    let config = DiscordConfigValidated::load_from_default_paths()?;
    if !json_output {
        print_success("Configuration loaded");
        print_info("Testing Discord API connection...");
    }

    let service = DiscordService::new(config)?;

    match service.test_connection().await {
        Ok(bot_name) => {
            if json_output {
                let output = json!({
                    "data": {
                        "bot_name": bot_name,
                        "status": "connected"
                    },
                    "artifact_type": "text",
                    "title": "Discord Connection Test"
                });
                println!("{}", serde_json::to_string(&output)?);
            } else {
                print_success(&format!("Connected as: {}", bot_name));
            }
            Ok(())
        }
        Err(e) => {
            if json_output {
                let output = json!({
                    "data": {
                        "error": e.to_string(),
                        "status": "failed"
                    },
                    "artifact_type": "text",
                    "title": "Discord Connection Test Failed"
                });
                println!("{}", serde_json::to_string(&output)?);
            } else {
                print_error(&format!("Connection failed: {}", e));
            }
            Err(e)
        }
    }
}

fn print_success(message: &str) {
    println!("{} {}", "✓".green(), message);
}

fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red(), message);
}

fn print_info(message: &str) {
    println!("{} {}", "→".blue(), message);
}
