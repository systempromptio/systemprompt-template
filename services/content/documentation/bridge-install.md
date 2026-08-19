---
title: "Install the Desktop Bridge"
description: "Install and link the desktop bridge on Windows, macOS, Linux, and WSL, then verify Claude Code is routed through the gateway."
author: "Astound Digital"
slug: "bridge-install"
keywords: "desktop bridge, install, windows, macos, linux, wsl, claude code, gateway mode, device link, exchange code, astound-bridge"
kind: "guide"
public: true
tags: ["documentation", "getting-started", "bridge", "claude-code"]
published_at: "2026-08-19"
updated_at: "2026-08-19"
after_reading_this:
  - "Install the bridge on Windows, macOS, Linux, or WSL"
  - "Link the bridge to your account with a one-shot code"
  - "Understand what the bridge configures for Claude Code"
  - "Verify the installation with astound-bridge doctor"
related_playbooks:
  - title: "Connect Claude Code"
    url: "/documentation/connect-claude-code"
  - title: "Create & Manage Users"
    url: "/documentation/admin-user-management"
  - title: "Rolling Out the Bridge"
    url: "/documentation/salesforce-bridge-rollout"
---

# Install the Desktop Bridge

This page is for anyone linking a desktop AI client — Claude Cowork on Windows
or macOS especially — to the gateway. If all you want is Claude Code in a
terminal, [Connect Claude Code](/documentation/connect-claude-code) is one
command and faster.

The bridge is a small program on your laptop that connects your Claude Code
(and other AI clients) to your organization's gateway. It keeps your skills,
plugins, and MCP servers in sync, and runs a local inference proxy so every
model request is routed, governed, and recorded through the gateway — you never
handle an API key.

Throughout this page, replace `https://gateway.example.com` with your
instance's URL. If your instance is not remotely reachable yet, see
[Expose Your Instance Remotely](/documentation/remote-access).

## Before you start: get a code

Linking the bridge to your account uses a **one-shot exchange code** — 32
random bytes, valid for 10 minutes, single use. Get one either way:

- **Self-serve:** sign in at `https://gateway.example.com/admin/login`
  (Salesforce SSO or passkey — see
  [Authentication](/documentation/authentication)), open your **Profile**
  page, and it mints a code with the install command filled in.
- **Admin-issued** (headless, or on someone's behalf):

  ```bash
  systemprompt admin bridge issue-code --user-id <email-or-uuid>
  ```

The code expires in ten minutes, so install the bridge first and mint the code
last.

## Linux

One command downloads, verifies (SHA-256), installs, links, and configures
Claude Code:

```bash
curl -fsSL https://gateway.example.com/files/downloads/install.sh | sh -s -- \
  --download-base https://gateway.example.com/files/downloads --code <code>
```

Installs to `/usr/local/bin` when run as root, otherwise `~/.local/bin`.
x86_64 and aarch64 are supported. Have an existing PAT instead of a code? Pass
`--pat sp-live-…`. Skip the Claude Code install with `--no-claude-code`.

What it sets up:

| Piece | Where |
|-------|-------|
| Bridge binary | `astound-bridge` on your `PATH` |
| Client config + PAT | `~/.config/astound/` (PAT is `0600`) |
| Shell environment | `~/.config/astound/env.sh`, sourced from a managed block in `~/.profile` |
| Claude Code settings | `~/.claude/managed-settings.json` (or `/etc/claude-code/` if writable): gateway base URL, `apiKeyHelper`, model discovery |
| Org plugins, skills, MCP servers | `~/.local/share/Claude/org-plugins/` |
| Background services | systemd user units: a 30-minute sync timer and the loopback inference proxy on `127.0.0.1:48217` |

Open a new login shell (or `. ~/.profile`), then run `claude`. Requests now go
laptop → local proxy → gateway → model provider, with every call audited.

Runtime dependencies on minimal distributions: `libdbus-1-3 libcap2
libgcrypt20 libsystemd0`.

## WSL (Windows Subsystem for Linux)

Use the **Linux** instructions above inside your WSL distribution — not the
Windows `.exe`. Two caveats:

- **No systemd user bus** (default on older WSL or inside containers): the
  installer still writes the systemd units but warns that it cannot enable
  them. Start the proxy yourself in that case:

  ```bash
  astound-bridge proxy &
  ```

  On WSL2 with systemd enabled (`systemd=true` in `/etc/wsl.conf`), the units
  work normally.
- **`localhost` is the WSL VM, not Windows.** If your gateway runs on the
  Windows host or another machine, use its real hostname or IP, not
  `localhost`.

## Windows

1. Download `astound-bridge-windows.exe` from
   `https://gateway.example.com/bridge-auth/setup` (sign in first). Verify the
   checksum against the `.sha256` file published alongside it.
2. Run it. The bridge signs you in via a browser **device link**: approve the
   link while signed in at the gateway, and the bridge exchanges the code for
   its durable credential.
3. The bridge configures Claude Code through Windows policy — registry keys
   under `HKLM\SOFTWARE\Policies\Claude` (falling back to `HKCU` without
   elevation): gateway mode, base URL, bearer auth, managed MCP servers.

Bridge configuration lives under `%APPDATA%\systemprompt\`.

## macOS

1. Download `astound-bridge-macos.dmg` from
   `https://gateway.example.com/bridge-auth/setup` and drag the app to
   Applications.
2. Launch it and sign in via the device link, as on Windows.
3. Claude configuration is applied as managed preferences at
   `/Library/Managed Preferences/com.anthropic.claudefordesktop.plist`.

Bridge configuration lives under `~/Library/Application Support/systemprompt/`.

## Verify

```bash
astound-bridge doctor
```

One line per check: credential valid, proxy running, Claude Code configured,
sync current. Then run `claude`, ask anything, and confirm the request appears
in the gateway audit trail (an admin can check with
`systemprompt infra logs request list --limit 5`).

## Troubleshooting

| Symptom | Cause |
|---------|-------|
| Code rejected | 10-minute TTL, single use. Mint a fresh one from your profile page. |
| `claude` works but the audit trail is empty | Claude Code is not routed through the gateway. Check `astound-bridge doctor`; on Linux, make sure you opened a new login shell after install. |
| Claude Code ignores the gateway token | Claude Code ≥ 2.1.146 blocks `apiKeyHelper` and auth-token env vars when `forceLoginMethod` / `forceLoginOrgUUID` are set in existing settings. Remove those keys. |
| IDE terminal not configured (Linux) | `~/.profile` only reaches login shells. `~/.claude/managed-settings.json` is the reliable channel and is written by the installer — restart the IDE. |
| Proxy not running on WSL | No systemd user bus — run `astound-bridge proxy` manually or enable systemd in `/etc/wsl.conf`. |

To undo everything the installer wrote: `astound-bridge uninstall`.
