# Install Cowork via Scoop (Windows)

This doc installs the Cowork Desktop app on a Windows workstation using [Scoop](https://scoop.sh). The Scoop manifest pulls `systemprompt-bridge.exe` (renamed from `cowork` in v0.7.0) from the latest `cowork-v*` release.

The gateway (server) does not ship a Windows binary — deploy it via Docker / GHCR / Helm / Linux package on a Linux host.

Bucket: [`systempromptio/scoop-bucket`](https://github.com/systempromptio/scoop-bucket).

## Install Scoop (once)

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
Invoke-RestMethod -Uri https://get.scoop.sh | Invoke-Expression
```

## Install Cowork

```powershell
scoop bucket add systemprompt https://github.com/systempromptio/scoop-bucket
scoop install cowork
systemprompt-bridge --version
systemprompt-bridge gui      # launch the Desktop app
```

The Scoop package name is `cowork` (unchanged); the binary it shims onto your `PATH` is `systemprompt-bridge.exe`.

## Upgrade

```powershell
scoop update cowork
```

## Uninstall

```powershell
scoop uninstall cowork
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

Docs: https://systemprompt.io/documentation/?utm_source=cowork-scoop&utm_medium=install_doc
