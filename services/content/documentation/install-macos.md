---
title: "Install on macOS"
description: "Install and verify systemprompt Bridge on macOS, sign in, and connect Claude Code or Cowork."
author: "systemprompt.io"
slug: "install-macos"
kind: "guide"
public: true
tags: ["documentation", "getting-started", "bridge"]
published_at: "2026-09-08"
updated_at: "2026-09-28"
related_playbooks:
  - title: "Connect Claude Code"
    url: "/documentation/connect-claude-code"
  - title: "Connect Cowork"
    url: "/documentation/connect-cowork"
  - title: "Downloads"
    url: "/documentation/downloads"
---

# Install on macOS

Install systemprompt Bridge, sign in to your gateway, then connect Claude Code or Cowork. The macOS download is available only when your administrator has published it on the gateway; if it is missing, ask for an approved build.

## 1. Download and verify

Download the [macOS disk image](/files/downloads/systemprompt-bridge-macos.dmg) and its [SHA-256 checksum](/files/downloads/systemprompt-bridge-macos.dmg.sha256) from your gateway. In Terminal, change to the directory containing both files:

```bash
cd "$HOME/Downloads"
shasum -a 256 -c systemprompt-bridge-macos.dmg.sha256
```

Continue only if verification reports `OK`. If your browser renamed either download, restore the published filenames before checking. More detail: [Downloads](/documentation/downloads).

Open the disk image and drag **Systemprompt Bridge** to **Applications**. Eject the disk image. The installed app must be at `/Applications/SystempromptBridge.app`; do not run it from the mounted image.

Whether the app is Developer ID signed and notarized depends on how your administrator built and packaged it. macOS may ask you to confirm opening an app downloaded from the internet. You can check the installed app:

```bash
spctl --assess --type execute --verbose=2 /Applications/SystempromptBridge.app
```

For a signed and notarized build, expect acceptance with a notarized Developer ID source. If macOS rejects the signature or blocks the app under your organization's policy, stop and contact IT. Do not remove quarantine attributes or disable Gatekeeper.

## 2. Configure your gateway and sign in

The command-line interface is inside the app. Define this alias in the Terminal window you will use:

```bash
alias systemprompt-bridge=/Applications/SystempromptBridge.app/Contents/MacOS/systemprompt-bridge
GATEWAY_URL="https://gateway.example.com"
```

Replace the example URL with the gateway supplied by your administrator. Do not assume the app's default gateway is your intended one.

For a fresh device, pin the manifest public key supplied through your administrator's approved channel:

```bash
systemprompt-bridge install --gateway "$GATEWAY_URL" \
  --pubkey "ADMINISTRATOR_PROVIDED_BASE64_PUBLIC_KEY"
systemprompt-bridge login --gateway "$GATEWAY_URL"
```

If IT has already configured a trusted key, do not replace it. Approve the sign-in link in your browser with your work account. For a one-time connect code instead, follow [Bridge sign-in](/documentation/bridge-install#sign-in).

Launch systemprompt Bridge from Applications and keep it running. Synchronize your account's resources:

```bash
systemprompt-bridge sync
systemprompt-bridge whoami
```

## 3. Connect your client

- **Terminal:** follow [Connect Claude Code](/documentation/connect-claude-code) to generate its settings and choose always-on or per-session routing. The bridge does not install Claude Code as part of copying the macOS app.
- **Desktop:** follow [Connect Cowork](/documentation/connect-cowork) to install Claude Desktop's managed profile and approve it in System Settings.

These are separate setup paths. The broad `install --apply` option applies platform policy and can reapply other enrolled client profiles; it is not a Claude Code-only switch. Use the client-specific instructions.

## 4. Verify

Run `systemprompt-bridge status` and `systemprompt-bridge doctor`, then make the real request described in your client guide. Confirm it in your gateway's [History](/admin/history). Sign-in and diagnostic success alone are not an end-to-end inference test.

Doctor can report checks for other installed clients, and some integration checks need an initial session. Investigate failures by client instead of assuming that every warning means sign-in failed.

## Find configuration and settings

The default bridge directory is:

```text
~/Library/Application Support/systemprompt/
```

In Finder, press **Command–Shift–G** and paste that path. An absolute `XDG_CONFIG_HOME` changes the base directory; use `systemprompt-bridge status` to find your actual configuration.

Claude Code enrolment creates `claude-code-settings.json` and its helper here. The file does not exist merely because the app was downloaded. See [Find your installed settings file](/documentation/connect-claude-code#find-your-installed-settings-file) for exact commands.

The directory also contains credentials. Share diagnostic error messages with support, not its contents.

## Disconnect a client

Use `systemprompt-bridge uninstall --host claude-code` for Claude Code. For Cowork, follow [Disconnect Cowork](/documentation/connect-cowork#disconnect-cowork), including any required macOS profile removal. Disconnecting one client does not sign out the whole bridge.
