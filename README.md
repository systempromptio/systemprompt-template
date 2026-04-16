<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://systemprompt.io/files/images/logo.svg">
  <source media="(prefers-color-scheme: light)" srcset="https://systemprompt.io/files/images/logo-dark.svg">
  <img src="https://systemprompt.io/files/images/logo-dark.svg" alt="systemprompt.io" width="380">
</picture>

# Own how your organization uses AI.

### The narrow waist between your AI and everything it touches. Self-hosted. Air-gapped. Every interaction governed and provable.

[![Built on systemprompt-core](https://img.shields.io/badge/built%20on-systemprompt--core-2b6cb0?style=flat-square)](https://github.com/systempromptio/systemprompt-core)
[![Template · MIT](https://img.shields.io/badge/template-MIT-16a34a?style=flat-square)](LICENSE)
[![Core · BSL--1.1](https://img.shields.io/badge/core-BSL--1.1-2b6cb0?style=flat-square)](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75+-f97316?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![PostgreSQL 18+](https://img.shields.io/badge/postgres-18+-336791?style=flat-square&logo=postgresql&logoColor=white)](https://www.postgresql.org/)

[**systemprompt.io**](https://systemprompt.io) · [**Documentation**](https://systemprompt.io/documentation/) · [**Guides**](https://systemprompt.io/guides) · [**Discord**](https://discord.gg/wkAbSuPWpr)

</div>

---

## This is the local evaluation

**systemprompt.io is AI Governance Infrastructure** — the governance layer for AI agents. A single compiled Rust binary that authenticates, authorises, rate-limits, logs, and costs every AI interaction. Self-hosted, air-gap capable, provider-agnostic.

This repo is how you evaluate it. Clone it, run it on your own machine, bring your own AI key, and watch every claim below execute against 40+ scripted demos. **The terminal recordings are real captures of those demos running — not mockups.** Regenerate them yourself with `just record-svgs`.

> Template: MIT. [systemprompt-core](https://github.com/systempromptio/systemprompt-core): BSL-1.1 — free for evaluation and non-production use. Production requires a commercial license.

---

## Evaluate it in three commands

```bash
just build                                               # 1. compile the workspace
just setup-local <anthropic> <openai> <gemini>           # 2. profile + Postgres + publish
just start                                               # 3. serve governance, agents, MCP, admin, API
```

Open **http://localhost:8080** and run `systemprompt --help`. Point Claude Code, Claude Desktop, or any MCP client at it — permissions follow the user, not the client.

**Prerequisites.** Rust 1.75+ · [`just`](https://just.systems) · Docker · `jq` · `yq` · one of an Anthropic, OpenAI, or Gemini API key · free ports `8080` and `5432`. Running a second clone side-by-side? `just setup-local <key> "" "" 8081 5433`.

---

## Infrastructure

One binary. One database. Deploys anywhere.

### Self-hosted deployment

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-self-hosted.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-self-hosted.svg">
  <img src="demo/recording/svg/output/dark/infra-self-hosted.svg" alt="Self-hosted deployment — terminal recording" width="820">
</picture>

> 50MB Rust binary, 4 in-process services, 144 database tables, zero sidecars. → [`svg-infra-self-hosted.sh`](demo/recording/svg/svg-infra-self-hosted.sh) · [Learn more](https://systemprompt.io/features/self-hosted-ai-platform)

### Deploy anywhere

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-deploy-anywhere.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-deploy-anywhere.svg">
  <img src="demo/recording/svg/output/dark/infra-deploy-anywhere.svg" alt="Deploy anywhere — terminal recording" width="820">
</picture>

> Same binary runs local, Docker, cloud, or air-gapped. Config follows the profile. → [`svg-infra-deploy-anywhere.sh`](demo/recording/svg/svg-infra-deploy-anywhere.sh) · [Learn more](https://systemprompt.io/features/self-hosted-ai-platform)

### One CLI, every domain

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-control-plane.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-control-plane.svg">
  <img src="demo/recording/svg/output/dark/infra-control-plane.svg" alt="Unified control plane — terminal recording" width="820">
</picture>

> 8 domains — govern, observe, manage — all from one binary, one CLI. → [`svg-infra-control-plane.sh`](demo/recording/svg/svg-infra-control-plane.sh)

### Every layer, an open standard

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-open-standards.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-open-standards.svg">
  <img src="demo/recording/svg/output/dark/infra-open-standards.svg" alt="Open standards — terminal recording" width="820">
</picture>

> MCP, OAuth 2.0, JWT, PostgreSQL, YAML — zero proprietary protocols at any layer. → [`svg-infra-open-standards.sh`](demo/recording/svg/svg-infra-open-standards.sh) · [Learn more](https://systemprompt.io/features/no-vendor-lock-in)

---

## Capabilities

Four enforcement layers. Full audit trail. Zero blind spots.

### Governance pipeline

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-governance.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-governance.svg">
  <img src="demo/recording/svg/output/dark/cap-governance.svg" alt="Governance pipeline — terminal recording" width="820">
</picture>

> Every tool call hits `PreToolUse → govern`. Scope, secret scan, blocklist, rate limit — all synchronous, all in-process, all audited. → [`svg-cap-governance.sh`](demo/recording/svg/svg-cap-governance.sh) · [Learn more](https://systemprompt.io/features/governance-pipeline)

### Secrets management

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-secrets.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-secrets.svg">
  <img src="demo/recording/svg/output/dark/cap-secrets.svg" alt="Secrets management — terminal recording" width="820">
</picture>

> Even admin-scope agents can't leak an AWS key, a GitHub PAT, or an RSA private key. Credentials are injected server-side at tool-call time. → [`svg-cap-secrets.sh`](demo/recording/svg/svg-cap-secrets.sh) · [Learn more](https://systemprompt.io/features/secrets-management)

### MCP governance

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-mcp.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-mcp.svg">
  <img src="demo/recording/svg/output/dark/cap-mcp.svg" alt="MCP governance — terminal recording" width="820">
</picture>

> Central registry, per-server OAuth2, scoped tool exposure, end-to-end access logs. → [`svg-cap-mcp.sh`](demo/recording/svg/svg-cap-mcp.sh) · [Learn more](https://systemprompt.io/features/mcp-governance)

### Analytics & observability

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-analytics.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-analytics.svg">
  <img src="demo/recording/svg/output/dark/cap-analytics.svg" alt="Analytics and observability — terminal recording" width="820">
</picture>

> Audit trail, execution traces, cost attribution, dashboard overview — every decision queryable, every token costed. → [`svg-cap-analytics.sh`](demo/recording/svg/svg-cap-analytics.sh) · [Learn more](https://systemprompt.io/features/analytics-and-observability)

### Closed-loop agents

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-agents.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-agents.svg">
  <img src="demo/recording/svg/output/dark/cap-agents.svg" alt="Closed-loop agents — terminal recording" width="820">
</picture>

> A2A discovery, AI reasoning with MCP tools, self-observation via analytics, full execution trace. → [`svg-cap-agents.sh`](demo/recording/svg/svg-cap-agents.sh) · [Learn more](https://systemprompt.io/features/closed-loop-agents)

### Compliance

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-compliance.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-compliance.svg">
  <img src="demo/recording/svg/output/dark/cap-compliance.svg" alt="Compliance — terminal recording" width="820">
</picture>

> Identity-bound audit table, structured log retention, config validation. Built for SOC 2, ISO 27001, HIPAA, and OWASP Agentic Top 10. → [`svg-cap-compliance.sh`](demo/recording/svg/svg-cap-compliance.sh) · [Learn more](https://systemprompt.io/features/compliance)

---

## Integrations

Provider-agnostic. Protocol-native. Fully extensible.

### Any AI agent

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-any-agent.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-any-agent.svg">
  <img src="demo/recording/svg/output/dark/int-any-agent.svg" alt="Any AI agent — terminal recording" width="820">
</picture>

> Anthropic, OpenAI, Gemini — any provider, any agent. One governance layer governs them all. → [`svg-int-any-agent.sh`](demo/recording/svg/svg-int-any-agent.sh) · [Learn more](https://systemprompt.io/features/any-ai-agent)

### Extensible architecture

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-extensions.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-extensions.svg">
  <img src="demo/recording/svg/output/dark/int-extensions.svg" alt="Extensions and capabilities — terminal recording" width="820">
</picture>

> 12 extensions, 71 sqlx-checked schemas, 13 jobs — all compiled into one binary, all discoverable via the CLI. → [`svg-int-extensions.sh`](demo/recording/svg/svg-int-extensions.sh) · [Learn more](https://systemprompt.io/features/extensible-architecture)

### Governance at scale

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-benchmark.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-benchmark.svg">
  <img src="demo/recording/svg/output/dark/int-benchmark.svg" alt="Governance benchmark — terminal recording" width="820">
</picture>

> Measure it yourself with `just benchmark`. Author's laptop: **3,308 req/s** on the burst run, p50 13.5 ms / p99 22.7 ms. Governance adds <1% to AI response time. → [`svg-int-benchmark.sh`](demo/recording/svg/svg-int-benchmark.sh)

---

## How it's built

One language. One database. One binary. One CLI.

- **Rust workspace** — `core/` is a read-only BSL-1.1 submodule; your code lives in `extensions/`.
- **PostgreSQL 18+** — the only runtime dependency. Air-gap deploys are a `docker run` away.
- **Thousands of req/s** — the pipeline is synchronous and in-process, not a sidecar. 3.3k req/s on a laptop, p50 ~13 ms.
- **Provider-agnostic** — Anthropic, OpenAI, Gemini — swap at the profile level.
- **MCP · A2A · OAuth2 · WebAuthn** — governed at the protocol layer, not bolted on.

```
my-eval/
├── extensions/       # Your Rust code (compile-time extensions)
├── services/         # Config-only (YAML/Markdown): agents, skills, plugins, providers
├── demo/             # 40+ runnable evaluation scripts + recording pipeline
├── storage/files/    # Static assets (CSS, JS, images)
├── Cargo.toml        # Workspace manifest
├── justfile          # Development commands
└── CLAUDE.md         # AI assistant instructions
```

---

## Demo index

40+ executable evaluation scripts across ten categories. Each script is numbered — run them in order.

| Pillar | Category | Scripts | Exercises | Recording |
|---|---|---|---|---|
| Infrastructure | [`demo/infrastructure/`](demo/infrastructure/) | 5 | Services, database, jobs, logs, config | `infra-self-hosted`, `infra-control-plane` |
| Infrastructure | [`demo/cloud/`](demo/cloud/) | 1 | Auth, profiles, deployment info | `infra-deploy-anywhere` |
| Capabilities | [`demo/governance/`](demo/governance/) | 8 | Tool-call approvals, denials, secret breach, rate limits, hooks | `cap-governance`, `cap-secrets` |
| Capabilities | [`demo/mcp/`](demo/mcp/) | 3 | MCP servers, access tracking, tool execution | `cap-mcp` |
| Capabilities | [`demo/analytics/`](demo/analytics/) | 8 | Overview, agents, costs, requests, sessions, content/traffic, conversations, tools | `cap-analytics` |
| Capabilities | [`demo/agents/`](demo/agents/) | 5 | Agent lifecycle, config, messaging, tracing, A2A registry | `cap-agents` |
| Capabilities | [`demo/users/`](demo/users/) | 4 | User CRUD, roles, sessions, IP bans | `cap-compliance` |
| Integrations | [`demo/skills/`](demo/skills/) | 5 | Skills, content, files, plugins, contexts | `int-extensions` |
| Integrations | [`demo/performance/`](demo/performance/) | 2 | Request tracing, benchmarks, load testing | `int-benchmark` |
| Integrations | [`demo/web/`](demo/web/) | 2 | Content types, templates, sitemaps, validation | – |

See [`demo/README.md`](demo/README.md) for the full catalogue, [`demo/AGENTS.md`](demo/AGENTS.md) for the LLM-targeted runbook, and [`demo/recording/svg/README.md`](demo/recording/svg/README.md) for how to regenerate the terminal SVGs above.

---

## Reference

| `just` target | Description |
|---|---|
| `just build` | Build the workspace |
| `just setup-local [keys] [http_port] [pg_port]` | Seed local profile, start Docker Postgres, run publish pipeline |
| `just start` | Start all services |
| `just publish` | Compile templates, bundle CSS/JS, copy assets |
| `just record-svgs [N…]` | Regenerate the terminal SVGs above |
| `just db-up` / `db-down` / `db-reset` | Manage the local Postgres container |
| `just clippy` | Lint the workspace (pedantic, deny-all) |

`systemprompt --help` is the entry point for the CLI. Every domain — `core`, `infra`, `admin`, `cloud`, `analytics`, `web`, `plugins`, `build` — is discoverable with `systemprompt <domain> --help`.

---

## License & production use

**This template** is MIT. Fork it, modify it, use it however you like — **for local evaluation**.

**[systemprompt-core](https://github.com/systempromptio/systemprompt-core)** is [BSL-1.1](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE): free for evaluation, testing, and non-production use. **Production use requires a commercial license.** Each version converts to Apache 2.0 four years after publication.

**Evaluate Free** → you're already here. **Book a Meeting** → [**systemprompt.io**](https://systemprompt.io) · [ed@systemprompt.io](mailto:ed@systemprompt.io)

---

<div align="center">

**[systemprompt.io](https://systemprompt.io)** · **[systemprompt-core](https://github.com/systempromptio/systemprompt-core)** · **[Documentation](https://systemprompt.io/documentation/)** · **[Guides](https://systemprompt.io/guides)** · **[Discord](https://discord.gg/wkAbSuPWpr)**

<sub>Own how your organization uses AI. Every interaction governed and provable.</sub>

</div>
