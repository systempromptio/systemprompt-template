# systemprompt Documentation

User-facing docs for the gateway published out of this repository.

## The product

| | What it is | Where it runs | Tag series |
|---|---|---|---|
| **systemprompt-gateway** (server) | AI governance gateway — Rust HTTP server + Postgres + MCP extensions | Kubernetes / Docker / Linux VM / PaaS | `v*` |

Clients authenticate with a personal access token issued on
`/admin/access-tokens`; see [`examples/pi/`](../examples/pi/) for a worked
client setup.

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

### Docker Compose and secrets

- [install/docker.md](install/docker.md): run the published image with the repository's compose file, remote Postgres, multi-node profiles, upgrades and rollbacks, and pointing a bridge at a gateway.
- [install/required-secrets.md](install/required-secrets.md): the provider secrets the shipped catalog reads, how to set them, and how to prove one with a real dispatch.

### Gateway configuration

- [gateway-routes.md](gateway-routes.md): `/v1/messages` provider routing, CLI route configuration, client attribution, route access control, and the extensible provider registry.

## Use the system

- [CONFIGURED-CONNECTORS.md](CONFIGURED-CONNECTORS.md): personal MCP accounts and the connector OAuth block.
- [integrations/centralized-mcp.md](integrations/centralized-mcp.md): server-held connector accounts for third-party MCP servers, provisioning and acceptance checks.
- [kits-on-another-instance.md](kits-on-another-instance.md): shipping a services kit to another instance.
- [profile.schema.json](profile.schema.json): the profile schema.
- [evals.md](evals.md) and [skill-optimization.md](skill-optimization.md): evaluating and improving skills.

## Maintain the repository

- [BRANCHING.md](BRANCHING.md) and [RELEASING.md](RELEASING.md): branches, the frozen promotion and the release procedure.
- [development/](development/): developer notes for the console and its ports.
- [tech-debt.md](tech-debt.md): recorded platform debt — the artifact chain, sync, data lifecycle — with a local activity-reset runbook.

## Licence

This template repository is **MIT**: see [LICENSE](../LICENSE). The compiled distributable links [`systemprompt-core`](https://github.com/systempromptio/systemprompt-core), which is **BSL-1.1** (source-available, converts to Apache 2.0 after 4 years; production use requires a commercial licence). OCI image labels, Helm chart metadata, and package `License` fields declare `MIT AND BUSL-1.1` to reflect both.
