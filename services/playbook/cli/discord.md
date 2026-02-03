---
title: "Discord CLI Extension Playbook"
description: "Send messages to Discord channels or users from the command line."
author: "SystemPrompt"
slug: "cli-discord"
keywords: "discord, messaging, notifications, cli"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Discord CLI Extension

Send messages to Discord channels or users from the command line.

---

## Overview

The Discord CLI extension allows you to send messages to Discord channels or users directly from the command line. Use it for notifications, alerts, or any automated messaging.

---

## Prerequisites

- [ ] Discord bot created at https://discord.com/developers/applications
- [ ] Bot token configured in `services/config/discord.yaml`
- [ ] Bot invited to your Discord server with "Send Messages" permission

---

## Configuration

Create `services/config/discord.yaml`:

```yaml
bot_token: "YOUR_BOT_TOKEN"
default_channel_id: "1234567890123456789"  # Optional: default channel
default_user_id: "9876543210987654321"     # Optional: default DM recipient
enabled: true
```

### Getting Your Bot Token

1. Go to https://discord.com/developers/applications
2. Click "New Application" -> name it -> "Create"
3. Go to "Bot" in sidebar -> "Reset Token" -> copy token

### Getting Discord IDs

1. Enable Developer Mode: Discord Settings -> Advanced -> Developer Mode
2. **Channel ID**: Right-click channel -> "Copy Channel ID"
3. **User ID**: Right-click username -> "Copy User ID"

### Inviting the Bot

1. Go to OAuth2 -> URL Generator
2. Select scopes: `bot`
3. Select permissions: `Send Messages`
4. Copy URL -> open in browser -> select server -> Authorize

---

## Commands

### Test Connection

Verify bot token and connectivity:

```json
{ "command": "plugins run discord test" }
```

**Expected output:**
```
-> Loading Discord configuration...
[OK] Configuration loaded
-> Testing Discord API connection...
[OK] Connected as: YourBot#1234
```

---

### Send Message

Send a message to the default target (from config):

```json
{ "command": "plugins run discord send \"Hello from SystemPrompt!\"" }
```

**Expected output:**
```
-> Sending to default target...
[OK] Message sent! (ID: 123456789, Channel: 987654321)
```

---

### Send to Specific Channel

Override the default and send to a specific channel:

```json
{ "command": "plugins run discord send --channel <channel-id> \"Channel message\"" }
```

Or using short flag:

```json
{ "command": "plugins run discord send -c <channel-id> \"Channel message\"" }
```

---

### Send Direct Message

Send a DM to a specific user:

```json
{ "command": "plugins run discord send --user <user-id> \"Private message\"" }
```

Or using short flag:

```json
{ "command": "plugins run discord send -u <user-id> \"Private message\"" }
```

**Note:** The bot must share a server with the user to send DMs.

---

## Examples

### Daily Report Notification

```bash
systemprompt plugins run discord send "Daily report generated: https://example.com/report"
```

### Build Status Alert

```bash
systemprompt plugins run discord send -c 1234567890 "Build #123 passed"
```

### Error Alert to Admin

```bash
systemprompt plugins run discord send -u 9876543210 "Critical error in production"
```

### Multi-line Message

```bash
systemprompt plugins run discord send "Build Report
━━━━━━━━━━━━
Status: Passed
Duration: 2m 34s
Commit: abc123"
```

---

## Troubleshooting

### "Missing Access" Error

**Cause:** Bot doesn't have permission to access the channel.

**Fix:**
1. Ensure bot is invited to the server
2. Check bot has "Send Messages" permission
3. Verify channel ID is correct

### "Cannot send DM to this user"

**Cause:** User has DMs disabled or doesn't share a server with the bot.

**Fix:**
1. User must share at least one server with the bot
2. User must have DMs enabled for that server

### "Invalid Discord channel/user ID format"

**Cause:** ID is not a valid Discord snowflake (17-20 digits).

**Fix:** Copy the ID directly from Discord using Developer Mode.

### "Discord bot token cannot be empty"

**Cause:** Config file missing or token not set.

**Fix:** Create `services/config/discord.yaml` with valid `bot_token`.

### Rate Limited

**Cause:** Too many messages sent too quickly.

**Fix:** Wait the indicated time and retry. Discord limits:
- 5 messages per 5 seconds per channel
- 10 DM channel creations per second

---

## Quick Reference

| Task | Command |
|------|---------|
| Test connection | `systemprompt plugins run discord test` |
| Send to default | `systemprompt plugins run discord send "message"` |
| Send to channel | `systemprompt plugins run discord send -c <id> "message"` |
| Send DM | `systemprompt plugins run discord send -u <id> "message"` |
| Show help | `systemprompt plugins run discord --help` |
| Show send help | `systemprompt plugins run discord send --help` |

---

## Related

- `build_extension-cli` - How to build CLI extensions
- `cli_plugins` - Managing plugins and extensions