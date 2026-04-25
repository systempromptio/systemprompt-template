# Install cowork on Windows

`cowork` is the **client-side CLI** that runs on a developer workstation and talks to a running [systemprompt-gateway](../install/ghcr.md). It handles device auth (Claude for Work / MCP device flow) and code-worktree operations.

If you're looking to deploy the **server** (the gateway), see [../install/](../install/) instead — cowork is a separate product.

## Option 1 — Scoop bucket (recommended)

```powershell
scoop bucket add systemprompt https://github.com/systempromptio/scoop-bucket
scoop install systemprompt/cowork
cowork --version
```

## Option 2 — direct download

Cowork bundles are published on the template repo under the `cowork-v*` tag series:

```powershell
# Raw .exe binary (no zip wrapper)
Invoke-WebRequest `
  -Uri https://github.com/systempromptio/systemprompt-template/releases/download/cowork-v0.4.0/systemprompt-cowork-x86_64-pc-windows-msvc.exe `
  -OutFile cowork.exe
Move-Item .\cowork.exe C:\Windows\System32\cowork.exe
```

Verify the SHA256:

```powershell
Invoke-WebRequest `
  -Uri https://github.com/systempromptio/systemprompt-template/releases/download/cowork-v0.4.0/SHA256SUMS.cowork `
  -OutFile SHA256SUMS.cowork
Get-FileHash C:\Windows\System32\cowork.exe -Algorithm SHA256
# Compare against the line ending in `systemprompt-cowork-x86_64-pc-windows-msvc.exe` in SHA256SUMS.cowork
```

Verify the cosign signature (requires cosign.exe):

```powershell
cosign verify-blob `
  --certificate-identity-regexp 'https://github.com/systempromptio/systemprompt-core/' `
  --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' `
  --signature SHA256SUMS.cowork.sig `
  --certificate SHA256SUMS.cowork.pem `
  SHA256SUMS.cowork
```

## Configure against a running gateway

```powershell
cowork config set gateway.url https://your-gateway.example.com
cowork login
```

See [device-auth.md](device-auth.md) for the auth-mode options.

See [windows-minimax-demo.md](windows-minimax-demo.md) for an end-to-end demo runbook.

## Uninstall

```powershell
scoop uninstall cowork                 # Scoop
Remove-Item C:\Windows\System32\cowork.exe   # manual install
```

## Links

- [systemprompt.io](https://systemprompt.io/?utm_source=cowork-windows&utm_medium=install_doc)
- [Documentation](https://systemprompt.io/documentation/?utm_source=cowork-windows&utm_medium=install_doc)
- [systemprompt-template on GitHub](https://github.com/systempromptio/systemprompt-template)
- [LICENSE](../../LICENSE)
