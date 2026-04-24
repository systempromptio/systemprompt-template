<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://systemprompt.io/files/images/logo.svg">
  <source media="(prefers-color-scheme: light)" srcset="https://systemprompt.io/files/images/logo-dark.svg">
  <img src="https://systemprompt.io/files/images/logo-dark.svg" alt="systemprompt.io" width="380">
</picture>

# Own how your organization uses AI.

One self-hosted binary governs inference, auditing, evals, and every tool call across your AI fleet. Native integration with Claude Cowork. Works with any agent, any model, any provider.

This repository is the evaluation template: clone it, compile it, point Claude for Work, Claude Code, or any Anthropic-SDK client at `http://localhost:8080`, and every request lands on a host you operate — on your network, in your air-gap, under your audit table. Single Rust binary, one PostgreSQL, four commands from `git clone` to serving inference. Built for SOC 2, ISO 27001, HIPAA, and the OWASP Agentic Top 10.

[![Built on systemprompt-core](https://img.shields.io/badge/built%20on-systemprompt--core-2b6cb0?style=flat-square)](https://github.com/systempromptio/systemprompt-core)
[![Template · MIT](https://img.shields.io/badge/template-MIT-16a34a?style=flat-square)](LICENSE)
[![Core · BSL--1.1](https://img.shields.io/badge/core-BSL--1.1-2b6cb0?style=flat-square)](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75+-f97316?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![PostgreSQL 18+](https://img.shields.io/badge/postgres-18+-336791?style=flat-square&logo=postgresql&logoColor=white)](https://www.postgresql.org/)

[**systemprompt.io**](https://systemprompt.io) · [**Documentation**](https://systemprompt.io/documentation/) · [**Guides**](https://systemprompt.io/guides) · [**Enterprise factsheet (PDF)**](https://systemprompt.io/files/documents/systemprompt-io-enterprise-factsheet.pdf) · [**Discord**](https://discord.gg/wkAbSuPWpr)

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-secrets.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-secrets.svg">
  <img src="demo/recording/svg/output/dark/cap-secrets.svg" alt="An AI agent attempts to exfiltrate a GitHub PAT through a tool call. The secret-detection layer denies the call before the tool process spawns. One row is written to the audit table. The recording is a live capture of `./demo/governance/06-secret-breach.sh`." width="820">
</picture>

<sub>Live capture of <code>./demo/governance/06-secret-breach.sh</code>. Secret exfiltration attempt denied before spawn. One audit row written. No model touched the key.</sub>

</div>

---

## Quick start

```bash
git clone https://github.com/systempromptio/systemprompt-template
cd systemprompt-template
just setup-local <anthropic_key> [openai_key] [gemini_key]   # writes profile, starts Postgres, runs publish pipeline
just start                                                   # serves governance + agents + MCP + admin on :8080
```

One AI key is required; the other two are optional. Running a second clone side-by-side? `just setup-local <keys> 8081 5433`. Discover the CLI with `systemprompt --help`.

<details>
<summary><strong>Prerequisites</strong></summary>

<br>

| Requirement | Purpose | Install |
|---|---|---|
| **Docker** | PostgreSQL runs in a container; `just setup-local` starts it | [docker.com](https://docs.docker.com/get-docker/) |
| **Rust 1.75+** | Compiles the workspace binary | [rustup.rs](https://rustup.rs/) |
| **`just`** | Task runner | [just.systems](https://just.systems/) |
| **`jq`, `yq`** | JSON and YAML processing in the scripts | `brew install jq yq` / `apt install jq yq` |
| **AI API keys** | One key per provider enabled in `services/ai/config.yaml`. Shipped config enables Anthropic, OpenAI, Gemini (default `gemini`). Disable providers you don't want or pass all three. | Provider dashboards |
| **Ports 8080 + 5432** | HTTP + PostgreSQL | Free on localhost |

</details>

---

<details>
<summary><strong>What a CISO gets</strong></summary>

<br>

- **A single query answers every AI audit.** Every request, scope decision, tool call, model output, and cost lands in one 18-column Postgres table. Six correlation columns (UserId, SessionId, TaskId, TraceId, ContextId, ClientId) bind identity at construction time, so a row without a trace is a programming error.
- **Credentials physically cannot enter the context window.** The governance process is the parent of every MCP tool subprocess. Keys are decrypted from a ChaCha20-Poly1305 store and injected into the child's environment by `Command::spawn()`. The parent, which owns the LLM context, never writes the value. 35+ regex patterns deny any tool call that tries to pass a secret through arguments.
- **Self-hosted, air-gap capable, single artifact.** One Rust binary. One PostgreSQL. No Redis, no Kafka, no Kubernetes, no SaaS handoff. The same binary runs on a laptop, a VM, and an air-gapped appliance without modification. Zero outbound telemetry by default.
- **Policy-as-code on PreToolUse hooks.** Destructive operations, blocklists, department scoping, six-tier RBAC (Admin, User, Service, A2A, MCP, Anonymous). Rate limiting at 300 req/min per session with role multipliers. Every deny reason is structured and auditable.
- **Certifications-ready, not certification-marketing.** Tiered log retention from debug (1 day) through error (90 days). 10 identity lifecycle event variants. SIEM-ready JSON events for Splunk, ELK, Datadog, Sumo. Built for **SOC 2 Type II**, **ISO 27001**, **HIPAA**, and the **OWASP Agentic Top 10**.

This repo is the evaluation template. Fork it, clone it, compile it. 43 scripted demos execute every claim above against the live binary on your own laptop.

</details>

<details>
<summary><strong>What you'll see in the first five minutes</strong></summary>

<br>

- **http://localhost:8080** — admin UI, live audit table, session viewer.
- **`systemprompt analytics overview`** — conversations, tool calls, costs in microdollars, anomalies flagged above 2x/3x of rolling average.
- **`systemprompt infra logs audit <request-id> --full`** — the full trace for any request: identity, scope, rule evaluations, tool call, model output, cost. One query, one row, one answer.
- **Point Claude Code, Claude Desktop, or any MCP client at it.** Permissions follow the user, not the client. Try to exfiltrate a key through a tool argument and watch the secret-detection layer deny it before the tool process spawns.
- **`./demo/governance/06-secret-breach.sh`** — the scripted version of that denial, recorded above.

</details>

<details>
<summary><strong>The scripted demos</strong></summary>

<br>

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

</details>

<details>
<summary><strong>The governance pipeline</strong></summary>

<br>

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

</details>

<details>
<summary><strong>How credential injection works</strong></summary>

<br>

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

</details>

<details>
<summary><strong>Performance</strong></summary>

<br>

Sub-5 ms governance overhead, benchmarked. Each request performs JWT validation, scope resolution, three rule evaluations, and an async audit write.

| Metric | Result |
|---|---|
| Throughput | 3,308 req/s burst, sustained under 100 concurrent workers |
| p50 latency | 13.5 ms |
| p99 latency | 22.7 ms |
| Added to AI response time | <1% |
| GC pauses | Zero |

Reproduce: `just benchmark`. Numbers measured on the author's laptop.

</details>

<details>
<summary><strong>Configuration & CLI</strong></summary>

<br>

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

</details>

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

<details>
<summary><strong>Claude for Work, on your infrastructure</strong></summary>

<br>

Claude for Work ships with extension points for inference, identity, and audit. Point them at this binary and every prompt, tool call, and cost line lands in a Postgres row you own.

```
  Managed Device                 Enterprise Gateway              Upstream Inference
  (Cowork via MDM)               (this binary, your VPC)         (pluggable)
  ───────────────── ──────────▶  ─────────────────────  ──────▶  ─────────────────
  Credential helper              /v1/messages                    Anthropic direct
  Managed MCP list               Governance pipeline             Bedrock / Vertex
  Signed plugins                 Audit to Postgres               OpenAI / Groq
                                                                 On-prem vLLM / Qwen
                                                                 Air-gap capable
```

Four governance layers enforce before a byte leaves your network:

- **Scope** — RBAC resolved from the JWT at request construction. Admin · User · Service · A2A · MCP · Anonymous.
- **Secrets** — 35+ regex patterns scan every tool argument and every prompt. A credential in the context path is denied before the tool process spawns.
- **Policy** — Blocklists, destructive-operation categories, tenant rules, PreToolUse hooks as code.
- **Quota** — 300 req/min per session with role multipliers; per-tool and per-budget limits.

In-process evaluation against a cached entitlement table. Governance stays out of the latency budget — p99 **22.7 ms**, <1% of AI response time.

### How it compares

| Dimension | Claude Enterprise | Cloud Custom | + systemprompt.io |
|---|---|---|---|
| **Data residency** | Anthropic infra | Cloud region | Your datacenter or air-gap |
| **Audit trail** | Anthropic-held | OTLP only | Prompt → tool → MCP → cost in your Postgres |
| **User revocation** | SSO / seat removal | Cloud IAM | IDP disable; next TTL fails closed |
| **Inference provider** | Anthropic only | Bedrock / Vertex (Claude) | Any `/v1/messages`, per-call routing |
| **MCP allowlist** | Anthropic-curated | Device-local config | One registry, per-principal policy |
| **Plugin catalogue** | Anthropic-hosted | Files on disk | Signed, scoped, versioned distribution |

Manual install is tested and works end-to-end today; signed installers, MDM packages, and Homebrew / winget distribution land in a later release. Install steps in the **Advanced** fold below.

</details>

<details>
<summary><strong>Advanced — gateway routes, Cowork install, org-plugins sync</strong></summary>

<br>

Manual install is tested end-to-end. Automated distribution — signed installers, MDM packages, Homebrew / winget — is in progress; today you download a binary and drop a TOML file, documented below.

### `/v1/messages` inference gateway

`POST /v1/messages` at the Anthropic wire format. Every inference request flows through the same governance pipeline as every tool call — on infrastructure you operate.

- **SDK- and Claude-Desktop-compatible.** Authenticated with a systemprompt JWT in `x-api-key` (falls back to `Authorization: Bearer`). No new credential type — existing user JWTs serve as the gateway credential.
- **Routes by `model_pattern`.** Built-in tags: `anthropic`, `openai`, `moonshot` (Kimi), `qwen`, `gemini`, `minimax`. Anthropic is a transparent byte proxy (extended thinking, cache-control headers, SSE events preserved verbatim). OpenAI-compatible providers get full Anthropic↔OpenAI request/response/SSE conversion. Upstream API keys resolve from the secrets file by name.
- **Zero overhead when disabled.** The `/v1` router mounts only if `gateway.enabled: true` in the active profile.


Profile YAML:

```yaml
gateway:
  enabled: true
  routes:
    - model_pattern: "claude-*"
      provider: anthropic
      endpoint: "https://api.anthropic.com/v1"
      api_key_secret: "anthropic_api_key"
    - model_pattern: "moonshot-*"
      provider: moonshot
      endpoint: "https://api.moonshot.cn/v1"
      api_key_secret: "kimi_api_key"
      upstream_model: "moonshot-v1-8k"
    - model_pattern: "MiniMax-*"
      provider: minimax
      endpoint: "https://api.minimax.io/anthropic"
      api_key_secret: "minimax"
    - model_pattern: "*"
      provider: anthropic
      endpoint: "https://api.anthropic.com/v1"
      api_key_secret: "anthropic_api_key"
```

Routes evaluate in order; first `model_pattern` match wins. `upstream_model` aliases a client-requested model to a different upstream name without the client knowing.

**Cowork credential helper endpoints.** `systemprompt-cowork` is a standalone ~2.4 MB Rust binary (no `tokio`, no `sqlx`, no `axum`) that trades a lower-privilege credential for a short-lived JWT. Progressive capability ladder — mTLS → dashboard session → PAT — mounted under `/v1/gateway/auth/cowork/`:

- `POST /pat` — `Authorization: Bearer <pat>` → `{token, ttl, headers}` with a fresh JWT and the canonical identity header map (`x-user-id`, `x-session-id`, `x-trace-id`, `x-client-id`, `x-tenant-id`, `x-policy-version`, `x-call-source`).
- `POST /session` — `501` (dashboard-cookie exchange not yet wired).
- `POST /mtls` — `501` (device-cert exchange not yet wired).
- `GET /capabilities` — `{"modes":["pat"]}`; probes advertise which exchange modes this deployment accepts.

The helper writes the signed JWT + expiry to the OS cache dir with mode `0600`. Stdout contract is exactly one JSON object; all diagnostics go to stderr. Released out-of-band as `cowork-v*` tags. Install / configure / wire-up steps below.

**Extensible provider registry.** `GatewayRoute.provider` is a free-form string resolved at dispatch time against a startup-built registry. Extension crates register new upstreams with:

```rust
inventory::submit! {
    systemprompt_api::services::gateway::GatewayUpstreamRegistration {
        tag: "my-provider",
        factory: || std::sync::Arc::new(MyUpstream),
    }
}
```

The `GatewayUpstream` trait (`async fn proxy(&self, ctx: UpstreamCtx<'_>)`) is the single integration seam. Built-in tags seeded automatically; extension tags may shadow built-ins (logged as a warning). Full detail: [`core/CHANGELOG.md`](https://github.com/systempromptio/systemprompt-core/blob/main/CHANGELOG.md#030---2026-04-22).

---

### Install the Cowork credential helper

The `systemprompt-cowork` binary is the **Credential helper script** slot in Claude for Work. It turns a PAT into a short-lived JWT that Claude Desktop merges into every inference request routed at this binary. Download the prebuilt Windows or Linux binary from [systempromptio/systemprompt-core releases](https://github.com/systempromptio/systemprompt-core/releases/tag/cowork-v0.2.0); macOS builds from source on any Mac.

Current release: **[cowork-v0.2.0](https://github.com/systempromptio/systemprompt-core/releases/tag/cowork-v0.2.0)** — Linux x86_64 + Windows x86_64 (mingw ABI). macOS build is pending a Mac-hosted CI.

#### 1. Download

**Linux x86_64**

```bash
curl -fsSL -o /usr/local/bin/systemprompt-cowork \
  https://github.com/systempromptio/systemprompt-core/releases/download/cowork-v0.2.0/systemprompt-cowork-x86_64-unknown-linux-gnu
chmod +x /usr/local/bin/systemprompt-cowork
curl -fsSL https://github.com/systempromptio/systemprompt-core/releases/download/cowork-v0.2.0/systemprompt-cowork-x86_64-unknown-linux-gnu.sha256 \
  | sha256sum -c --ignore-missing
```

**Windows x86_64** (PowerShell as Administrator):

```powershell
$dir = "C:\Program Files\systemprompt"
New-Item -ItemType Directory -Force -Path $dir | Out-Null
Invoke-WebRequest `
  -Uri "https://github.com/systempromptio/systemprompt-core/releases/download/cowork-v0.2.0/systemprompt-cowork-x86_64-pc-windows-gnu.exe" `
  -OutFile "$dir\systemprompt-cowork.exe"
[Environment]::SetEnvironmentVariable("PATH", "$env:PATH;$dir", "User")
```

Windows Smart Screen will flag the unsigned binary on first run → "More info" → "Run anyway".

**macOS** (source build):

```bash
git clone https://github.com/systempromptio/systemprompt-core.git
cd systemprompt-core
cargo build --manifest-path bin/cowork/Cargo.toml --release \
  --target "$(rustc -vV | awk '/host:/ {print $2}')"
sudo install -m 755 \
  "bin/cowork/target/$(rustc -vV | awk '/host:/ {print $2}')/release/systemprompt-cowork" \
  /usr/local/bin/
```

#### 2. Configure

Linux/macOS: `~/.config/systemprompt/systemprompt-cowork.toml`
Windows: `%APPDATA%\systemprompt\systemprompt-cowork.toml`

```toml
[gateway]
url = "http://localhost:8080"   # for the local-trial template; swap to your production host

[pat]
token = "sp-live-your-personal-access-token"
```

Issue a PAT from the running binary with `systemprompt admin users pat issue <user-id> --name cowork-laptop`. Absent config sections are silently skipped. Dev overrides: `SP_COWORK_GATEWAY_URL`, `SP_COWORK_PAT`.

#### 3. Verify

```bash
systemprompt-cowork           # prints exactly one JSON {token, ttl, headers}
systemprompt-cowork --check   # exits 0 if a token can be issued
```

Diagnostics go to stderr only. The stdout JSON matches Anthropic's `inferenceCredentialHelper` contract byte-for-byte.

#### 4. Point Claude Desktop at it

In Claude Desktop **Enterprise → Settings → Inference**:

- **Credential helper script**: `/usr/local/bin/systemprompt-cowork` (or `C:\Program Files\systemprompt\systemprompt-cowork.exe`).
- **API base URL**: the `gateway.url` from your TOML.

Every Claude Desktop request now lands a row in `ai_requests` with `user_id`, `tenant_id`, `session_id`, `trace_id`, tokens, cost, and latency — identical governance to every other tool call. Run `systemprompt infra logs audit <request-id> --full` after a prompt to see the trace end-to-end.

#### 5. (Optional) Install the `org-plugins/` sync agent

The same binary manages Cowork's signed plugin / managed-MCP mount:

```bash
systemprompt-cowork install     # register launchd (macOS) / scheduled task (Windows) / systemd --user (Linux)
systemprompt-cowork sync        # pull signed plugin manifest + allowlist now
systemprompt-cowork validate    # verify the ed25519 signature
systemprompt-cowork uninstall   # remove
```

Mount targets: `/Library/Application Support/Claude/org-plugins/` (macOS), `C:\ProgramData\Claude\org-plugins\` (Windows), `${XDG_DATA_HOME:-$HOME/.local/share}/Claude/org-plugins/` (Linux).

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
