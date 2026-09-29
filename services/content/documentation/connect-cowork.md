---
title: "Connect Cowork"
description: "Connect Claude Cowork through systemprompt Bridge on macOS and Windows, approve Desktop policy, and verify a session."
author: "systemprompt.io"
slug: "connect-cowork"
kind: "guide"
public: true
tags: ["documentation", "getting-started", "bridge"]
published_at: "2026-09-09"
updated_at: "2026-09-28"
related_playbooks:
  - title: "Connect Claude Code"
    url: "/documentation/connect-claude-code"
  - title: "Install the Desktop Bridge"
    url: "/documentation/bridge-install"
  - title: "Downloads"
    url: "/documentation/downloads"
---

# Connect Cowork

Connect Cowork in Claude Desktop to your organization's gateway on macOS or Windows. This is separate from [Claude Code in a terminal](/documentation/connect-claude-code).

## Before you start

- Install your organization's approved Claude Desktop app.
- Complete [macOS bridge installation](/documentation/install-macos) or [Windows bridge installation](/documentation/install-windows), including gateway selection, signing trust, and sign-in.
- Confirm with your administrator that Claude Desktop gateway access and the models you need are enabled for your account.
- Obtain permission to install the Desktop managed profile. Administrator approval may be required.

This guide configures the Desktop app on your computer. It does not configure a browser-based Cowork session.

## 1. Sync and install the Desktop profile

Save your work and quit Claude Desktop before applying the profile.

On **macOS**, use the alias from the installation guide:

```bash
systemprompt-bridge sync
systemprompt-bridge install --host claude-desktop
```

The bridge prepares a configuration profile and opens the macOS profile-installation flow. In System Settings, review and install the downloaded profile. Follow the displayed instructions and approve the administrator prompt if authorized. Opening the profile is not the same as completing installation.

On **Windows**, use the `$Bridge` executable path from the installation guide:

```powershell
& $Bridge sync
& $Bridge install --host claude-desktop
```

The command writes and verifies Claude Desktop policy. It does not configure the native Claude Code CLI. If it reports that the organizational plugins directory is not provisioned, open PowerShell **as Administrator**, set `$Bridge` to the same verified executable path, and run:

```powershell
& $Bridge install --apply
```

This provisions the Desktop policy and organizational plugin directory; it can also reapply profiles for other enrolled clients. Obtain IT approval before using it on a managed device. Return to your normal-user PowerShell, run sync, and repeat Desktop enrolment. Do not use a different administrator account's sign-in as your own.

If installation reports pending approval, disabled access, or a failure, resolve it before continuing. Do not remove an organization's existing policy to force installation. Your administrator can manage macOS Desktop policy using [Anthropic's Desktop configuration guidance](https://support.claude.com/en/articles/12611117-deploy-claude-desktop-for-macos).

## 2. Open Cowork

Keep systemprompt Bridge running and reopen Claude Desktop after the profile is installed. Open Cowork and choose a Claude model available through your organization's gateway configuration.

If Cowork or gateway mode is unavailable, confirm the approved Desktop version, installed profile, and account access with your administrator. Do not assume that signing in to a personal Claude account establishes the gateway connection.

The `--model` examples in the Claude Code guide are terminal commands, not Cowork setup instructions.

## 3. Verify a session

1. Ask Cowork a short question that requires no tools.
2. Find the request in the gateway's [History](/admin/history), or ask your administrator to confirm the account, model, and timestamp.
3. Try an approved read-only organizational skill or tool and confirm its result separately.
4. Run bridge diagnostics if either test fails:

**macOS:**

```bash
systemprompt-bridge whoami
systemprompt-bridge status
systemprompt-bridge doctor
```

**Windows PowerShell:**

```powershell
& $Bridge whoami
& $Bridge status
& $Bridge doctor
```

A connected bridge or visible plugin does not, by itself, prove that a Cowork model request reached the gateway. Initial session-dependent checks may need a Cowork session before they can report useful results.

## Troubleshooting

**Desktop still shows the previous configuration.** Fully quit and reopen Claude Desktop. Confirm the profile installation or Windows approval completed, rather than only downloading or opening the profile.

**Cowork cannot connect.** Keep the bridge running, verify the gateway and account with `whoami`, and inspect the relevant doctor failures. Do not substitute a guessed localhost port in the profile.

**Skills or tools are missing.** Run sync again, check its errors, and restart Desktop. Confirm your account has access to the expected resources.

**A profile or security policy blocks setup.** Ask IT to resolve it. Do not bypass management restrictions or disable OS security protections.

## Disconnect Cowork

Save your work and quit Claude Desktop. Remove its bridge profile:

**macOS:**

```bash
systemprompt-bridge uninstall --host claude-desktop
```

**Windows PowerShell:**

```powershell
& $Bridge uninstall --host claude-desktop
```

Complete any OS approval or manual profile-removal instructions reported by the command. An IT-managed profile may require your administrator to remove it. Reopen Desktop after removal.

This disconnects the Desktop integration; it does not remove Claude Code's routing or sign out the bridge.
