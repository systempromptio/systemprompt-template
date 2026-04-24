# systemprompt Documentation

User-facing docs for installing and operating the `systemprompt` AI governance gateway.

## Install

Choose the channel that fits your environment. Each doc is a copy-paste recipe.

| Channel | Doc | Audience |
|---|---|---|
| GitHub Container Registry | [install/ghcr.md](install/ghcr.md) | **Recommended** — primary public image surface |
| Docker Hub *(coming soon)* | [install/docker.md](install/docker.md) | Pending — requires a paid Docker Hub Team subscription |
| Binary (GitHub Releases + `curl \| sh`) | [install/binary.md](install/binary.md) | Bare-metal, VM, one-shot installs |
| Homebrew tap | [install/homebrew.md](install/homebrew.md) | macOS + Linuxbrew |
| Scoop bucket | [install/scoop.md](install/scoop.md) | Windows developers |
| Helm chart | [install/helm.md](install/helm.md) | Kubernetes |
| APT repo *(deferred)* | [install/apt.md](install/apt.md) | Debian / Ubuntu — planned |
| RPM repo *(deferred)* | [install/rpm.md](install/rpm.md) | RHEL / Rocky / Fedora — planned |
| Winget *(deferred)* | [install/winget.md](install/winget.md) | Windows 11 — planned |
| Nix flake | [install/nix.md](install/nix.md) | NixOS / Nix users |
| Railway template | [install/railway.md](install/railway.md) | Railway PaaS |
| Render blueprint | [install/render.md](install/render.md) | Render PaaS |
| Coolify template | [install/coolify.md](install/coolify.md) | Coolify self-host |

## Operations

- [cowork-device-auth.md](cowork-device-auth.md) — Authentication modes for Claude for Work → gateway (PAT, session, mTLS).
- [windows-cowork-minimax-demo.md](windows-cowork-minimax-demo.md) — End-to-end Windows + gateway + MiniMax demo runbook.

## Licence

This template repository is **MIT** — see [LICENSE](../LICENSE). The compiled distributable links [`systemprompt-core`](https://github.com/systempromptio/systemprompt-core), which is **BSL-1.1** (source-available, converts to Apache 2.0 after 4 years; production use requires a commercial licence). OCI image labels, Helm chart metadata, and package `License` fields declare `MIT AND BUSL-1.1` to reflect both.
