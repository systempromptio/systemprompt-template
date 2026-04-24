# Install cowork via Scoop (Windows)

`cowork` is the **client-side CLI** that talks to a running [systemprompt-gateway](../install/ghcr.md). This doc installs cowork on a Windows workstation using [Scoop](https://scoop.sh).

The gateway (server) does not ship a Windows binary — deploy it via Docker / GHCR / Helm / Linux package on a Linux host.

Bucket: [`systempromptio/scoop-bucket`](https://github.com/systempromptio/scoop-bucket).

## Install Scoop (once)

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
Invoke-RestMethod -Uri https://get.scoop.sh | Invoke-Expression
```

## Install cowork

```powershell
scoop bucket add systemprompt https://github.com/systempromptio/scoop-bucket
scoop install cowork
cowork --version
```

## Upgrade

```powershell
scoop update cowork
```

## Uninstall

```powershell
scoop uninstall cowork
```

## Configure against a gateway

```powershell
cowork config set gateway.url https://your-gateway.example.com
cowork login
```

See [device-auth.md](device-auth.md) and [windows-minimax-demo.md](windows-minimax-demo.md).

Docs: https://systemprompt.io/documentation/?utm_source=cowork-scoop&utm_medium=install_doc
