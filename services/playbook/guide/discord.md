---
title: "Discord Integration Guide"
description: "Principal guide for Discord integration. Covers sending messages (CLI) and receiving messages (Gateway)."
author: "SystemPrompt"
slug: "guide-discord"
keywords: "discord, bot, gateway, messaging, notifications, webhook"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Discord Integration Guide

Principal guide for integrating Discord with SystemPrompt. Enables bidirectional communication between agents and Discord.

> **Read playbooks**: `systemprompt core playbooks show <playbook_id>`

---

## Overview

SystemPrompt supports two Discord integration modes:

| Mode | Direction | Component | Use Case |
|------|-----------|-----------|----------|
| **CLI** | Outbound | `extensions/cli/discord/` | Send notifications, alerts, status updates |
| **Gateway** | Inbound | `extensions/soul/src/discord/` | Receive messages, trigger agent workflows |

```
+------------------------------------------------------------------+
|                       DISCORD                                      |
|                           |                                        |
|              +------------+------------+                           |
|              |                         |                           |
|              v                         v                           |
|      +-------------+           +-------------+                     |
|      |   GATEWAY   |           |    CLI      |                     |
|      |  (Inbound)  |           | (Outbound)  |                     |
|      +------+------+           +------+------+                     |
|             |                         |                            |
|             v                         |                            |
|      +-------------+                  |                            |
|      |   AGENT     |<-----------------+                            |
|      | (processes) |                                               |
|      +-------------+                                               |
+------------------------------------------------------------------+
```

---

## Prerequisites

Before using Discord integration:

- [ ] Discord bot created at https://discord.com/developers/applications
- [ ] Bot token obtained and configured
- [ ] Bot invited to your Discord server
- [ ] MESSAGE CONTENT intent enabled (for Gateway)

---

## Discord Developer Portal Setup

### 1. Create Application

1. Go to https://discord.com/developers/applications
2. Click **New Application** -> name it -> **Create**
3. Fill in:
   - **Name**: `systemprompt` (or your preferred name)
   - **Description**: `AI agent communication hub for SystemPrompt`
   - **Tags**: `AI`, `Automation`, `Bot`, `Productivity`, `Developer Tools`

### 2. Configure Bot

1. Go to **Bot** in sidebar
2. Click **Reset Token** -> copy and save the token securely
3. Configure settings:
   - **Public Bot**: OFF (only you can invite)
   - **Requires OAuth2 Code Grant**: OFF

### 3. Enable Gateway Intents (Required for Inbound Messages)

In the **Bot** settings page, under **Privileged Gateway Intents**:

- [ ] **MESSAGE CONTENT INTENT**: ON (required to read message text)
- [ ] **SERVER MEMBERS INTENT**: Optional
- [ ] **PRESENCE INTENT**: Optional

> **CRITICAL**: Without MESSAGE CONTENT INTENT, the gateway cannot read message content.

### 4. Invite Bot to Server

1. Go to **OAuth2** -> **URL Generator**
2. Select scopes:
   - [ ] `bot`
3. Select bot permissions:
   - [ ] Read Messages/View Channels
   - [ ] Send Messages
   - [ ] Read Message History
4. Copy the generated URL
5. Open URL in browser -> select server -> **Authorize**

---

## Configuration

### Config File Location

```
services/config/discord.yaml
```

### Full Configuration

```yaml
# Bot authentication
bot_token: "YOUR_BOT_TOKEN"

# Default targets for CLI (outbound)
default_channel_id: "1234567890123456789"  # Optional
default_user_id: "9876543210987654321"     # Optional

# Enable/disable Discord integration
enabled: true

# Gateway configuration (inbound)
gateway:
  enabled: true
  target_agent: "systemprompt_hub"      # Agent to receive messages
  message_prefix: "DISCORD_MESSAGE"     # Prefix for formatted messages
  ignore_channels: []                   # Channel IDs to ignore
  ignore_bots: true                     # Ignore messages from other bots
```

### Getting Discord IDs

Enable Developer Mode in Discord:
1. User Settings -> Advanced -> **Developer Mode**: ON
2. **Channel ID**: Right-click channel -> Copy Channel ID
3. **User ID**: Right-click username -> Copy User ID

---

## Outbound: CLI Extension

Send messages TO Discord from agents or CLI.

-> See [Discord CLI Playbook](/playbooks/cli-discord) for complete commands

### Quick Reference

| Task | Command |
|------|---------|
| Test connection | `systemprompt plugins run discord test` |
| Send to default | `systemprompt plugins run discord send "message"` |
| Send to channel | `systemprompt plugins run discord send -c <id> "message"` |
| Send DM | `systemprompt plugins run discord send -u <id> "message"` |

### From Agents

Agents can send Discord notifications using the `systemprompt` MCP tool:

```yaml
# In agent instructions
Discord notifications:
- Use: systemprompt plugins run discord send "<message>"
- With channel: systemprompt plugins run discord send "<message>" --channel <id>
```

---

## Inbound: Gateway Job

Receive messages FROM Discord and forward to agents.

### Architecture

The Gateway uses serenity (Rust Discord library) to maintain a WebSocket connection:

```
Discord Gateway (WebSocket)
        |
        v
+---------------------------+
| Soul Extension            |
|  +- DiscordGatewayJob     |
|       |                   |
|       v                   |
|  systemprompt admin       |
|  agents message <agent>   |
|       -m "<message>"      |
+---------------------------+
        |
        v
   Target Agent
```

### Source Files

| File | Purpose |
|------|---------|
| `extensions/soul/src/discord/mod.rs` | Module exports |
| `extensions/soul/src/discord/config.rs` | Configuration types |
| `extensions/soul/src/discord/handler.rs` | Serenity event handler |
| `extensions/soul/src/discord/service.rs` | Outbound HTTP service |
| `extensions/soul/src/jobs/discord_gateway.rs` | Gateway job |

### Path Resolution

The handler uses core utilities for binary path resolution:

```rust
use systemprompt::cloud::ProjectContext;
use systemprompt::loader::ExtensionLoader;

fn resolve_cli_binary() -> PathBuf {
    let project = ProjectContext::discover();
    ExtensionLoader::get_cli_binary_path(project.root(), "systemprompt")
        .unwrap_or_else(|| PathBuf::from("systemprompt"))
}
```

> **Note**: Always use `ProjectContext` and `ExtensionLoader` for path resolution. Never hardcode paths.

### Message Format

Messages forwarded to agents use this format:

```
DISCORD_MESSAGE: channel=<id> (<name>) author=<username> content=<message>
```

Example:
```
DISCORD_MESSAGE: channel=1234567890 (general) author=ejb503 content=Hello Claude!
```

### Running the Gateway

**Automatic Start:** The gateway starts automatically when the server boots. No manual action required for production.

**Manual Start (Development):**
```bash
# Run as a job (blocks forever)
systemprompt infra jobs run soul_discord_gateway

# Check job status
systemprompt infra jobs list | grep discord
```

The gateway job has:
- Schedule: `@reboot` - runs on server startup
- `run_on_startup: true` - auto-starts with the scheduler
- Blocks forever maintaining a persistent WebSocket connection

### Monitoring

```bash
# View recent logs
systemprompt infra logs view --limit 50

# Search for Discord events
systemprompt infra logs search "discord"
```

Expected log messages:
```
INFO  Resolved systemprompt CLI binary via ExtensionLoader path=/path/to/systemprompt
INFO  Starting Discord gateway bot target_agent=systemprompt_hub
INFO  Discord gateway connected bot_name=systemprompt guild_count=1
INFO  Received Discord message, forwarding to agent channel=general author=user
INFO  Discord message forwarded to agent successfully
```

---

## Target Agent Configuration

The gateway forwards messages to an agent (default: `systemprompt_hub`). Configure the agent to handle Discord messages:

### Agent Instructions Example

```yaml
# services/agents/systemprompt_hub.yaml
name: systemprompt_hub
description: Central communications hub

instructions: |
  You receive messages from various sources including Discord.

  ## Discord Messages

  Messages from Discord arrive in this format:
  DISCORD_MESSAGE: channel=<id> (<name>) author=<username> content=<message>

  When you receive a DISCORD_MESSAGE:
  1. Parse the author and content
  2. Process the request appropriately
  3. Respond via Discord if needed:
     systemprompt plugins run discord send "<response>"
```

---

## Troubleshooting

### Gateway Won't Connect

**Symptoms**: `Discord gateway client starting...` but no `connected` message

**Check**:
1. Bot token is valid: `systemprompt plugins run discord test`
2. TOKEN in config matches Discord Developer Portal
3. Network connectivity to Discord

### Messages Not Received

**Symptoms**: Messages sent in Discord don't appear in logs

**Check**:
1. MESSAGE CONTENT INTENT is enabled in Developer Portal
2. Bot is in the server/channel
3. `ignore_bots: true` isn't filtering your messages
4. Channel isn't in `ignore_channels` list

### "Permission denied" Error

**Symptoms**: `Failed to execute agent message command: Permission denied`

**Fix**: The handler couldn't find the CLI binary. Ensure:
1. Project is built: `just build`
2. Running from project root directory
3. Binary exists at `target/debug/systemprompt` or `target/release/systemprompt`

### Agent Returns Error

**Symptoms**: Message forwarded but agent fails

**Check**:
1. Target agent exists: `systemprompt admin agents list`
2. Agent is configured correctly
3. Agent's AI provider is working

---

## Quick Reference

### Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `bot_token` | Required | Discord bot token |
| `default_channel_id` | None | Default channel for CLI |
| `default_user_id` | None | Default user for DMs |
| `enabled` | `true` | Enable Discord integration |
| `gateway.enabled` | `true` | Enable inbound gateway |
| `gateway.target_agent` | `systemprompt_hub` | Agent to receive messages |
| `gateway.message_prefix` | `DISCORD_MESSAGE` | Message format prefix |
| `gateway.ignore_channels` | `[]` | Channel IDs to ignore |
| `gateway.ignore_bots` | `true` | Ignore bot messages |

### Commands

| Task | Command |
|------|---------|
| Test CLI connection | `systemprompt plugins run discord test` |
| Send message | `systemprompt plugins run discord send "msg"` |
| Run gateway | `systemprompt infra jobs run soul_discord_gateway` |
| List jobs | `systemprompt infra jobs list` |
| View logs | `systemprompt infra logs view --limit 50` |

### Files

| Path | Purpose |
|------|---------|
| `services/config/discord.yaml` | Configuration |
| `extensions/soul/src/discord/` | Gateway implementation |
| `extensions/cli/discord/` | CLI extension |
| `services/agents/*.yaml` | Agent configurations |

---

## Related Playbooks

- `cli_discord` - Discord CLI commands
- `cli_jobs` - Job scheduler
- `cli_agents` - Agent messaging
- `build_extension-checklist` - Extension standards
- `guide_coding-standards` - Rust coding standards