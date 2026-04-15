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

## Govern. Every tool call.

Four layers of synchronous in-process enforcement on every tool call: **scope** check → **secret** scan → **block**list → **rate** limit. No sidecar. No proxy. Deny reasons are structured and auditable.

### Four-layer governance pipeline

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/01-governance.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/01-governance.svg">
  <img src="demo/recording/svg/output/dark/01-governance.svg" alt="Governance pipeline — terminal recording" width="820">
</picture>

> Every tool call hits `PreToolUse → govern`. Scope, secret scan, blocklist, rate limit — all synchronous, all in-process, all audited. → [`demo/recording/svg/svg-01-governance.sh`](demo/recording/svg/svg-01-governance.sh)

### Secrets never touch inference

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/02-secrets.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/02-secrets.svg">
  <img src="demo/recording/svg/output/dark/02-secrets.svg" alt="Secrets management — terminal recording" width="820">
</picture>

> Even admin-scope agents can't leak an AWS key, a GitHub PAT, or an RSA private key. Credentials are injected server-side at tool-call time. → [`svg-02-secrets.sh`](demo/recording/svg/svg-02-secrets.sh)

### Thousands of governed requests per second

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/06-benchmark.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/06-benchmark.svg">
  <img src="demo/recording/svg/output/dark/06-benchmark.svg" alt="Governance benchmark — terminal recording" width="820">
</picture>

> Measure it yourself with `just benchmark`. Author's laptop: **3,308 req/s** on the burst run, p50 13.5 ms / p99 22.7 ms — every request running JWT validation, scope resolution, three rule evaluations, and an async audit write. Governance adds <1% to AI response time. → [`svg-06-benchmark.sh`](demo/recording/svg/svg-06-benchmark.sh)

---

## Prove. Every decision.

Structured evidence for every interaction. Every tool call produces a five-point audit trace — the shape auditors ask for, not the shape your logging framework happens to emit.

### Queryable audit trail

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/03-audit.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/03-audit.svg">
  <img src="demo/recording/svg/output/dark/03-audit.svg" alt="Audit trail — terminal recording" width="820">
</picture>

> Identity → Agent Context → Permissions → Tool Execution → Result. Queryable, SIEM-ready, cost-attributed. → [`svg-03-audit.sh`](demo/recording/svg/svg-03-audit.sh)

### Replay any request, end to end

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/05-tracing.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/05-tracing.svg">
  <img src="demo/recording/svg/output/dark/05-tracing.svg" alt="Request tracing — terminal recording" width="820">
</picture>

> Reconstruct any request as the model, the permission engine, and the operator saw it. → [`svg-05-tracing.sh`](demo/recording/svg/svg-05-tracing.sh)

---

## Standardise. Every team.

One CLI, one binary, one database — the same surface local and remote. Register agents, distribute skills by role, run them under identical policy everywhere.

### Multi-agent orchestration

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/07-agent.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/07-agent.svg">
  <img src="demo/recording/svg/output/dark/07-agent.svg" alt="Agent orchestration — terminal recording" width="820">
</picture>

> Discover, configure, message, trace, and register agents. MCP and A2A at the protocol layer. → [`svg-07-agent.sh`](demo/recording/svg/svg-07-agent.sh)

### MCP servers under central governance

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/04-mcp-tracking.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/04-mcp-tracking.svg">
  <img src="demo/recording/svg/output/dark/04-mcp-tracking.svg" alt="MCP tool tracking — terminal recording" width="820">
</picture>

> Central registry, per-server OAuth2, scoped tool exposure, end-to-end access logs. → [`svg-04-mcp-tracking.sh`](demo/recording/svg/svg-04-mcp-tracking.sh)

### One binary, every capability compiled in

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/08-skills.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/08-skills.svg">
  <img src="demo/recording/svg/output/dark/08-skills.svg" alt="Extensions and capabilities — terminal recording" width="820">
</picture>

> 12 extensions, 71 sqlx-checked schemas, 13 jobs — all in one process, all discoverable via the CLI. → [`svg-08-skills.sh`](demo/recording/svg/svg-08-skills.sh)

### Services, database, and logs in one CLI

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/09-infrastructure.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/09-infrastructure.svg">
  <img src="demo/recording/svg/output/dark/09-infrastructure.svg" alt="Infrastructure overview — terminal recording" width="820">
</picture>

> `infra services status` → `infra db status` → `infra logs view`. No sidecars, no sprawl. → [`svg-09-infrastructure.sh`](demo/recording/svg/svg-09-infrastructure.sh)

### Identity, roles, and network-layer bans

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/10-users.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/10-users.svg">
  <img src="demo/recording/svg/output/dark/10-users.svg" alt="Users and identity — terminal recording" width="820">
</picture>

> Users are first-class; roles scope every tool call; IP bans are audited. Governance follows the user, not the client. → [`svg-10-users.sh`](demo/recording/svg/svg-10-users.sh)

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

| Category | Scripts | Exercises | Recording |
|---|---|---|---|
| [`demo/governance/`](demo/governance/) | 8 | Tool-call approvals, denials, secret breach, rate limits, hooks | ✓ `01`, `02` |
| [`demo/agents/`](demo/agents/) | 5 | Agent lifecycle, config, messaging, tracing, A2A registry | ✓ `07` |
| [`demo/mcp/`](demo/mcp/) | 3 | MCP servers, access tracking, tool execution | ✓ `04` |
| [`demo/skills/`](demo/skills/) | 5 | Skills, content, files, plugins, contexts | ✓ `08` |
| [`demo/infrastructure/`](demo/infrastructure/) | 5 | Services, database, jobs, logs, config | ✓ `09` |
| [`demo/analytics/`](demo/analytics/) | 8 | Overview, agents, costs, requests, sessions, content/traffic, conversations, tools | ✓ `03` |
| [`demo/users/`](demo/users/) | 4 | User CRUD, roles, sessions, IP bans | ✓ `10` |
| [`demo/performance/`](demo/performance/) | 2 | Request tracing, benchmarks, load testing | ✓ `05`, `06` |
| [`demo/web/`](demo/web/) | 2 | Content types, templates, sitemaps, validation | – |
| [`demo/cloud/`](demo/cloud/) | 1 | Auth, profiles, deployment info | – |

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
