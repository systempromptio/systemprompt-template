---
title: "Install on Windows"
description: "Install native Windows Claude Code with gateway routing from PowerShell, or connect Cowork."
author: "systemprompt.io"
slug: "install-windows"
kind: "guide"
public: true
tags: ["documentation", "getting-started", "bridge"]
published_at: "2026-09-09"
updated_at: "2026-09-28"
related_playbooks:
  - title: "Connect Claude Code"
    url: "/documentation/connect-claude-code"
  - title: "Connect Cowork"
    url: "/documentation/connect-cowork"
  - title: "Downloads"
    url: "/documentation/downloads"
---

# Install on Windows

Run Claude Code directly in Windows PowerShell with the native systemprompt Bridge. WSL is optional; use it when your projects need a Linux environment.

| Client | Where it runs | Bridge to install |
|---|---|---|
| Claude Code in PowerShell or CMD | Native Windows | Windows executable with Claude Code enrolment support |
| Claude Cowork in Claude Desktop | Native Windows | Windows executable; separate Desktop enrolment |
| Claude Code in WSL | Inside WSL | Linux bridge inside the same distribution |

## Required bridge build

Native Claude Code enrolment needs a Windows bridge build that offers the `claude-code` host. Older Windows builds synchronize plugins but do not generate Claude Code inference settings. Updating documentation does not update the executable your gateway serves; ask your administrator for the approved build.

After enrolment, confirm that the command reports **gateway keys merged into Claude Code's settings file**, the standalone file exists, and `doctor` includes **claude code settings** checks. A version number or successful sync alone does not establish support.

## 1. Install Claude Code

Open a normal-user **PowerShell** terminal. Install Claude Code using your organization's approved method or Anthropic's native installer:

```powershell
irm https://claude.ai/install.ps1 | iex
```

Open a new PowerShell window after installation, then check:

```powershell
claude --version
```

See [Anthropic's Windows setup instructions](https://code.claude.com/docs/en/setup#set-up-on-windows) for supported Windows versions and installation alternatives. Git for Windows enables Claude Code's Bash tool; current Claude Code can use PowerShell when Git Bash is absent. Older client versions may require Git for Windows. Neither option requires WSL.

## 2. Download and verify the bridge

Download the [Windows executable](/files/downloads/systemprompt-bridge-windows.exe) and its [checksum](/files/downloads/systemprompt-bridge-windows.exe.sha256) from your gateway. These links work only when your administrator has published the files; otherwise ask for an approved build. Complete the [PowerShell checksum comparison](/documentation/downloads#windows-powershell) before running it.

Keep the verified executable in a stable folder approved by your organization. Claude Code's generated credential helper calls this executable, so moving or deleting it breaks authentication until you repeat enrolment from its new location. In each PowerShell window where you run bridge commands, define its actual path:

```powershell
$Bridge = "C:\Users\YOUR_USERNAME\Applications\systemprompt\systemprompt-bridge-windows.exe"
& $Bridge --version
```

Replace the example with the path where you placed the verified download. If endpoint policy blocks it, ask IT to approve the package.

## 3. Configure your gateway and sign in

Use the HTTPS address and trusted manifest public key supplied by your administrator. Replace the placeholder below with your administrator's gateway URL, and download and sign in against that same gateway.

```powershell
$GatewayUrl = "https://gateway.example.com"
& $Bridge install --gateway $GatewayUrl --pubkey "ADMINISTRATOR_PROVIDED_BASE64_PUBLIC_KEY"
& $Bridge login --gateway $GatewayUrl
```

If IT already provisioned the trusted key, omit the `--pubkey` argument rather than replacing it. Approve the browser's device link with your work account.

Alternatively, open your gateway Profile, click **Generate a connect code**, then run:

```powershell
& $Bridge login --gateway $GatewayUrl --code "YOUR_CONNECT_CODE"
```

The code is single-use and expires after ten minutes. Generate another if necessary; reloading Profile does not create one.

## 4. Start the bridge and enrol Claude Code

Open the verified executable from File Explorer and keep the bridge app running. For CLI-only use, run the following in a separate PowerShell window and leave it open:

```powershell
& $Bridge proxy
```

Define `$Bridge` in that window first. Use either the running app or the proxy process; do not start another proxy when one is already serving this installation.

Back in your working PowerShell window, close existing Claude Code sessions and run:

```powershell
& $Bridge sync
& $Bridge install --host claude-code
& $Bridge sync
& $Bridge whoami
& $Bridge doctor
```

Enrolment generates `%APPDATA%\systemprompt\claude-code-settings.json` and merges routing into Claude Code's managed settings when writable, otherwise `%USERPROFILE%\.claude\settings.json`. The Windows helper invokes the installed bridge to read its machine-local proxy credential; it does not require a `.sh` file or a provider API key. Run enrolment and Claude Code as the same Windows user.

If sync reports a Cowork organizational-directory provisioning error, follow the Windows provisioning step in [Connect Cowork](/documentation/connect-cowork), or ask your administrator to review the hosts enabled for your account. A failed sync is not successful Claude Code setup.

## 5. Choose how to launch

### Always-on routing

Keep the persistent configuration and start Claude Code normally:

```powershell
claude
```

For an exact model ID enabled for your account:

```powershell
claude --model gemini-2.5-flash
```

### Per-session routing

Remove persistent Claude Code routing while retaining the standalone file:

```powershell
& $Bridge uninstall --host claude-code
& $Bridge sync
$ClaudeSettings = Join-Path $env:APPDATA "systemprompt\claude-code-settings.json"
Get-Item -LiteralPath $ClaudeSettings
claude --settings "$ClaudeSettings" --model gemini-2.5-flash
```

Use an approved model ID. If you set an absolute `XDG_CONFIG_HOME`, the file is under that directory's `systemprompt` subdirectory instead; `& $Bridge status` reports the bridge configuration location. A custom `CLAUDE_CONFIG_DIR` is separate: use explicit `--settings` because enrolment's user fallback targets the default Claude Code directory.

A short **CMD** launch after completing PowerShell setup:

```bat
claude --settings "%APPDATA%\systemprompt\claude-code-settings.json" --model gemini-2.5-flash
```

PowerShell uses `$env:APPDATA`; CMD uses `%APPDATA%`. Managed settings can override either launch. See [Claude Code settings precedence](https://code.claude.com/docs/en/settings).

## 6. Verify routing and tools

1. Ask Claude Code: **Reply with SYSTEMPROMPT_CONNECTED. Do not use tools.**
2. Find the request in your gateway's [History](/admin/history), or ask an administrator to confirm its time, account, and model.
3. Run `claude plugin list`. In a fresh session, try a specific read-only tool your organization provides and verify its result and gateway activity separately.

A successful response, billing label, or visible plugin alone does not prove both inference routing and tool access.

## Troubleshooting and disconnect

- **`claude` is not recognized:** reopen PowerShell after installation and check the installation's PATH instructions.
- **Enrolment only says sync-only, or no settings file appears:** obtain a Windows bridge build that supports Claude Code enrolment, as described above, then repeat enrolment. Do not copy a WSL settings file or Unix helper.
- **Credential helper fails:** confirm the verified executable still exists at its enrolled path and PowerShell can run it. Re-enrol after moving or replacing it. Do not run the credential-helper command manually in shared logs: its stdout is a credential.
- **Connection refused or bad loopback secret:** keep the correct native bridge running. Windows and WSL have separate credentials and can compete for a loopback port. Stop the conflicting installation and re-enrol against the native bridge's actual port.
- **A different sign-in or model appears:** check conflicting `ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN`, provider-selection variables, and project/managed settings without printing secret values. Ask IT to resolve managed-policy conflicts.
- **Model rejected:** confirm the exact ID and your entitlement with an administrator.

To disconnect persistent Claude Code routing:

```powershell
& $Bridge uninstall --host claude-code
```

Also stop passing the standalone settings file. This does not sign out the bridge or disconnect Cowork.

## Cowork on Windows

Complete the download, gateway, and sign-in steps above, then follow [Connect Cowork](/documentation/connect-cowork) for Desktop enrolment and any required policy provisioning. Claude Code enrolment and Claude Desktop policy are separate.

## Claude Code in WSL

If you choose WSL, install both Claude Code and the Linux bridge inside the same approved distribution. Follow [Linux and WSL bridge installation](/documentation/bridge-install#linux-and-wsl), then [Connect Claude Code](/documentation/connect-claude-code) using its Linux commands.

The standalone settings file is normally `~/.config/systemprompt/claude-code-settings.json` **inside WSL**. Keep the WSL proxy or its systemd user service running. Native Windows and WSL installations have separate credentials, settings, and process lifecycles; one does not configure the other.
