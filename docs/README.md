# systemprompt Documentation

User-facing docs for the two products published out of this repository.

## Two products

| | What it is | Where it runs | Tag series |
|---|---|---|---|
| **systemprompt-gateway** (server) | AI governance gateway — Rust HTTP server + Postgres + MCP extensions | Kubernetes / Docker / Linux VM / PaaS | `v*` |
| **systemprompt-cowork** (client) | `cowork` CLI for developer workstations — device auth + code-worktree ops against a running gateway | macOS / Windows / Linux laptops | `cowork-v*` |

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

## Install cowork (client)

| Platform | Doc |
|---|---|
| macOS | [cowork/install-macos.md](cowork/install-macos.md) |
| Windows | [cowork/install-windows.md](cowork/install-windows.md) |
| Scoop bucket (Windows) | [install/scoop.md](install/scoop.md) |

## Operating cowork

- [cowork/device-auth.md](cowork/device-auth.md) — Authentication modes for Claude for Work → gateway (PAT, session, mTLS).
- [cowork/windows-minimax-demo.md](cowork/windows-minimax-demo.md) — End-to-end Windows + gateway + MiniMax demo runbook.

## Licence

This template repository is **MIT** — see [LICENSE](../LICENSE). The compiled distributable links [`systemprompt-core`](https://github.com/systempromptio/systemprompt-core), which is **BSL-1.1** (source-available, converts to Apache 2.0 after 4 years; production use requires a commercial licence). OCI image labels, Helm chart metadata, and package `License` fields declare `MIT AND BUSL-1.1` to reflect both.
