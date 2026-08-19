---
title: "Connect Claude Code"
description: "Connect Claude Code to the hosted Astound gateway with one command: get a one-shot code from your profile page, run the installer, and every session is routed, governed, and audited — no repo, no build."
author: "Astound Digital"
slug: "connect-claude-code"
keywords: "claude code, connect, install, bridge, one-shot code, exchange code, remote gateway, hosted instance, skills, plugins"
kind: "guide"
public: true
tags: ["documentation", "getting-started", "claude-code"]
published_at: "2026-08-06"
updated_at: "2026-08-19"
after_reading_this:
  - "Connect Claude Code to the hosted gateway with one command"
  - "Know what the installer writes, and how to undo it"
  - "Verify the connection and the synced skills"
related_playbooks:
  - title: "Install the Desktop Bridge"
    url: "/documentation/bridge-install"
  - title: "Develop Against a Local Gateway"
    url: "/documentation/develop-claude-code"
---

# Connect Claude Code

One command takes a machine with nothing installed to a working `claude` wired
to the hosted Astound instance. You do not clone anything, build anything, or
run a server: the installer downloads from the gateway, installs Claude Code
itself if it is missing, signs you in, and syncs your organization's skills,
plugins, and MCP servers. From then on every session routes through the gateway
and lands in the audit trail.

Prefer a desktop app? The bridge also powers Claude Cowork on Windows and
macOS: [Install the Desktop Bridge](/documentation/bridge-install).

Running your own gateway from a checkout of this repository? That is a
different page: [Develop Against a Local Gateway](/documentation/develop-claude-code).

## 1. Get an account

Accounts live on the hosted instance — there is nothing to install for this
step. At [astound.systemprompt.io/admin/login](https://astound.systemprompt.io/admin/login),
either:

- **Sign in with Salesforce.** If your email domain is on the allow-list
  (`@astounddigital.com`, `@astoundcommerce.com`), your account is created
  automatically the first time you sign in — no admin involved.
- **Register with a passkey.** Same domain gate; you verify your email and
  register a passkey, no password.

If your domain is not allow-listed, an administrator creates the account and
hands you a connect code in one step:

```bash
systemprompt admin users create --name <name> --email <email> --if-not-exists
systemprompt --json admin bridge issue-code --user-id <email> \
  | jq -r '.sections[] | select(.heading == "code") | .content'
```

## 2. Get a connect code

Sign in and open your **Profile** page. It mints a one-shot connect code and prints the
install command with the code filled in — copy that command and skip to the
next step.

The code is 32 random bytes, stored only as a SHA-256 hash, valid for ten
minutes, single use. The installer redeems it for a durable personal access
token that stays on your machine and never passes through the browser. A
leaked code is dead within minutes; if yours expires, reload the profile page
for a fresh one. Headless or scripted? The admin `issue-code` command in the
previous step mints the same code without a browser.

## 3. Run the installer

On Linux or WSL:

```bash
curl -fsSL https://astound.systemprompt.io/files/downloads/install.sh | sh -s -- --code <code>
```

For Windows and macOS, use the platform installers on
[Install the Desktop Bridge](/documentation/bridge-install) instead. The
Linux tarball is published for x86_64; the installer also supports aarch64
where that artifact is staged.

Run without `--code` and the installer falls back to interactive single
sign-on: it opens a browser against the gateway (or prints the URL when there
is no display) and you approve the machine from your signed-in session.

The installer, in order: downloads the bridge binary and verifies its SHA-256
checksum, installs Claude Code (`npm i -g @anthropic-ai/claude-code`, falling
back to the native installer), redeems the code, writes the environment,
starts the loopback inference proxy, syncs your organization's plugins, and
finishes with a self-test (`astound-bridge doctor`).

What it writes:

| Path | Contents |
|------|----------|
| `~/.local/bin/astound-bridge` | The bridge binary (`/usr/local/bin` as root) |
| `~/.config/astound/` | Client config, PAT (0600), loopback key, `env.sh` |
| `~/.profile` | Managed block setting `ANTHROPIC_BASE_URL` and the auth token |
| `~/.claude/managed-settings.json` | Base URL, `apiKeyHelper`, model discovery |
| `~/.local/share/Claude/org-plugins/` | Organization plugins, skills, MCP servers |
| systemd user units | 30-minute sync timer, loopback inference proxy |

To undo all of it: `astound-bridge uninstall`, then remove the binary.

## 4. Use it

Open a new login shell (or `. ~/.profile`) and run:

```bash
claude
```

No exports needed — `~/.config/astound/env.sh` points Claude Code at the
loopback proxy, which authenticates to the gateway for you. Your
organization's skills and plugins are already registered:

```bash
claude plugin list
```

The bridge re-syncs every 30 minutes, so plugins and skills published to the
instance show up on your machine without reinstalling.

## Verify

```bash
astound-bridge doctor
```

One line per check: credential, loopback secret, proxy, org marketplace,
filesystem layout. Exit 11 means at least one hard failure. A hook-token
warning is expected on a fresh install — OAuth client provisioning is lazy,
on the first plugin hook request rather than during sync.

## Troubleshooting

| Symptom | Cause |
|---------|-------|
| Code rejected | 10-minute TTL, single use. Reload the profile page for a fresh one. |
| `claude` works, audit trail empty | Not routed through the gateway. Check `ANTHROPIC_BASE_URL` points at the loopback proxy and `astound-bridge doctor` reports it running. |
| The proxy is not listening | Start it with `astound-bridge proxy` and check `${TMPDIR:-/tmp}/astound-bridge-proxy.log`; on systemd hosts, `systemctl --user status astound-bridge-proxy.service`. |
| `claude plugin list` is empty | Claude Code was not installed when sync ran — the marketplace step skips silently without the CLI. Install Claude Code, then `astound-bridge sync`. |
| No browser for SSO (SSH box) | `astound-bridge login --no-browser --gateway https://astound.systemprompt.io`, or use a `--code` from your profile page. |
