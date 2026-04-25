<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://systemprompt.io/files/images/logo.svg">
  <source media="(prefers-color-scheme: light)" srcset="https://systemprompt.io/files/images/logo-dark.svg">
  <img src="https://systemprompt.io/files/images/logo-dark.svg" alt="systemprompt.io" width="320">
</picture>

# systempromptio/scoop-bucket

**Scoop bucket for the systemprompt AI governance gateway on Windows.**

The governance layer for AI agents — a single compiled Rust binary that authenticates, authorises, rate-limits, logs, and costs every AI interaction. Self-hosted, air-gap capable, provider-agnostic.

[**systemprompt.io**](https://systemprompt.io) · [**Documentation**](https://systemprompt.io/documentation/) · [**Guides**](https://systemprompt.io/guides) · [**Main repo**](https://github.com/systempromptio/systemprompt-template) · [**Discord**](https://discord.gg/wkAbSuPWpr)

[![Template · MIT](https://img.shields.io/badge/template-MIT-16a34a?style=flat-square)](https://github.com/systempromptio/systemprompt-template/blob/main/LICENSE)
[![Core · BSL--1.1](https://img.shields.io/badge/core-BSL--1.1-2b6cb0?style=flat-square)](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE)

</div>

---

## Install

```powershell
scoop bucket add systemprompt https://github.com/systempromptio/scoop-bucket
scoop install gateway
```

Full install + service registration docs: [systemprompt-template/docs/install/scoop.md](https://github.com/systempromptio/systemprompt-template/blob/main/docs/install/scoop.md).

## What you get

Three signed binaries on your PATH:

- `systemprompt.exe` — AI governance gateway
- `systemprompt-mcp-agent.exe` — MCP agent server
- `systemprompt-mcp-marketplace.exe` — MCP marketplace server

Every tool call authenticated, scoped, secret-scanned, rate-limited, and audited before the tool process spawns. Built for SOC 2, ISO 27001, HIPAA, and the OWASP Agentic Top 10.

## Upgrade

```powershell
scoop update gateway
```

## Other install channels

Docker, GHCR, Helm, APT, RPM, Winget, Homebrew, Nix, Railway, Render, Coolify — see the [install docs index](https://github.com/systempromptio/systemprompt-template/tree/main/docs/install).

## Licence

Bucket manifests: MIT (this repo). Compiled binary: `MIT AND BUSL-1.1` — template code is [MIT](https://github.com/systempromptio/systemprompt-template/blob/main/LICENSE); the compiled binary links `systemprompt-core` which is [BSL-1.1](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE).
