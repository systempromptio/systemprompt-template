<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://systemprompt.io/files/images/logo.svg">
  <source media="(prefers-color-scheme: light)" srcset="https://systemprompt.io/files/images/logo-dark.svg">
  <img src="https://systemprompt.io/files/images/logo-dark.svg" alt="systemprompt.io" width="320">
</picture>

# systempromptio/homebrew-tap

**Homebrew tap for the systemprompt AI governance gateway.**

The governance layer for AI agents — a single compiled Rust binary that authenticates, authorises, rate-limits, logs, and costs every AI interaction. Self-hosted, air-gap capable, provider-agnostic.

[**systemprompt.io**](https://systemprompt.io) · [**Documentation**](https://systemprompt.io/documentation/) · [**Guides**](https://systemprompt.io/guides) · [**Main repo**](https://github.com/systempromptio/systemprompt-template) · [**Discord**](https://discord.gg/wkAbSuPWpr)

[![Template · MIT](https://img.shields.io/badge/template-MIT-16a34a?style=flat-square)](https://github.com/systempromptio/systemprompt-template/blob/main/LICENSE)
[![Core · BSL--1.1](https://img.shields.io/badge/core-BSL--1.1-2b6cb0?style=flat-square)](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE)

</div>

---

## Install

```bash
brew tap systempromptio/tap
brew install gateway
```

Or in one line:

```bash
brew install systempromptio/tap/gateway
```

Run as a service:

```bash
brew services start gateway
```

Full install + configuration docs: [systemprompt-template/docs/install/homebrew.md](https://github.com/systempromptio/systemprompt-template/blob/main/docs/install/homebrew.md).

## What you get

The formula installs three signed binaries to `$(brew --prefix)/bin`:

- `systemprompt` — the AI governance gateway
- `systemprompt-mcp-agent` — MCP agent server
- `systemprompt-mcp-marketplace` — MCP marketplace server

Every tool call authenticated, scoped, secret-scanned, rate-limited, and audited before the tool process spawns. ~50 MB Rust binary, one PostgreSQL. Built for SOC 2, ISO 27001, HIPAA, and the OWASP Agentic Top 10.

## Other install channels

Docker, GHCR, Helm, APT, RPM, Winget, Scoop, Nix, Railway, Render, Coolify — see the [install docs index](https://github.com/systempromptio/systemprompt-template/tree/main/docs/install).

## Why a tap, not Homebrew core

Faster iteration than Homebrew core, and no pinning our release cadence to homebrew/core review queues. The compiled binary is `MIT AND BUSL-1.1` (template MIT + linked BSL-1.1 core).

## Licence

Formula: MIT (this repo). Compiled binary: `MIT AND BUSL-1.1` — template code is [MIT](https://github.com/systempromptio/systemprompt-template/blob/main/LICENSE); the compiled binary links `systemprompt-core` which is [BSL-1.1](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE) (converts to Apache 2.0 after 4 years; production use of the compiled binary requires a commercial core licence).
