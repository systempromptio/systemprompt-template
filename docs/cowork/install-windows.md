# Install the systemprompt bridge on Windows

The bridge is the **client-side companion** to a running [systemprompt-gateway](../install/ghcr.md). On Windows it ships as `systemprompt-bridge.exe` — a single native binary that exposes both a credential-helper CLI and a Desktop GUI (winit + wry, MSVC build).

If you're looking to deploy the **server** (the gateway), see [../install/](../install/) instead.

For the full GUI tour, see [desktop-app.md](desktop-app.md).

> **Naming note.** The crate and binary were renamed `cowork → bridge` in v0.7.0; the Windows executable is `systemprompt-bridge.exe`. The Claude Desktop host-integration label is still "Cowork". The Scoop package is now named `bridge`; the legacy `cowork` package is retained for existing installs.

## Option 1 — Scoop bucket (recommended)

```powershell
scoop bucket add systemprompt https://github.com/systempromptio/scoop-bucket
scoop install systemprompt/bridge
systemprompt-bridge --version
systemprompt-bridge gui      # opens the Desktop app
```

## Option 2 — direct download

The `systemprompt-bridge` binaries are published on the template repo under the `bridge-v*` tag series:

```powershell
$dir = "C:\Program Files\systemprompt"
New-Item -ItemType Directory -Force -Path $dir | Out-Null

Invoke-WebRequest `
  -Uri https://github.com/systempromptio/systemprompt-template/releases/download/bridge-v0.10.0/systemprompt-bridge-x86_64-pc-windows-msvc.exe `
  -OutFile "$dir\systemprompt-bridge.exe"

[Environment]::SetEnvironmentVariable("PATH", "$env:PATH;$dir", "User")
```

Open a new terminal so the PATH change takes effect. Windows SmartScreen will flag the unsigned binary on first run → **More info** → **Run anyway**. (The binary is cosign-signed but not Authenticode-signed yet.)

Verify the SHA256:

```powershell
Invoke-WebRequest `
  -Uri https://github.com/systempromptio/systemprompt-template/releases/download/bridge-v0.10.0/SHA256SUMS.bridge `
  -OutFile SHA256SUMS.bridge
Get-FileHash "$dir\systemprompt-bridge.exe" -Algorithm SHA256
# Compare against the line ending in `systemprompt-bridge-x86_64-pc-windows-msvc.exe`
```

Verify the cosign signature (requires `cosign.exe`):

```powershell
cosign verify-blob `
  --certificate-identity-regexp 'https://github.com/systempromptio/systemprompt-core/' `
  --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' `
  --signature SHA256SUMS.bridge.sig `
  --certificate SHA256SUMS.bridge.pem `
  SHA256SUMS.bridge
```

## First-run

Launch the Desktop GUI:

```powershell
systemprompt-bridge gui
```

The **Setup wizard** opens automatically on first run. Two steps:

1. Paste the gateway URL and a PAT (or pick session / mTLS).
2. Enable each host integration you want managed (Claude Desktop, Codex CLI, …). Hosts default to **disabled** — nothing is silently probed.

The wizard can be re-run any time from **Settings → Re-run setup**. Once setup completes, the proxy starts on `127.0.0.1:48217` and the activity drawer shows live request flow.

CLI alternative (no GUI — for CI or headless service install):

```powershell
systemprompt-bridge login sp-live-... --gateway https://your-gateway.example.com
systemprompt-bridge install --apply --pubkey <base64>
systemprompt-bridge status
```

See [device-auth.md](device-auth.md) for the auth-mode options.

For a full Windows runbook against a MiniMax-routed gateway, see [windows-minimax-demo.md](windows-minimax-demo.md).

## Uninstall

```powershell
scoop uninstall bridge                                    # Scoop
Remove-Item "C:\Program Files\systemprompt\systemprompt-bridge.exe"   # manual install
```

To also wipe credentials and cache:

```powershell
systemprompt-bridge uninstall --purge
```

This removes `%APPDATA%\systemprompt\systemprompt-bridge.toml`, `%LOCALAPPDATA%\systemprompt-bridge\cache.json`, and `%APPDATA%\systemprompt\agents.json`.

## Links

- [Desktop app overview](desktop-app.md)
- [Device auth modes](device-auth.md)
- [systemprompt.io](https://systemprompt.io/?utm_source=bridge-windows&utm_medium=install_doc)
- [Documentation](https://systemprompt.io/documentation/?utm_source=bridge-windows&utm_medium=install_doc)
- [systemprompt-template on GitHub](https://github.com/systempromptio/systemprompt-template)
- [LICENSE](../../LICENSE)
