<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://systemprompt.io/files/images/logo.svg">
  <source media="(prefers-color-scheme: light)" srcset="https://systemprompt.io/files/images/logo-dark.svg">
  <img src="https://systemprompt.io/files/images/logo-dark.svg" alt="systemprompt.io" width="380">
</picture>

# Own how your organization uses AI.

### Every Claude, OpenAI, and Gemini tool call audited before it runs. Self-hosted Rust binary. Air-gap capable. Built for SOC 2, ISO 27001, HIPAA, and the OWASP Agentic Top 10.

[![Built on systemprompt-core](https://img.shields.io/badge/built%20on-systemprompt--core-2b6cb0?style=flat-square)](https://github.com/systempromptio/systemprompt-core)
[![Template · MIT](https://img.shields.io/badge/template-MIT-16a34a?style=flat-square)](LICENSE)
[![Core · BSL--1.1](https://img.shields.io/badge/core-BSL--1.1-2b6cb0?style=flat-square)](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75+-f97316?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![PostgreSQL 18+](https://img.shields.io/badge/postgres-18+-336791?style=flat-square&logo=postgresql&logoColor=white)](https://www.postgresql.org/)

[**systemprompt.io**](https://systemprompt.io) · [**Documentation**](https://systemprompt.io/documentation/) · [**Guides**](https://systemprompt.io/guides) · [**Discord**](https://discord.gg/wkAbSuPWpr)

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-secrets.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-secrets.svg">
  <img src="demo/recording/svg/output/dark/cap-secrets.svg" alt="An AI agent attempts to exfiltrate a GitHub PAT through a tool call. The secret-detection layer denies the call before the tool process spawns. One row is written to the audit table. The recording is a live capture of `./demo/governance/06-secret-breach.sh`." width="820">
</picture>

<sub>Live capture of <code>./demo/governance/06-secret-breach.sh</code>. Secret exfiltration attempt denied before spawn. One audit row written. No model touched the key.</sub>

</div>

---

## What a CISO gets

- **A single query answers every AI audit.** Every request, scope decision, tool call, model output, and cost lands in one 18-column Postgres table. Six correlation columns (UserId, SessionId, TaskId, TraceId, ContextId, ClientId) bind identity at construction time, so a row without a trace is a programming error.
- **Credentials physically cannot enter the context window.** The governance process is the parent of every MCP tool subprocess. Keys are decrypted from a ChaCha20-Poly1305 store and injected into the child's environment by `Command::spawn()`. The parent, which owns the LLM context, never writes the value. 35+ regex patterns deny any tool call that tries to pass a secret through arguments.
- **Self-hosted, air-gap capable, single artifact.** One Rust binary. One PostgreSQL. No Redis, no Kafka, no Kubernetes, no SaaS handoff. The same binary runs on a laptop, a VM, and an air-gapped appliance without modification. Zero outbound telemetry by default.
- **Policy-as-code on PreToolUse hooks.** Destructive operations, blocklists, department scoping, six-tier RBAC (Admin, User, Service, A2A, MCP, Anonymous). Rate limiting at 300 req/min per session with role multipliers. Every deny reason is structured and auditable.
- **Certifications-ready, not certification-marketing.** Tiered log retention from debug (1 day) through error (90 days). 10 identity lifecycle event variants. SIEM-ready JSON events for Splunk, ELK, Datadog, Sumo. Built for **SOC 2 Type II**, **ISO 27001**, **HIPAA**, and the **OWASP Agentic Top 10**.

This repo is the evaluation template. Fork it, clone it, compile it. 43 scripted demos execute every claim above against the live binary on your own laptop.

---

## Quick start

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
- **`./demo/governance/06-secret-breach.sh`** — the scripted version of that denial, recorded above.

### The scripted demos

```bash
./demo/00-preflight.sh                    # acquire token, verify services, create admin
./demo/01-seed-data.sh                    # populate analytics + trace data

# Governance — the audit line
./demo/governance/01-happy-path.sh        # allowed tool call, full trace chain
./demo/governance/05-governance-denied.sh # scope check rejects out-of-role call
./demo/governance/06-secret-breach.sh     # secret-detection blocks exfiltration
./demo/governance/07-rate-limiting.sh     # 300 req/min per session enforced
./demo/governance/08-hooks.sh             # PreToolUse policy-as-code

# Observability — the audit table
./demo/analytics/01-overview.sh           # conversations, costs, anomalies
./demo/infrastructure/04-logs.sh          # structured JSON events, SIEM-ready

# Scale — the overhead budget
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

## The governance pipeline

Every tool call passes five in-process checks, synchronously, before it reaches a tool process. Every decision lands in an 18-column audit row.

```
  LLM Agent
      │
      ▼
  Governance pipeline  (in-process, synchronous, <5 ms p99)
      │
      ├─ 1. JWT validation       (HS256, verified locally, offline-capable)
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

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-governance.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-governance.svg">
  <img src="demo/recording/svg/output/dark/cap-governance.svg" alt="Governance pipeline — terminal recording" width="820">
</picture>

<sub>Run it: <code>./demo/governance/05-governance-denied.sh</code> · <a href="https://systemprompt.io/features/governance-pipeline">Feature detail</a></sub>

---

## How credential injection works

When a tool call passes the pipeline, `spawn_server()` decrypts credentials from the ChaCha20-Poly1305 store and injects them into the child process environment. The parent process — which owns the LLM context window — never writes the value.

Source: [`systemprompt-core/crates/domain/mcp/src/services/process/spawner.rs`](https://github.com/systempromptio/systemprompt-core/blob/main/crates/domain/mcp/src/services/process/spawner.rs).

```rust
let secrets = SecretsBootstrap::get()?;

let mut child_command = Command::new(&binary_path);

// Child env only. The parent (LLM context path) never touches the value.
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

Before spawn, a secret-detection pipeline scans tool arguments for 35+ credential patterns. A tool call that tries to pass a secret through the context window is blocked even if the agent has scope to run the tool. The hero recording above is the scripted proof: `./demo/governance/06-secret-breach.sh`.

---

## Performance

Sub-5 ms governance overhead, benchmarked. Each request performs JWT validation, scope resolution, three rule evaluations, and an async audit write.

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

Runtime configuration is flat YAML under `services/`, loaded through `services/config/config.yaml`. Unknown keys fail loudly (`#[serde(deny_unknown_fields)]`). No database-stored config, no admin UI required. Every change is a diff.

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

Eight CLI domains cover every operational surface. No dashboard required for any task.

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

---

<details>
<summary><strong>More recordings</strong> — infrastructure, integrations, analytics, agents, compliance, MCP governance</summary>

<br>

Each recording is a live capture of the named script running against the binary.

**Infrastructure** — one binary, one process, one database. Same artifact runs laptop to air-gap.

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-self-hosted.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-self-hosted.svg"><img src="demo/recording/svg/output/dark/infra-self-hosted.svg" alt="Self-hosted deployment" width="820"></picture>

<sub>All data on your infrastructure, zero outbound telemetry · <code>./demo/infrastructure/01-services.sh</code> · <a href="https://systemprompt.io/features/self-hosted-ai-platform">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-deploy-anywhere.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-deploy-anywhere.svg"><img src="demo/recording/svg/output/dark/infra-deploy-anywhere.svg" alt="Deploy anywhere" width="820"></picture>

<sub>Profile YAML promotes environments without rebuilding · <code>./demo/cloud/01-cloud-auth.sh</code> · <a href="https://systemprompt.io/features/deploy-anywhere">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-control-plane.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-control-plane.svg"><img src="demo/recording/svg/output/dark/infra-control-plane.svg" alt="Unified control plane" width="820"></picture>

<sub>Every operational surface has a CLI verb · <code>./demo/infrastructure/03-jobs.sh</code> · <a href="https://systemprompt.io/features/unified-control-plane">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-open-standards.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-open-standards.svg"><img src="demo/recording/svg/output/dark/infra-open-standards.svg" alt="Open standards" width="820"></picture>

<sub>MCP, OAuth 2.0, PostgreSQL, Git · zero proprietary protocols · <code>./demo/mcp/01-servers.sh</code> · <a href="https://systemprompt.io/features/no-vendor-lock-in">Feature</a></sub>

---

**MCP governance, analytics, closed-loop agents, compliance.**

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-mcp.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-mcp.svg"><img src="demo/recording/svg/output/dark/cap-mcp.svg" alt="MCP governance" width="820"></picture>

<sub>Each MCP server is an isolated OAuth2 resource server with per-server scope validation · <code>./demo/mcp/02-access-tracking.sh</code> · <a href="https://systemprompt.io/features/mcp-governance">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-analytics.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-analytics.svg"><img src="demo/recording/svg/output/dark/cap-analytics.svg" alt="Analytics and observability" width="820"></picture>

<sub>Nine analytics subcommands, anomaly detection, SIEM-ready JSON · <code>./demo/analytics/01-overview.sh</code> · <a href="https://systemprompt.io/features/analytics-and-observability">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-agents.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-agents.svg"><img src="demo/recording/svg/output/dark/cap-agents.svg" alt="Closed-loop agents" width="820"></picture>

<sub>Agents query their own error rate, cost, and latency via MCP tools and adjust · <code>./demo/agents/03-tracing.sh</code> · <a href="https://systemprompt.io/features/closed-loop-agents">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-compliance.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-compliance.svg"><img src="demo/recording/svg/output/dark/cap-compliance.svg" alt="Compliance" width="820"></picture>

<sub>Tiered retention, 10 identity lifecycle events, SOC 2 / ISO 27001 / HIPAA / OWASP Agentic Top 10 · <code>./demo/users/03-session-management.sh</code> · <a href="https://systemprompt.io/features/compliance">Feature</a></sub>

---

**Integrations** — any provider, Claude Desktop, web publisher, extensions.

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-any-agent.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-any-agent.svg"><img src="demo/recording/svg/output/dark/int-any-agent.svg" alt="Any AI agent" width="820"></picture>

<sub>Anthropic, OpenAI, Gemini swap at the profile level · cost attribution in integer microdollars · <code>./demo/agents/01-agents.sh</code> · <a href="https://systemprompt.io/features/any-ai-agent">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-cowork.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-cowork.svg"><img src="demo/recording/svg/output/dark/int-cowork.svg" alt="Claude Desktop & Cowork" width="820"></picture>

<sub>Skills persist across sessions via OAuth2 · <code>./demo/skills/01-skills.sh</code> · <a href="https://systemprompt.io/features/cowork">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-web-publisher.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-web-publisher.svg"><img src="demo/recording/svg/output/dark/int-web-publisher.svg" alt="Web server & publisher" width="820"></picture>

<sub>Same binary serves your website, blog, and docs · systemprompt.io runs on this binary · <code>./demo/web/01-web-config.sh</code> · <a href="https://systemprompt.io/features/web-publisher">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-extensions.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-extensions.svg"><img src="demo/recording/svg/output/dark/int-extensions.svg" alt="Extensible architecture" width="820"></picture>

<sub>Your code compiles into your binary via the <code>Extension</code> trait · no runtime reflection · <code>./demo/skills/05-plugins.sh</code> · <a href="https://systemprompt.io/features/extensible-architecture">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-benchmark.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-benchmark.svg"><img src="demo/recording/svg/output/dark/int-benchmark.svg" alt="Governance benchmark" width="820"></picture>

<sub>3,308 req/s burst, p99 22.7 ms · <code>just benchmark</code></sub>

</details>

---

## License

**This template** is [MIT](LICENSE). Fork it, modify it, use it however you like.

**[systemprompt-core](https://github.com/systempromptio/systemprompt-core)** is [BSL-1.1](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE): free for evaluation, testing, and non-production use. Production use requires a commercial license. Each version converts to Apache 2.0 four years after publication. Licensing enquiries: [ed@systemprompt.io](mailto:ed@systemprompt.io).

---

<div align="center">

[![systemprompt.io](https://img.shields.io/badge/systemprompt.io-2b6cb0?style=for-the-badge)](https://systemprompt.io) &nbsp; [![Core](https://img.shields.io/badge/systemprompt--core-2b6cb0?style=for-the-badge)](https://github.com/systempromptio/systemprompt-core) &nbsp; [![Documentation](https://img.shields.io/badge/documentation-16a34a?style=for-the-badge)](https://systemprompt.io/documentation/) &nbsp; [![Guides](https://img.shields.io/badge/guides-f97316?style=for-the-badge)](https://systemprompt.io/guides) &nbsp; [![Discord](https://img.shields.io/badge/discord-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/wkAbSuPWpr)

<sub>Own how your organization uses AI. Every interaction governed and provable.</sub>

</div>
