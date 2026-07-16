# Install the systemprompt bridge via Scoop (Windows)

This doc installs the bridge on a Windows workstation using [Scoop](https://scoop.sh). The Scoop manifest pulls `systemprompt-bridge.exe` (renamed from `cowork` in v0.7.0) from the latest `bridge-v*` release.

The gateway (server) also ships on Scoop from `v0.5.0` — see [../install/scoop.md](../install/scoop.md) — or deploy it via Docker / GHCR / Helm on a Linux host.

Bucket: [`systempromptio/scoop-bucket`](https://github.com/systempromptio/scoop-bucket).

## Install Scoop (once)

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
Invoke-RestMethod -Uri https://get.scoop.sh | Invoke-Expression
```

## Install the bridge

```powershell
scoop bucket add systemprompt https://github.com/systempromptio/scoop-bucket
scoop install bridge
systemprompt-bridge --version
systemprompt-bridge gui      # launch the Desktop app
```

The Scoop package name is `bridge`; the binary it shims onto your `PATH` is `systemprompt-bridge.exe`. The legacy `cowork` package is retained for existing installs.

## Upgrade

```powershell
scoop update bridge
```

## Uninstall

```powershell
scoop uninstall bridge
```

## Configure against a gateway

Easiest path — launch the GUI and use the Setup wizard:

```powershell
systemprompt-bridge gui
```

Headless / scripted alternative:

```powershell
systemprompt-bridge login sp-live-... --gateway https://your-gateway.example.com
systemprompt-bridge install --apply --pubkey <base64>
systemprompt-bridge status
```

See [desktop-app.md](desktop-app.md), [device-auth.md](device-auth.md), and [windows-minimax-demo.md](windows-minimax-demo.md).

Docs: https://systemprompt.io/documentation/?utm_source=bridge-scoop&utm_medium=install_doc
