<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://systemprompt.io/files/images/logo.svg">
  <source media="(prefers-color-scheme: light)" srcset="https://systemprompt.io/files/images/logo-dark.svg">
  <img src="https://systemprompt.io/files/images/logo-dark.svg" alt="systemprompt.io" width="380">
</picture>

# Own how your organization uses AI.

### Self-hosted governance for every AI tool call. One Rust binary. One Postgres. Every decision audited before execution.

[![Built on systemprompt-core](https://img.shields.io/badge/built%20on-systemprompt--core-2b6cb0?style=flat-square)](https://github.com/systempromptio/systemprompt-core)
[![Template · MIT](https://img.shields.io/badge/template-MIT-16a34a?style=flat-square)](LICENSE)
[![Core · BSL--1.1](https://img.shields.io/badge/core-BSL--1.1-2b6cb0?style=flat-square)](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75+-f97316?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![PostgreSQL 18+](https://img.shields.io/badge/postgres-18+-336791?style=flat-square&logo=postgresql&logoColor=white)](https://www.postgresql.org/)

[**systemprompt.io**](https://systemprompt.io) · [**Documentation**](https://systemprompt.io/documentation/) · [**Guides**](https://systemprompt.io/guides) · [**Discord**](https://discord.gg/wkAbSuPWpr)

</div>

---

## What this is

A single Rust binary that sits between your AI agents and every tool, database, or API they touch. It authenticates the caller, checks scope, scans for secrets, blocklists destructive operations, and rate-limits — synchronously, in-process, before the call executes. Every allow/deny lands in an 18-column Postgres audit table. No sidecars. No Kubernetes. No Redis. No Kafka. One binary, one database, one PID.

This repo is the evaluation template. Fork it, clone it, compile it, and every claim below runs against 43 scripted demos on your own laptop.

```
  LLM Agent
      │
      ▼
  Governance pipeline  (in-process, synchronous, <5 ms p99)
      │
      ├─ 1. JWT validation       (HS256, verified locally)
      ├─ 2. RBAC scope check     (Admin · User · Service · A2A · MCP · Anonymous)
      ├─ 3. Secret detection     (35+ regex: API keys, PATs, PEM, AWS prefixes)
      ├─ 4. Blocklist            (destructive operation categories)
      └─ 5. Rate limiting        (300 req/min per session, role multipliers)
      │
      ▼
  ALLOW or DENY   →  18-column audit row, always
      │
      ▼ (ALLOW)
  spawn_server()
      │
      ├─ decrypt secrets from ChaCha20-Poly1305 store
      └─ inject into subprocess env vars only (never parent)
      │
      ▼
  MCP tool process     credentials live here, never in the LLM context path
```

---

## Quick start

Three commands from fresh clone to a running governance binary. A fourth runs every demo.

```bash
just build                                               # 1. compile the workspace
just setup-local <anthropic> <openai> <gemini>           # 2. profile + Postgres + publish
just start                                               # 3. serve governance, agents, MCP, admin, API
./demo/sweep.sh                                          # 4. run all 43 demos against the live binary
```

### What you'll see in the first five minutes

- **http://localhost:8080** — admin UI, live audit table, session viewer.
- **`systemprompt analytics overview`** — conversations, tool calls, costs in microdollars, anomalies flagged above 2x/3x of rolling average.
- **`systemprompt infra logs audit <request-id> --full`** — the full trace for any request: identity, scope, rule evaluations, tool call, model output, cost. One query, one row, one answer.
- **Point Claude Code, Claude Desktop, or any MCP client at it.** Permissions follow the user, not the client. Try to exfiltrate a key through a tool argument and watch the secret-detection layer deny it before the tool process spawns.
- **`./demo/governance/06-secret-breach.sh`** — a scripted version of that denial, running against the same binary you just started.

If any of those five land, you have enough to decide whether this belongs in your stack.

### Run the demos

Each recording below has a script that produced it. Every script runs against the live binary. No mocks.

```bash
./demo/00-preflight.sh                    # acquire token, verify services, create admin
./demo/01-seed-data.sh                    # populate analytics + trace data

# Capabilities — governance, secrets, audit
./demo/governance/01-happy-path.sh        # allowed tool call, trace chain
./demo/governance/05-governance-denied.sh # scope check rejects out-of-role call
./demo/governance/06-secret-breach.sh     # secret-detection blocks exfiltration
./demo/governance/07-rate-limiting.sh     # 300 req/min per session enforced
./demo/governance/08-hooks.sh             # PreToolUse policy-as-code

# Infrastructure — one binary, CLI control plane
./demo/infrastructure/02-database.sh      # 144 tables, schema introspection
./demo/infrastructure/04-logs.sh          # structured JSON events
./demo/analytics/01-overview.sh           # conversations, costs, anomalies

# Integrations — MCP, agents, A2A
./demo/mcp/01-servers.sh                  # per-server OAuth2, manifest validation
./demo/agents/02-messaging.sh             # governed A2A handoff (~$0.01)
./demo/performance/02-load-test.sh        # 3,308 req/s burst, p99 22.7 ms
```

Full index: [`demo/README.md`](demo/README.md). 41 of 43 scripts are free; two cost ~$0.01 each (real model calls).

### Prerequisites

| Requirement | Purpose | Install |
|---|---|---|
| **Docker** | PostgreSQL runs in a container; `just setup-local` starts it | [docker.com](https://docs.docker.com/get-docker/) |
| **Rust 1.75+** | Compiles the workspace binary | [rustup.rs](https://rustup.rs/) |
| **`just`** | Task runner | [just.systems](https://just.systems/) |
| **`jq`, `yq`** | JSON and YAML processing in the scripts | `brew install jq yq` / `apt install jq yq` |
| **AI API keys** | One key per provider enabled in `services/ai/config.yaml`. Shipped config enables Anthropic, OpenAI, Gemini (default `gemini`). Disable providers you don't want or pass all three. | Provider dashboards |
| **Ports 8080 + 5432** | HTTP + PostgreSQL | Free on localhost |

Running a second clone side-by-side: `just setup-local <anthropic> <openai> <gemini> 8081 5433`.

---

## How credential injection works

When a tool call passes the pipeline, `spawn_server()` decrypts credentials from the ChaCha20-Poly1305 store and injects them into the child process environment. The parent process — which owns the LLM context window — never writes the value.

Source: [`systemprompt-core/crates/domain/mcp/src/services/process/spawner.rs`](https://github.com/systempromptio/systemprompt-core/blob/main/crates/domain/mcp/src/services/process/spawner.rs).

```rust
let secrets = SecretsBootstrap::get()?;

let mut child_command = Command::new(&binary_path);

// Child env only. The parent process (LLM context path) never touches the value.
if let Some(key) = &secrets.anthropic {
    child_command.env("ANTHROPIC_API_KEY", key);
}
if let Some(key) = &secrets.github {
    child_command.env("GITHUB_TOKEN", key);
}

// Detach; parent forgets the child after spawn.
let child = child_command.spawn()?;
std::mem::forget(child);
```

Before the spawn, a secret-detection pipeline scans the tool arguments for 35+ credential patterns. A tool call that tries to pass a secret through the context window is blocked even if the agent has scope to run the tool. Run it:

```bash
./demo/governance/06-secret-breach.sh
```

---

## Infrastructure

One binary, one database, deploys anywhere. 144 schema-checked Postgres tables. 12 extensions compiled in at link time via `register_extension!`. Eight CLI domains cover every operational surface.

| Domain | Purpose |
|---|---|
| `core` | Skills, content, files, contexts, plugins, hooks, artifacts |
| `infra` | Services, database, jobs, logs |
| `admin` | Users, agents, config, setup, session, rate limits |
| `cloud` | Auth, deploy, sync, secrets, tenant, domain |
| `analytics` | Overview, conversations, agents, tools, requests, sessions, content, traffic, costs |
| `web` | Content types, templates, assets, sitemap, validate |
| `plugins` | Extensions, MCP servers, capabilities |
| `build` | Build core workspace and MCP extensions |

Every layer is an open standard: **MCP** for tools, **OAuth 2.0** and **WebAuthn** for identity, **ChaCha20-Poly1305** for encryption at rest, **PostgreSQL** for storage, **Git** for distribution. You can leave without a migration.

### Self-hosted deployment

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-self-hosted.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-self-hosted.svg">
  <img src="demo/recording/svg/output/dark/infra-self-hosted.svg" alt="Self-hosted deployment — terminal recording" width="820">
</picture>

One binary, one process, one database. All data stays on your infrastructure. Zero outbound telemetry by default. Run: `./demo/infrastructure/01-services.sh` · [Learn more](https://systemprompt.io/features/self-hosted-ai-platform)

### Deploy anywhere

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-deploy-anywhere.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-deploy-anywhere.svg">
  <img src="demo/recording/svg/output/dark/infra-deploy-anywhere.svg" alt="Deploy anywhere — terminal recording" width="820">
</picture>

Same binary runs laptop, VM, Kubernetes pod, and air-gapped box. Profile YAML promotes environments without rebuilding. Run: `./demo/cloud/01-cloud-auth.sh` · [Learn more](https://systemprompt.io/features/deploy-anywhere)

### Unified control plane

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-control-plane.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-control-plane.svg">
  <img src="demo/recording/svg/output/dark/infra-control-plane.svg" alt="Unified control plane — terminal recording" width="820">
</picture>

Every operational surface has a CLI verb. No dashboard is required for any task. `systemprompt <domain> --help` works everywhere. Run: `./demo/infrastructure/03-jobs.sh` · [Learn more](https://systemprompt.io/features/unified-control-plane)

### Open standards

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-open-standards.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-open-standards.svg">
  <img src="demo/recording/svg/output/dark/infra-open-standards.svg" alt="Open standards — terminal recording" width="820">
</picture>

MCP for tools, OAuth 2.0 for identity, PostgreSQL for storage, Git for distribution. Zero proprietary protocols at any layer. Run: `./demo/mcp/01-servers.sh` · [Learn more](https://systemprompt.io/features/no-vendor-lock-in)

---

## Capabilities

Four layers of enforcement per request. Six-tier RBAC (Admin 10x · User 1x · Service 5x · A2A 5x · MCP 5x · Anonymous 0.5x). 35+ regex secret patterns. 300 req/min base rate per session with role multipliers. Sub-5 ms p99 overhead. Every decision lands in an 18-column audit row with six correlation columns (UserId, SessionId, TaskId, TraceId, ContextId, ClientId). Nothing sampled. Cost tracked in integer microdollars.

### Governance pipeline

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-governance.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-governance.svg">
  <img src="demo/recording/svg/output/dark/cap-governance.svg" alt="Governance pipeline — terminal recording" width="820">
</picture>

Scope check, secret scan, blocklist, rate limit. Synchronous and in-process on every tool call. Run: `./demo/governance/05-governance-denied.sh` · [Learn more](https://systemprompt.io/features/governance-pipeline)

### Secrets management

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-secrets.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-secrets.svg">
  <img src="demo/recording/svg/output/dark/cap-secrets.svg" alt="Secrets management — terminal recording" width="820">
</picture>

Credentials encrypted at rest with ChaCha20-Poly1305 and injected into the tool subprocess at spawn time. They never enter the context window, never appear in logs, never transit the inference path. An admin-scope agent cannot exfiltrate an AWS key, a GitHub PAT, or a PEM block. Run: `./demo/governance/06-secret-breach.sh` · [Learn more](https://systemprompt.io/features/secrets-management)

### MCP governance

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-mcp.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-mcp.svg">
  <img src="demo/recording/svg/output/dark/cap-mcp.svg" alt="MCP governance — terminal recording" width="820">
</picture>

Each MCP server runs as an independent OAuth2 resource server with isolated credentials and per-server scope validation. If a tool is not in the plugin manifest, it does not exist for the agent. Four-pass deploy-time validation catches port conflicts, missing configs, empty OAuth scopes, and malformed server types before anything starts. Run: `./demo/mcp/02-access-tracking.sh` · [Learn more](https://systemprompt.io/features/mcp-governance)

### Analytics and observability

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-analytics.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-analytics.svg">
  <img src="demo/recording/svg/output/dark/cap-analytics.svg" alt="Analytics and observability — terminal recording" width="820">
</picture>

Nine analytics subcommands: overview, conversations, agents, tools, requests, sessions, content, traffic, costs. Anomaly detection flags values past 2x (warn) or 3x (critical) of rolling average. JSON event streaming feeds Splunk, ELK, Datadog, Sumo. Run: `./demo/analytics/01-overview.sh` · [Learn more](https://systemprompt.io/features/analytics-and-observability)

### Closed-loop agents

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-agents.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-agents.svg">
  <img src="demo/recording/svg/output/dark/cap-agents.svg" alt="Closed-loop agents — terminal recording" width="820">
</picture>

Agents query their own error rate, cost, and latency through exposed MCP tools and adjust without a human in the loop. A2A protocol carries governance on every hop of a multi-provider workflow. Run: `./demo/agents/03-tracing.sh` · [Learn more](https://systemprompt.io/features/closed-loop-agents)

### Compliance

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-compliance.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-compliance.svg">
  <img src="demo/recording/svg/output/dark/cap-compliance.svg" alt="Compliance — terminal recording" width="820">
</picture>

JWT-bound audit trails, 10 lifecycle event variants, tiered retention (debug 1 day through error 90 days). Policy-as-code on PreToolUse hooks. All data on-premises by default. Built for **SOC 2 Type II**, **ISO 27001**, **HIPAA**, and **OWASP Agentic Top 10**. Run: `./demo/users/03-session-management.sh` · [Learn more](https://systemprompt.io/features/compliance)

---

## Integrations

The `AiProvider` trait abstracts 19 methods, so swapping Anthropic, OpenAI, and Gemini is a config change, not a rewrite. The `Extension` trait contributes routes, schemas, migrations, jobs, LLM providers, tool providers, page prerenderers, roles, and config namespaces. 71 sqlx-checked schemas and 13 background jobs ship by default. Same RBAC, same audit, same pipeline on every provider.

### Any AI agent

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-any-agent.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-any-agent.svg">
  <img src="demo/recording/svg/output/dark/int-any-agent.svg" alt="Any AI agent — terminal recording" width="820">
</picture>

One governance layer for every provider. Swap Anthropic, OpenAI, or Gemini at the profile level. Cost attribution tracks spend per provider, model, and agent in integer microdollars. Run: `./demo/agents/01-agents.sh` · [Learn more](https://systemprompt.io/features/any-ai-agent)

### Claude Desktop & Cowork

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-cowork.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-cowork.svg">
  <img src="demo/recording/svg/output/dark/int-cowork.svg" alt="Claude Desktop & Cowork — terminal recording" width="820">
</picture>

Skills persist across sessions via OAuth2. Slash commands activate business skills governed by the four-layer pipeline. Install once, sync forever. Run: `./demo/skills/01-skills.sh` · [Learn more](https://systemprompt.io/features/cowork)

### Web server & publisher

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-web-publisher.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-web-publisher.svg">
  <img src="demo/recording/svg/output/dark/int-web-publisher.svg" alt="Web Server & Publisher — terminal recording" width="820">
</picture>

The same binary that governs AI agents serves your website, blog, and documentation. Markdown content, PostgreSQL full-text search, engagement analytics. systemprompt.io itself runs on this binary. Run: `./demo/web/01-web-config.sh` · [Learn more](https://systemprompt.io/features/web-publisher)

### Extensible architecture

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-extensions.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-extensions.svg">
  <img src="demo/recording/svg/output/dark/int-extensions.svg" alt="Extensions and capabilities — terminal recording" width="820">
</picture>

Your code compiles into your binary via the `Extension` trait. Routes, tables, jobs, and providers register at link time. No runtime reflection, no dynamic loading. Run: `./demo/skills/05-plugins.sh` · [Learn more](https://systemprompt.io/features/extensible-architecture)

---

## Performance

Sub-5 ms governance overhead, benchmarked. Each request performs JWT validation, scope resolution, three rule evaluations, and an async audit write.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-benchmark.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-benchmark.svg">
  <img src="demo/recording/svg/output/dark/int-benchmark.svg" alt="Governance benchmark — terminal recording" width="820">
</picture>

| Metric | Result |
|---|---|
| Throughput | 3,308 req/s burst, sustained under 100 concurrent workers |
| p50 latency | 13.5 ms |
| p99 latency | 22.7 ms |
| Added to AI response time | <1% |
| GC pauses | Zero |

Reproduce: `just benchmark`. Numbers measured on the author's laptop.

---

## Configuration

Runtime configuration is flat YAML under `services/`, loaded through `services/config/config.yaml`. Unknown keys fail loudly (`#[serde(deny_unknown_fields)]`).

```
services/
  config/config.yaml        Root aggregator
  agents/<id>.yaml          Agent: scope, model, tool access
  mcp/<name>.yaml           MCP server: OAuth2 config, scopes
  skills/<id>.yaml          Skill: config + markdown instruction body
  plugins/<name>.yaml       Plugin bindings (references agents, skills, MCP)
  ai/config.yaml            AI provider config (Anthropic, OpenAI, Gemini)
  scheduler/config.yaml     Background job schedule
  web/config.yaml           Web frontend, navigation, theme
  content/config.yaml       Content sources and indexing
```

No database-stored config, no admin UI required. Every change is a diff.

---

## Compared to alternatives

Architectural differences, not marketing comparisons.

**Shell environment variables (`~/.claude/settings.json`, per-developer key config).** Each developer configures themselves. No team-level enforcement, no audit trail, no per-agent scope, no anomaly detection. A new hire with a misconfigured environment goes uncaught.

**Proxy-layer gateways (Microsoft Azure AI Gateway, Kong AI Gateway, nginx variants).** Intercept HTTP between the client and the model. Can observe and block, but do not own the execution context of tools. Credential injection through a proxy requires the credential to transit the proxy as a value; the proxy process can log it, and a misconfigured proxy can leak it. Azure-native gateways need Azure infrastructure and are not air-gap capable.

**Vault + Kubernetes Secrets.** Stores credentials well, but enforcement is not at the point of LLM tool use. When an MCP tool needs a secret it makes a network call to retrieve it, and an LLM agent can observe, manipulate (via prompt injection), or mis-time that call. Vault does not know what the LLM is doing or why.

**This binary.** The governance process is the parent of every MCP tool subprocess. `Command::spawn()` writes credentials directly into the child's environment. No network round-trip, no side channel, no secondary auth flow. The parent (LLM context) never writes the value.

| | Shell env | Proxy gateway | Vault + k8s | systemprompt.io |
|---|---|---|---|---|
| Credentials absent from LLM context | Risk | Risk | Risk | Yes |
| Team-enforced policies | No | Yes | No | Yes |
| Air-gap capable | Yes | No | Partial | Yes |
| Per-agent scope | No | Partial | No | Yes |
| Audit trail | No | Partial | Yes | Yes |
| Single binary | — | No | No | Yes |
| Kubernetes required | No | Yes | Yes | No |

---

## License

**This template** is [MIT](LICENSE). Fork it, modify it, use it however you like.

**[systemprompt-core](https://github.com/systempromptio/systemprompt-core)** is [BSL-1.1](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE): free for evaluation, testing, and non-production use. Production use requires a commercial license. Each version converts to Apache 2.0 four years after publication. Licensing enquiries: [ed@systemprompt.io](mailto:ed@systemprompt.io).

---

<div align="center">

[![systemprompt.io](https://img.shields.io/badge/systemprompt.io-2b6cb0?style=for-the-badge)](https://systemprompt.io) &nbsp; [![Core](https://img.shields.io/badge/systemprompt--core-2b6cb0?style=for-the-badge)](https://github.com/systempromptio/systemprompt-core) &nbsp; [![Documentation](https://img.shields.io/badge/documentation-16a34a?style=for-the-badge)](https://systemprompt.io/documentation/) &nbsp; [![Guides](https://img.shields.io/badge/guides-f97316?style=for-the-badge)](https://systemprompt.io/guides) &nbsp; [![Discord](https://img.shields.io/badge/discord-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/wkAbSuPWpr)

<sub>Own how your organization uses AI. Every interaction governed and provable.</sub>

</div>
