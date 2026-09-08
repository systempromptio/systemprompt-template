<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://systemprompt.io/files/images/logo.svg">
  <source media="(prefers-color-scheme: light)" srcset="https://systemprompt.io/files/images/logo-dark.svg">
  <img src="https://systemprompt.io/files/images/logo-dark.svg" alt="systemprompt.io" width="380">
</picture>

# Your AI control plane. Clone, configure, run.

**Govern your AI. Build your own capabilities. Run it on your infrastructure.**

Identity, model access, MCP tool execution, policy and audit in a Rust runtime you operate. Start with a working governance system, then compile your company's capabilities into it.

[Quick start](#quick-start) · [Run the proof](#run-the-proof) · [Core](https://github.com/systempromptio/systemprompt-core) · [Documentation](https://systemprompt.io/documentation/)

</div>

## Start with control

SystemPrompt puts a policy boundary between your AI clients and the models and tools you connect. Permissions follow the authenticated user; audit records connect identity, policy decisions, tool activity and inference cost.

| What you control | How |
|---|---|
| Model access | Route supported clients through the gateway and grant or deny access per user. |
| Tool execution | Check scope, credential patterns, blocklists and rate limits on governed MCP calls before execution. |
| Audit data | Keep request records and correlated tool traces in your PostgreSQL database. |
| Deployment | Run the compiled runtime on your infrastructure, with PostgreSQL as its required database. |
| Domain capabilities | Add Rust extensions for your integrations, policies and application behavior. |

This repository is the **evaluation template**: Core plus configuration, an admin UI, extensions and executable demos. Use it to evaluate enforcement and as the starting point for your own deployment.

## Quick start

Install Docker, [just](https://just.systems/) and Rust through rustup. The repo pins its toolchain in [rust-toolchain.toml](rust-toolchain.toml); the workspace requires Rust 1.96+. Local setup provisions PostgreSQL 18 and asks for an AI provider key.

```bash
git clone https://github.com/systempromptio/systemprompt-template
cd systemprompt-template
just setup-local
just start
```

Open **http://localhost:8080**. Setup builds the binary, provisions the local profile and database, runs migrations and publishes assets. Inference uses your chosen provider account.

For non-interactive setup, pass provider keys as documented in the [installation guide](docs/README.md). That guide also covers containers, deployment platforms and alternate ports.

## Run the proof

Start with an allowed tool call, then a denied one:

```bash
./demo/00-preflight.sh
./demo/01-seed-data.sh
./demo/governance/01-happy-path.sh
./demo/governance/05-governance-denied.sh
./demo/governance/06-secret-breach.sh
```

Inspect the resulting decisions and traces in the admin UI. The [demo index](demo/README.md) explains prerequisites and which scripts invoke paid models.

For a third-party client, follow the [Pi walkthrough](examples/pi/WALKTHROUGH.md). Connect Pi to the gateway, make a request, disable that user's model in **Model Selection** (`/admin/models`), then retry. The denied request appears in the audit view (`/admin/requests`). Access tokens are managed at `/admin/access-tokens`.

<img src="docs/images/pi-demo-model-selection.png" alt="Model Selection dashboard showing per-user model permissions and usage during the Pi demo" width="900">

Gateway routing governs inference sent through that endpoint. The Pi integration also installs hooks for prompt and local tool checks; connecting a model endpoint alone does not govern arbitrary local commands.

## Build your capabilities into the runtime

```text
SystemPrompt Core + your Rust extensions + your configuration
                            ↓
                  Your compiled deployment
                            ↓
                       PostgreSQL
```

Core supplies shared identity, gateway, MCP, policy and audit capabilities. You supply the domain behavior. Configure agents, providers, MCP servers and scheduled work under `services/`; implement application capabilities in `extensions/`.

Extensions register at compile/link time and are discovered and dependency-validated at startup. They can contribute routes, tools, jobs, schemas and migrations. Compiled extensions are trusted code sharing the runtime process; external MCP servers can run as separate processes.

The host entry point stays thin because it delegates to Core. Explore the [Core API](https://docs.rs/systemprompt) and this repo's [web extension](extensions/web/README.md) to see the composition model.

## Deployment and security boundaries

Keep governance and audit storage inside your perimeter. Air-gapped operation requires locally available models, tools and dependencies; cloud inference still sends requests to the selected provider.

Tool credentials can be supplied to subprocess environments without requiring their inclusion in model context. Pattern scanning checks for recognizable credentials in governed arguments. Tool code, its outputs and its network access remain part of your security boundary.

See the [deployment guides](docs/README.md) and [reproducible performance demos](demo/performance/README.md) for evaluation details.

## License and next steps

This template is [MIT licensed](LICENSE). Core is [BSL-1.1](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE), available for evaluation, testing and non-production use under its license terms. Production use requires a commercial license; each Core version converts to Apache-2.0 four years after publication.

[Evaluate the hosted demo](https://demo.systemprompt.io) · [Inspect Core](https://github.com/systempromptio/systemprompt-core) · [Discuss production licensing](mailto:ed@systemprompt.io)
