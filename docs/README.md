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
| Dokploy blueprint | [install/dokploy.md](install/dokploy.md) | Dokploy self-host |
| Portainer app template | [install/portainer.md](install/portainer.md) | Portainer stacks |
| CapRover one-click app | [install/caprover.md](install/caprover.md) | CapRover self-host |
| CasaOS app | [install/casaos.md](install/casaos.md) | Home lab |
| Zeabur template | [install/zeabur.md](install/zeabur.md) | Zeabur PaaS |
| Northflank stack | [install/northflank.md](install/northflank.md) | Northflank PaaS |
| DigitalOcean 1-Click | [install/digitalocean.md](install/digitalocean.md) | Single-VM droplet (bundled Postgres) |

Maintainers: the release process (versioning, tag scheme, retention, rollback) is documented in [RELEASING.md](RELEASING.md).

### Running a second clone side-by-side

`just setup-local` accepts port overrides after the three key positions. To run a second clone on HTTP 8081 and Postgres 5433:

```bash
just setup-local <anthropic_key> "" "" 8081 5433
```

### Gateway configuration

- [gateway-routes.md](gateway-routes.md) — `/v1/messages` provider routing, CLI route configuration, route access control, and the extensible provider registry.
- [bridge-install.md](bridge-install.md) — install and configure the `systemprompt-bridge` credential helper for Claude for Work.

## Install the bridge (client)

The bridge — a single `systemprompt-bridge` binary with a native GUI on macOS and Windows. Renamed from `cowork → bridge` in v0.7.0; the binaries ship on the `bridge-v*` release tag series and install as the `bridge` Homebrew formula / Scoop package. The Claude Desktop host-integration label is still "Cowork", and the legacy `cowork` package is retained for existing installs.

| Platform | Doc |
|---|---|
| Overview / GUI tour | [cowork/desktop-app.md](cowork/desktop-app.md) |
| macOS (binary) | [cowork/install-macos.md](cowork/install-macos.md) |
| Windows (`.exe`) | [cowork/install-windows.md](cowork/install-windows.md) |
| Scoop bucket (Windows) | [cowork/scoop.md](cowork/scoop.md) |
| Linux / headless (Claude Code CLI) | [cowork/claude-code-linux.md](cowork/claude-code-linux.md) |

## Operating the bridge

- [cowork/desktop-app.md](cowork/desktop-app.md) — Setup wizard, agents tab (per-agent enable/disable, Codex CLI alongside Claude Desktop), marketplace, settings, activity drawer, diagnostics export.
- [cowork/device-auth.md](cowork/device-auth.md) — Authentication modes for Claude for Work → gateway (PAT, session, mTLS). Updated for `SP_BRIDGE_*` env vars and the new `systemprompt-bridge.toml` config path.
- [cowork/windows-minimax-demo.md](cowork/windows-minimax-demo.md) — End-to-end Windows + gateway + MiniMax demo runbook.
- [cowork/claude-code-linux.md](cowork/claude-code-linux.md) — Route the Claude Code CLI through the gateway on Linux/headless via `systemprompt-bridge proxy`. Includes the `docker/claude-code-clean-room/` from-scratch test.

## Licence

This template repository is **MIT** — see [LICENSE](../LICENSE). The compiled distributable links [`systemprompt-core`](https://github.com/systempromptio/systemprompt-core), which is **BSL-1.1** (source-available, converts to Apache 2.0 after 4 years; production use requires a commercial licence). OCI image labels, Helm chart metadata, and package `License` fields declare `MIT AND BUSL-1.1` to reflect both.
