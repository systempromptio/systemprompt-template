# systemprompt Documentation

User-facing docs for the two products published out of this repository.

## Two products

| | What it is | Where it runs | Tag series |
|---|---|---|---|
| **systemprompt-gateway** (server) | AI governance gateway — Rust HTTP server + Postgres + MCP extensions | Kubernetes / Docker / Linux VM / PaaS | `v*` |
| **systemprompt-bridge** (client) | `systemprompt-bridge` CLI for developer workstations — device auth + code-worktree ops against a running gateway | macOS / Windows / Linux laptops | `bridge-v*` |

Pick the product first, then the install channel.

---

## Install the gateway (server)

Choose the channel that fits your environment. Each doc is a copy-paste recipe.

| Channel | Doc | Audience |
|---|---|---|
| GitHub Container Registry | [install/ghcr.md](install/ghcr.md) | Primary public image surface |
| Binary (`curl \| sh`) | [install/binary.md](install/binary.md) | Bare-metal, VM, one-shot installs |
| Homebrew tap | [install/homebrew.md](install/homebrew.md) | macOS servers / development |
| Helm chart | [install/helm.md](install/helm.md) | Kubernetes |
| Nix flake | [install/nix.md](install/nix.md) | NixOS / Nix users |
| Railway template | [install/railway.md](install/railway.md) | Railway PaaS |
| Render blueprint | [install/render.md](install/render.md) | Render PaaS |
| Coolify template | [install/coolify.md](install/coolify.md) | Coolify self-host |

## Install the bridge (client)

The bridge — a single `systemprompt-bridge` binary with a native GUI on macOS and Windows. Renamed from `cowork → bridge` in v0.7.0; the binaries ship on the `bridge-v*` release tag series and install as the `bridge` Homebrew formula / Scoop package. The Claude Desktop host-integration label is still "Cowork", and the legacy `cowork` package is retained for existing installs.

| Platform | Doc |
|---|---|
| Overview / GUI tour | [cowork/desktop-app.md](cowork/desktop-app.md) |
| macOS (binary) | [cowork/install-macos.md](cowork/install-macos.md) |
| Windows (`.exe`) | [cowork/install-windows.md](cowork/install-windows.md) |
| Scoop bucket (Windows) | [cowork/scoop.md](cowork/scoop.md) |

## Operating the bridge

- [cowork/desktop-app.md](cowork/desktop-app.md) — Setup wizard, agents tab (per-agent enable/disable, Codex CLI alongside Claude Desktop), marketplace, settings, activity drawer, diagnostics export.
- [cowork/device-auth.md](cowork/device-auth.md) — Authentication modes for Claude for Work → gateway (PAT, session, mTLS). Updated for `SP_BRIDGE_*` env vars and the new `systemprompt-bridge.toml` config path.
- [cowork/windows-minimax-demo.md](cowork/windows-minimax-demo.md) — End-to-end Windows + gateway + MiniMax demo runbook.

## Licence

This template repository is **MIT** — see [LICENSE](../LICENSE). The compiled distributable links [`systemprompt-core`](https://github.com/systempromptio/systemprompt-core), which is **BSL-1.1** (source-available, converts to Apache 2.0 after 4 years; production use requires a commercial licence). OCI image labels, Helm chart metadata, and package `License` fields declare `MIT AND BUSL-1.1` to reflect both.
