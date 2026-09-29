---
title: "Downloads"
description: "Download systemprompt Bridge for your platform, compare its SHA-256 checksum, and follow the installation guide."
author: "systemprompt.io"
slug: "downloads"
kind: "reference"
public: true
tags: ["documentation", "getting-started", "bridge"]
published_at: "2026-09-02"
updated_at: "2026-09-28"
related_playbooks:
  - title: "Connect Claude Code"
    url: "/documentation/connect-claude-code"
  - title: "Connect Cowork"
    url: "/documentation/connect-cowork"
  - title: "Install the Desktop Bridge"
    url: "/documentation/bridge-install"
---

# Downloads

Download systemprompt Bridge from the gateway you intend to use. Choose your platform, verify the checksum, and follow its setup guide. A checksum confirms that your download matches the published file; it does not replace OS signature verification or your organization's software-approval policy.

These files are served from your gateway's `/files/downloads` directory only when your administrator has built and published them there. If a link below returns an error, or your Profile page says that desktop client downloads are not configured, ask your administrator for an approved build.

| Platform | Download | SHA-256 checksum |
|---|---|---|
| macOS | [systemprompt-bridge-macos.dmg](/files/downloads/systemprompt-bridge-macos.dmg) | [Checksum](/files/downloads/systemprompt-bridge-macos.dmg.sha256) |
| Windows x86_64 | [systemprompt-bridge-windows.exe](/files/downloads/systemprompt-bridge-windows.exe) | [Checksum](/files/downloads/systemprompt-bridge-windows.exe.sha256) |
| Linux x86_64 | [systemprompt-bridge-linux-x86_64.tar.gz](/files/downloads/systemprompt-bridge-linux-x86_64.tar.gz) | [Checksum](/files/downloads/systemprompt-bridge-linux-x86_64.tar.gz.sha256) |
| Linux aarch64 | [systemprompt-bridge-linux-aarch64.tar.gz](/files/downloads/systemprompt-bridge-linux-aarch64.tar.gz) | [Checksum](/files/downloads/systemprompt-bridge-linux-aarch64.tar.gz.sha256) |
| Linux and WSL installer | [install.sh](/files/downloads/install.sh) | The installer checks the downloaded bridge archive |

## Verify your download

Save the binary or archive and its checksum in the same directory. Keep the published filenames; browsers sometimes add a number to duplicate downloads.

### macOS

From the directory containing the download:

```bash
shasum -a 256 -c systemprompt-bridge-macos.dmg.sha256
```

Continue only if it reports `OK`. Follow [Install on macOS](/documentation/install-macos) for app installation and Gatekeeper verification. Do not disable Gatekeeper or remove quarantine attributes to work around a rejected download.

### Windows PowerShell

In the directory containing both files, compare the expected and actual values:

```powershell
$ExpectedHash = ((Get-Content ".\systemprompt-bridge-windows.exe.sha256" -Raw).Trim() -split '\s+')[0]
$ActualHash = (Get-FileHash ".\systemprompt-bridge-windows.exe" -Algorithm SHA256).Hash
if ($ExpectedHash -notmatch '^[0-9a-fA-F]{64}$' -or $ActualHash -ne $ExpectedHash) {
    throw "Checksum verification failed. Do not run this download."
}
"Checksum verified."
```

Follow [Install on Windows](/documentation/install-windows). The executable may be unsigned, so SmartScreen can warn on first run. If Windows blocks the executable, ask IT to verify and approve it; do not disable SmartScreen or endpoint protection.

### Linux

For x86_64:

```bash
sha256sum -c systemprompt-bridge-linux-x86_64.tar.gz.sha256
```

For aarch64:

```bash
sha256sum -c systemprompt-bridge-linux-aarch64.tar.gz.sha256
```

Continue only if the check reports `OK`. The [Linux and WSL installation guide](/documentation/bridge-install#linux-and-wsl) covers prerequisites, the installer, trusted manifest keys, sign-in, and background services.

## Next steps

- [Install on macOS](/documentation/install-macos)
- [Install on Windows](/documentation/install-windows)
- [Connect Claude Code](/documentation/connect-claude-code), including model selection and per-session settings
- [Connect Cowork](/documentation/connect-cowork)
- [Bridge setup and troubleshooting](/documentation/bridge-install)

If a file is unavailable or its checksum does not match, stop and report the download URL and verification error to your administrator. Do not substitute a binary from an unrelated gateway.
