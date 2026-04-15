<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://systemprompt.io/files/images/logo.svg">
  <source media="(prefers-color-scheme: light)" srcset="https://systemprompt.io/files/images/logo-dark.svg">
  <img src="https://systemprompt.io/files/images/logo-dark.svg" alt="systemprompt.io" width="380">
</picture>

# Own how your organization uses AI.

### systemprompt.io is the narrow waist between your AI and everything it touches.
### Self-hosted. Air-gapped. Owned. Every interaction governed and provable.

[![Built on systemprompt-core](https://img.shields.io/badge/built%20on-systemprompt--core-2b6cb0?style=flat-square)](https://github.com/systempromptio/systemprompt-core)
[![Template · MIT](https://img.shields.io/badge/template-MIT-16a34a?style=flat-square)](LICENSE)
[![Core · BSL--1.1](https://img.shields.io/badge/core-BSL--1.1-2b6cb0?style=flat-square)](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75+-f97316?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![PostgreSQL 18+](https://img.shields.io/badge/postgres-18+-336791?style=flat-square&logo=postgresql&logoColor=white)](https://www.postgresql.org/)
[![Provider-agnostic](https://img.shields.io/badge/provider-agnostic-111827?style=flat-square)](https://systemprompt.io)

[**systemprompt.io**](https://systemprompt.io) · [**Documentation**](https://systemprompt.io/documentation/) · [**Guides**](https://systemprompt.io/guides) · [**Discord**](https://discord.gg/wkAbSuPWpr)

</div>

---

## This is the local evaluation

**systemprompt.io is AI Governance Infrastructure** — the governance layer for AI agents. A single compiled Rust binary that authenticates, authorises, rate-limits, logs, and costs every AI interaction. Self-hosted, air-gap capable, provider-agnostic.

This repo is how you evaluate it. Clone it, run it on your own machine, bring your own AI key, and watch every claim above execute against 40+ scripted demos. The terminal recordings below are real captures of those demos running — not mockups.

> The template is MIT. The underlying [systemprompt-core](https://github.com/systempromptio/systemprompt-core) library is BSL-1.1 — free for evaluation and non-production use. Production requires a commercial license.

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

## Govern. Every tool call.

Four layers of synchronous in-process enforcement on every tool call: **scope** check → **secret** scan → **block**list → **rate** limit. No sidecar. No proxy. Deny reasons are structured and auditable.

### Four-layer governance pipeline

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/light/01-governance.svg">
  <img src="demo/recording/svg/output/dark/01-governance.svg" alt="Governance pipeline — terminal recording" width="820">
</picture>

> `./demo/governance/01-happy-path.sh` → `08-rate-limit.sh` — approvals, scope denials, blocklists, hooks.

### MCP servers under central governance

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/light/04-mcp-tracking.svg">
  <img src="demo/recording/svg/output/dark/04-mcp-tracking.svg" alt="MCP tool tracking — terminal recording" width="820">
</picture>

> `./demo/mcp/*.sh` — central registry, per-server OAuth2, scoped tool exposure, end-to-end access logs.

### Sub-5 ms governance overhead, benchmarked

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/light/06-benchmark.svg">
  <img src="demo/recording/svg/output/dark/06-benchmark.svg" alt="Governance benchmark — terminal recording" width="820">
</picture>

> `./demo/performance/02-benchmark.sh` — measure the policy overhead yourself. Author's laptop: p50 < 5 ms, p99 < 12 ms.

---

## Prove. Every decision.

Structured evidence for every interaction. Secrets never touch inference. Every tool call produces a five-point audit trace — the shape auditors ask for, not the shape your logging framework happens to emit.

### Secrets never touch inference

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/light/02-secrets.svg">
  <img src="demo/recording/svg/output/dark/02-secrets.svg" alt="Secrets management — terminal recording" width="820">
</picture>

> `./demo/governance/06-secret-breach.sh` — fire a call that would exfiltrate an AWS key; the pipeline drops it before it leaves the process. Credentials are injected server-side at tool-call time.

### Five-point audit trace

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/light/03-audit.svg">
  <img src="demo/recording/svg/output/dark/03-audit.svg" alt="Audit trail — terminal recording" width="820">
</picture>

> `./demo/analytics/*.sh` — Identity → Agent Context → Permissions → Tool Execution → Result. Queryable, SIEM-ready.

### Replay any conversation

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/light/05-tracing.svg">
  <img src="demo/recording/svg/output/dark/05-tracing.svg" alt="Request tracing — terminal recording" width="820">
</picture>

> `./demo/performance/01-tracing.sh` — reconstruct any request as the model, the permission engine, and the operator saw it.

---

## Standardise. Every team.

One CLI, one binary, one database — the same surface local and remote. Register agents, distribute skills by role and department, run them under identical policy everywhere.

### Multi-agent orchestration

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/light/07-agent.svg">
  <img src="demo/recording/svg/output/dark/07-agent.svg" alt="Agent orchestration — terminal recording" width="820">
</picture>

> `./demo/agents/*.sh` — discover, configure, message, trace, and register agents. MCP and A2A at the protocol layer.

### Admin dashboard

<details>
<summary><b>See the dashboard (2 screenshots)</b></summary>

<br>

<img src="docs/images/admin-dashboard-overview.png" alt="Admin dashboard overview" width="900">

<br><br>

<img src="docs/images/admin-dashboard-governance.png" alt="Governance panel" width="900">

</details>

---

## How it's built

One language. One database. One binary. One CLI.

- **Rust workspace** — `core/` is a read-only BSL-1.1 submodule; your code lives in `extensions/`.
- **PostgreSQL 18+** — the only runtime dependency. Air-gap deploys are a `docker run` away.
- **Sub-5 ms governance** — the pipeline is synchronous and in-process, not a sidecar.
- **Provider-agnostic** — Anthropic, OpenAI, Gemini — swap at the profile level.
- **MCP · A2A · OAuth2 · WebAuthn** — governed at the protocol layer, not bolted on.

```
my-eval/
├── extensions/       # Your Rust code (compile-time extensions)
│   ├── web/          # Web publishing, themes, SSR
│   └── mcp/          # MCP server implementations
├── services/         # Config-only (YAML/Markdown): agents, skills, plugins, AI providers
├── demo/             # 40+ runnable evaluation scripts, 10 categories
├── storage/files/    # Static assets (CSS, JS, images)
├── docs/images/      # README screenshots
├── Cargo.toml        # Workspace manifest
├── justfile          # Development commands
└── CLAUDE.md         # AI assistant instructions
```

---

## Commands

| `just` target | Description |
|---|---|
| `just build` | Build the workspace |
| `just setup-local [keys] [http_port] [pg_port]` | Seed local profile, start Docker Postgres, run publish pipeline |
| `just start` | Start all services |
| `just publish` | Compile templates, bundle CSS/JS, copy assets |
| `just db-up` / `db-down` / `db-reset` | Manage the local Postgres container |
| `just clippy` | Lint the workspace (pedantic, deny-all) |

### `systemprompt` CLI cheatsheet

| Domain | Purpose |
|---|---|
| `core` | Skills, content, files, contexts, plugins, hooks, artifacts |
| `infra` | Services, database, jobs, logs |
| `admin` | Users, agents, config, setup, session |
| `cloud` | Auth, deploy, sync, secrets, tenant, domain |
| `analytics` | Overview, conversations, agents, tools, requests, sessions, content, traffic, costs |
| `web` | Content-types, templates, assets, sitemap, validate |
| `plugins` | Extensions, MCP servers, capabilities |
| `build` | Build core workspace and MCP extensions |

Everything is discoverable — `systemprompt <domain> --help` everywhere.

---

## Demo index

The [`demo/`](demo/) directory is 40+ executable evaluation scripts across ten categories. Each script is numbered — run them in order.

| Category | Scripts | Exercises |
|---|---|---|
| [`demo/governance/`](demo/governance/) | 8 | Tool-call approvals, denials, secret breach, rate limits, hooks |
| [`demo/agents/`](demo/agents/) | 5 | Agent lifecycle, config, messaging, tracing, A2A registry |
| [`demo/mcp/`](demo/mcp/) | 3 | MCP servers, access tracking, tool execution |
| [`demo/skills/`](demo/skills/) | 5 | Skills, content, files, plugins, contexts |
| [`demo/infrastructure/`](demo/infrastructure/) | 5 | Services, database, jobs, logs, config |
| [`demo/analytics/`](demo/analytics/) | 8 | Overview, agents, costs, requests, sessions, content/traffic, conversations, tools |
| [`demo/users/`](demo/users/) | 4 | User CRUD, roles, sessions, IP bans |
| [`demo/web/`](demo/web/) | 2 | Content types, templates, sitemaps, validation |
| [`demo/cloud/`](demo/cloud/) | 1 | Auth, profiles, deployment info |
| [`demo/performance/`](demo/performance/) | 2 | Request tracing, benchmarks, load testing |

See [`demo/README.md`](demo/README.md) for the full catalogue and [`demo/AGENTS.md`](demo/AGENTS.md) for the LLM-targeted runbook. The terminal SVGs above are generated from these scripts — regenerate with [`demo/recording/`](demo/recording/).

---

## License & production use

**This template** is MIT licensed. Fork it, modify it, use it however you like — **for local evaluation**.

**[systemprompt-core](https://github.com/systempromptio/systemprompt-core)** is [BSL-1.1](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE): free for evaluation, testing, and non-production use. **Production use requires a commercial license.** Each version converts to Apache 2.0 four years after publication.

**Evaluate Free** → you're already here. **Book a Meeting** → [**systemprompt.io**](https://systemprompt.io) · [ed@systemprompt.io](mailto:ed@systemprompt.io)

---

<div align="center">

**[systemprompt.io](https://systemprompt.io)** · **[systemprompt-core](https://github.com/systempromptio/systemprompt-core)** · **[Documentation](https://systemprompt.io/documentation/)** · **[Guides](https://systemprompt.io/guides)** · **[Discord](https://discord.gg/wkAbSuPWpr)**

<sub>Own how your organization uses AI. Every interaction governed and provable.</sub>

</div>
