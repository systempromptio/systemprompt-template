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

**AI governance infrastructure for agentic systems.** A single compiled Rust binary that authenticates, authorises, rate-limits, logs, and costs every AI interaction before it reaches a tool, a database, or an external service. Self-hosted, air-gap capable, provider-agnostic. One binary (~50 MB). One database (PostgreSQL). No microservices. No Kubernetes. No Redis. No Kafka.

This repo is how you evaluate it. Clone it, run it on your own machine, bring your own AI key, and watch every claim on this page execute against 40+ scripted demos. The terminal recordings below are real captures of those demos running, not mockups.

> **Ready to evaluate?** Three commands from fresh clone to running platform. Every recording on this page is a real script you can run yourself.
>
> **Looking at the source?** This template is MIT. The engine is [systemprompt-core](https://github.com/systempromptio/systemprompt-core) (BSL-1.1), a 30-crate Rust workspace published on crates.io under `systemprompt-*`.
>
> **Active:** 635 unique cloners in the first 14 days with zero paid acquisition — engineers who found it via crates.io and GitHub search.

```
  LLM Agent
      |
      v
  [Governance Pipeline — in-process, synchronous]
      |
      +-- 1. JWT validation
      +-- 2. RBAC scope check  (Admin/User/Service/A2A/MCP/Anonymous)
      +-- 3. Secret detection  (35+ regex patterns — API keys, PATs, PEM headers)
      +-- 4. Blocklist         (destructive operation categories)
      +-- 5. Rate limiting     (300 req/min per session, role-based multipliers)
      |
      v
  ALLOW or DENY  (every decision written to 18-column audit table)
      |
      v (ALLOW only)
  spawn_server()
      |
      +-- load secrets from encrypted store (ChaCha20-Poly1305)
      +-- inject into subprocess environment variables only
      |
      v
  [MCP Tool Process]  <-- credentials live here, passed via env at spawn time

  The parent process (LLM context path) never writes credential values.
```

## Table of Contents

- [Get started](#get-started)
- [How credential injection works](#how-credential-injection-works)
- [Infrastructure](#infrastructure) — Self-hosted deployment, deploy anywhere, unified control plane, open standards
- [Capabilities](#capabilities) — Governance pipeline, secrets management, MCP governance, analytics, agents, compliance
- [Integrations](#integrations) — Any AI agent, Claude Desktop & Cowork, web publisher, extensible architecture
- [Performance](#performance)
- [Configuration](#configuration)
- [Compared to alternatives](#compared-to-alternatives)
- [License](#license)

---

## Get started

```bash
just build                                               # 1. compile the workspace
just setup-local <anthropic> <openai> <gemini>           # 2. profile + Postgres + publish
just start                                               # 3. serve governance, agents, MCP, admin, API
```

Open **http://localhost:8080** and run `systemprompt --help`. Point Claude Code, Claude Desktop, or any MCP client at it. Permissions follow the user, not the client.

### Prerequisites

| Requirement | Purpose | Install |
|---|---|---|
| **Docker** | PostgreSQL runs in a container. `just setup-local` starts it automatically | [docker.com](https://docs.docker.com/get-docker/) |
| **Rust 1.75+** | Compiles the workspace binary | [rustup.rs](https://rustup.rs/) |
| **`just`** | Task runner for build, setup, and start commands | [just.systems](https://just.systems/) |
| **`jq`** | JSON processing for config and session management | `brew install jq` / `apt install jq` |
| **`yq`** | YAML processing for profile and secrets config | `brew install yq` / `pip install yq` |
| **AI API keys** | Keys for every provider enabled in `services/ai/config.yaml`. The shipped config enables all three (Anthropic, OpenAI, Gemini) with `default_provider: gemini`, so the marketplace MCP server will refuse to boot unless each enabled provider has a real key. Disable the ones you don't want in the AI config, or pass all three. | Provider dashboards |
| **Ports `8080` + `5432`** | HTTP server + PostgreSQL | Free on localhost |

Running a second clone side-by-side? `just setup-local <anthropic> <openai> <gemini> 8081 5433`.

---

## How credential injection works

When a tool call passes the governance pipeline, `spawn_server()` loads credentials from the encrypted store and injects them directly into the child process environment — never into the parent process that handles LLM communication.

The mechanism is about 30 lines in [`systemprompt-core/crates/domain/mcp/src/services/process/spawner.rs`](https://github.com/systempromptio/systemprompt-core/blob/main/crates/domain/mcp/src/services/process/spawner.rs):

```rust
// Secrets are decrypted from ChaCha20-Poly1305 encrypted store
let secrets = SecretsBootstrap::get()?;

let mut child_command = Command::new(&binary_path);

// Injected into subprocess env vars only — never the LLM context path
if let Some(key) = &secrets.anthropic {
    child_command.env("ANTHROPIC_API_KEY", key);
}
if let Some(key) = &secrets.openai {
    child_command.env("OPENAI_API_KEY", key);
}
if let Some(key) = &secrets.github {
    child_command.env("GITHUB_TOKEN", key);
}

// Subprocess is detached — parent process forgets it after spawn
let child = child_command.spawn()?;
std::mem::forget(child);
```

The parent process — which owns the LLM context window — never touches these values. `std::mem::forget(child)` detaches the subprocess so the governance binary does not wait on it; the tool process runs independently with credentials in its own environment only.

**The second defence layer** is a secret detection pipeline that runs *before* spawn: 35+ regex patterns catch API key formats, GitHub PATs, PEM headers, AWS access key prefixes, and more. A tool call that passes a secret through the context window is blocked even if the agent has sufficient scope. Try it:

```bash
./demo/governance/06-secret-breach.sh
```

---

## Infrastructure

**One binary. One database. Deploys anywhere.** What others assemble from six services, this ships as a single 50MB Rust binary with PostgreSQL as the only dependency. Four in-process services, 144 database tables, zero sidecars, one PID to monitor. The same artifact runs on Docker, bare metal, cloud, or an air-gapped network without modification.

Configuration is profile-based YAML checked into version control. Agents, MCP servers, skills, AI providers, content sources, scheduler jobs, and web themes all live as flat files under `services/`. Environment drift is a diff, not a mystery. Every operation that works against localhost takes a `--profile` flag and works identically against staging or production.

Eight CLI domains cover every operational surface. No dashboard required for any task:

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

Every layer uses an open standard: **MCP** for tool communication, **OAuth 2.0** and **WebAuthn** for identity, **ChaCha20-Poly1305** for encryption at rest, **PostgreSQL** for storage, **Git** for distribution. No proprietary protocols at any layer. You can leave without a migration.

### Self-hosted deployment

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-self-hosted.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-self-hosted.svg">
  <img src="demo/recording/svg/output/dark/infra-self-hosted.svg" alt="Self-hosted deployment — terminal recording" width="820">
</picture>

> One binary, one process, one database. All data stays on your infrastructure. No outbound telemetry. &nbsp; [![Learn more](https://img.shields.io/badge/learn%20more-self--hosted-2b6cb0?style=flat-square)](https://systemprompt.io/features/self-hosted-ai-platform)

### Deploy anywhere

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-deploy-anywhere.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-deploy-anywhere.svg">
  <img src="demo/recording/svg/output/dark/infra-deploy-anywhere.svg" alt="Deploy anywhere — terminal recording" width="820">
</picture>

> Profile-based config means the same binary promotes from laptop to production without rebuilding. JWT validation and rate limiting execute locally per process without distributed infrastructure. &nbsp; [![Learn more](https://img.shields.io/badge/learn%20more-deploy%20anywhere-2b6cb0?style=flat-square)](https://systemprompt.io/features/deploy-anywhere)

### Unified control plane

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-control-plane.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-control-plane.svg">
  <img src="demo/recording/svg/output/dark/infra-control-plane.svg" alt="Unified control plane — terminal recording" width="820">
</picture>

> Govern, observe, and manage from one binary. `systemprompt <domain> --help` works everywhere. The same CLI surface drives local dev and production operations. &nbsp; [![Learn more](https://img.shields.io/badge/learn%20more-control%20plane-2b6cb0?style=flat-square)](https://systemprompt.io/features/unified-control-plane)

### Open standards

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-open-standards.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-open-standards.svg">
  <img src="demo/recording/svg/output/dark/infra-open-standards.svg" alt="Open standards — terminal recording" width="820">
</picture>

> MCP for tools, OAuth 2.0 for identity, PostgreSQL for storage, Git for distribution. Zero proprietary protocols at any layer. &nbsp; [![Learn more](https://img.shields.io/badge/learn%20more-open%20standards-2b6cb0?style=flat-square)](https://systemprompt.io/features/no-vendor-lock-in)

---

## Capabilities

**Every tool call governed. Synchronous evaluation before execution, not after.** Four layers of enforcement in the request path: scope check against a six-tier RBAC hierarchy (Admin 10x, User 1x, Service 5x, A2A 5x, MCP 5x, Anonymous 0.5x), secret detection with 35+ regex patterns, blocklist for destructive operations, and rate limiting at 300 requests per minute per session with role-based multipliers. Deny reasons are structured and auditable. Single-digit milliseconds overhead. No sidecar. No proxy.

**Secrets never touch inference.** The agent calls the tool, the MCP service injects the credential server-side via subprocess environment variables, and the LLM never sees it. Per-user key hierarchy encrypted with ChaCha20-Poly1305 AEAD. 12 dangerous file extensions blocked at the edge. 20+ scanner tool signatures identified before they reach the application layer.

**Every decision lands in an 18-column audit table** with 17 indexes, queryable from the CLI or exportable to your SIEM via structured JSON events. Cost tracking in integer microdollars by model, agent, and department. Nothing is sampled. Nothing is approximate. A single TraceId correlates every event from login through model output. Six correlation columns (UserId, SessionId, TaskId, TraceId, ContextId, ClientId) bind identity at construction time so a row that reaches the database without a trace is a programming error.

### Governance pipeline

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-governance.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-governance.svg">
  <img src="demo/recording/svg/output/dark/cap-governance.svg" alt="Governance pipeline — terminal recording" width="820">
</picture>

> Scope, secret scan, blocklist, rate limit. All synchronous, all in-process, all audited. The backup line of defense behind tool mapping. &nbsp; [![Learn more](https://img.shields.io/badge/learn%20more-governance-2b6cb0?style=flat-square)](https://systemprompt.io/features/governance-pipeline)

### Secrets management

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-secrets.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-secrets.svg">
  <img src="demo/recording/svg/output/dark/cap-secrets.svg" alt="Secrets management — terminal recording" width="820">
</picture>

> Credentials encrypted at rest with ChaCha20-Poly1305 and injected server-side at tool-call time. They never enter the context window, never appear in logs, never transit the inference path. Even an admin-scope agent cannot exfiltrate an AWS key, a GitHub PAT, or a PEM private key. &nbsp; [![Learn more](https://img.shields.io/badge/learn%20more-secrets-2b6cb0?style=flat-square)](https://systemprompt.io/features/secrets-management)

### MCP governance

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-mcp.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-mcp.svg">
  <img src="demo/recording/svg/output/dark/cap-mcp.svg" alt="MCP governance — terminal recording" width="820">
</picture>

> Each MCP server operates as an independent OAuth2 resource server with isolated credentials and per-server scope validation. If a tool is not declared in the plugin manifest, it does not exist for that agent. Four-pass deploy-time validation catches port conflicts, missing configs, empty OAuth scopes, and malformed server types before anything starts. &nbsp; [![Learn more](https://img.shields.io/badge/learn%20more-MCP-2b6cb0?style=flat-square)](https://systemprompt.io/features/mcp-governance)

### Analytics & observability

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-analytics.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-analytics.svg">
  <img src="demo/recording/svg/output/dark/cap-analytics.svg" alt="Analytics and observability — terminal recording" width="820">
</picture>

> Nine CLI subcommands cover overview, conversations, agents, tools, requests, sessions, content, traffic, and costs. Anomaly detection flags values exceeding 2x (warning) or 3x (critical) of the rolling average. SIEM-ready JSON event streaming for Splunk, ELK, Datadog, Sumo Logic. &nbsp; [![Learn more](https://img.shields.io/badge/learn%20more-analytics-2b6cb0?style=flat-square)](https://systemprompt.io/features/analytics-and-observability)

### Closed-loop agents

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-agents.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-agents.svg">
  <img src="demo/recording/svg/output/dark/cap-agents.svg" alt="Closed-loop agents — terminal recording" width="820">
</picture>

> Agents query their own error rate, cost, and latency through exposed MCP tools and adjust without a human in the loop. Every logged event carries eight correlation fields for complete request lineage reconstruction. A2A protocol enables multi-provider agent workflows with full governance on every hop. &nbsp; [![Learn more](https://img.shields.io/badge/learn%20more-agents-2b6cb0?style=flat-square)](https://systemprompt.io/features/closed-loop-agents)

### Compliance

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-compliance.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-compliance.svg">
  <img src="demo/recording/svg/output/dark/cap-compliance.svg" alt="Compliance — terminal recording" width="820">
</picture>

> Identity-bound audit trails via JWT with 10 lifecycle event variants. Tiered log retention from debug (1 day) through error (90 days). Policy-as-code on PreToolUse hooks. All data on-premises, no outbound telemetry. Built for **SOC 2 Type II**, **ISO 27001**, **HIPAA**, and **OWASP Agentic Top 10**. &nbsp; [![Learn more](https://img.shields.io/badge/learn%20more-compliance-2b6cb0?style=flat-square)](https://systemprompt.io/features/compliance)

---

## Integrations

**Provider-agnostic by trait, not by adapter.** The `AiProvider` trait abstracts 19 methods so switching between Anthropic, OpenAI, and Gemini changes config, not code. Cost attribution tracks spend across providers, models, and agents with microdollar precision. Same RBAC, same audit trail, same enforcement pipeline regardless of which model is behind the call.

**Extensions compile into the binary.** The `Extension` trait exposes routes, schemas, migrations, jobs, LLM providers, tool providers, page prerenderers, roles, and config namespaces. Registration happens at link time via `register_extension!` with no runtime reflection and no dynamic loading. 12 extensions, 71 sqlx-checked schemas, and 13 background jobs ship by default.

**Skills persist across sessions.** Claude Desktop and Cowork users install once via OAuth2 and get governed slash commands in every session. The same binary that governs AI agents also serves your website, blog, and documentation with Markdown content, PostgreSQL full-text search, and engagement analytics. systemprompt.io itself runs on this binary.

### Any AI agent

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-any-agent.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-any-agent.svg">
  <img src="demo/recording/svg/output/dark/int-any-agent.svg" alt="Any AI agent — terminal recording" width="820">
</picture>

> One governance layer for every provider. Swap Anthropic, OpenAI, or Gemini at the profile level. &nbsp; [![Learn more](https://img.shields.io/badge/learn%20more-any%20agent-2b6cb0?style=flat-square)](https://systemprompt.io/features/any-ai-agent)

### Claude Desktop & Cowork

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-cowork.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-cowork.svg">
  <img src="demo/recording/svg/output/dark/int-cowork.svg" alt="Claude Desktop & Cowork — terminal recording" width="820">
</picture>

> Skills persist across sessions via OAuth2. Slash commands activate business skills governed by the same four-layer pipeline. Install once, sync forever. &nbsp; [![Learn more](https://img.shields.io/badge/learn%20more-cowork-2b6cb0?style=flat-square)](https://systemprompt.io/features/cowork)

### Web Server & Publisher

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-web-publisher.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-web-publisher.svg">
  <img src="demo/recording/svg/output/dark/int-web-publisher.svg" alt="Web Server & Publisher — terminal recording" width="820">
</picture>

> The same binary that governs AI agents serves your website, blog, and documentation. Markdown content, PostgreSQL full-text search, engagement analytics. No separate web tier, no CMS. &nbsp; [![Learn more](https://img.shields.io/badge/learn%20more-web%20publisher-2b6cb0?style=flat-square)](https://systemprompt.io/features/web-publisher)

### Extensible architecture

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-extensions.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-extensions.svg">
  <img src="demo/recording/svg/output/dark/int-extensions.svg" alt="Extensions and capabilities — terminal recording" width="820">
</picture>

> Your code compiles into your binary. Add routes, tables, jobs, and providers through Rust extension traits. No runtime reflection, no dynamic loading. &nbsp; [![Learn more](https://img.shields.io/badge/learn%20more-extensions-2b6cb0?style=flat-square)](https://systemprompt.io/features/extensible-architecture)

---

## Performance

Sub-5 ms governance overhead, benchmarked. Each request performs JWT validation, scope resolution, three rule evaluations, and an async database write. The enforcement pipeline is not a bottleneck.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-benchmark.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-benchmark.svg">
  <img src="demo/recording/svg/output/dark/int-benchmark.svg" alt="Governance benchmark — terminal recording" width="820">
</picture>

| Metric | Result |
|---|---|
| **Throughput** | 3,308 req/s (burst), sustained under 100 concurrent workers |
| **p50 latency** | 13.5 ms |
| **p99 latency** | 22.7 ms |
| **Overhead** | <1% added to AI response time |
| **GC pauses** | Zero. Hundreds of concurrent developers on a single instance. |

Reproduce with `just benchmark`. Numbers measured on the author's laptop.

---

## Configuration

All runtime configuration lives as flat YAML files under `services/`. The root `services/config/config.yaml` is a thin aggregator. Unknown YAML keys cause loud errors at load time (`#[serde(deny_unknown_fields)]`).

```
services/
  config/config.yaml        Root aggregator (includes all resource files)
  agents/<id>.yaml          Agent definitions with scope, model, and tool access
  mcp/<name>.yaml           MCP server definitions with OAuth2 config
  skills/<id>.yaml          Skill definitions (config + markdown instruction body)
  plugins/<name>.yaml       Plugin bindings referencing agents, skills, MCP servers
  ai/config.yaml            AI provider config (Anthropic, OpenAI, Gemini)
  scheduler/config.yaml     Background job schedule
  web/config.yaml           Web frontend, navigation, theme
  content/config.yaml       Content sources and indexing
```

Every resource is a flat file you can diff, review, and version. No database-stored config. No admin UI required for any configuration change.

---

## Compared to alternatives

These are architectural differences, not marketing comparisons.

**Shell environment variables (`~/.claude/settings.json` DENY lists, per-developer key config)**

The most common approach: each developer sets up their own environment. No enforcement at the team level, no audit trail, no per-agent scope, no anomaly detection. A developer can add or remove keys without any visibility. If a new hire configures their environment incorrectly, nothing catches it.

**Proxy-layer gateways (Microsoft Azure AI Gateway, Kong AI Gateway, nginx-based approaches)**

These intercept HTTP between the LLM client and its destination. They can observe and block traffic but do not own the execution context of tools. Credential injection through a proxy requires the credential to transit the proxy as a value (in headers or request transformation) — the proxy process can log it, and a misconfigured proxy can leak it. Microsoft AGT is also Azure-native: not self-hostable on bare metal, not air-gap capable, requires Azure infrastructure.

**Vault + Kubernetes Secrets**

Manages credential storage well but is not enforcement at the point of LLM tool use. When an MCP tool needs a secret, it makes a network call to retrieve it — a call the LLM agent can observe, manipulate via prompt injection, or trigger at an unintended time. Vault does not know what the LLM is doing or why; it only knows a service account authenticated and requested a secret.

**This implementation**

The governance binary is the parent process of every MCP tool subprocess. `Command::spawn()` injects credentials directly into the child's environment. No network round-trip. No side-channel. No secondary auth flow. The parent process (LLM context) never writes credential values. The subprocess (tool execution) never makes an outbound credential request. Enforcement happens at the transport layer, not the proxy layer.

| | Shell env | Proxy gateway | Vault + k8s | systemprompt.io |
|---|---|---|---|---|
| **Credentials in LLM context** | Risk | Risk | Risk | No |
| **Team-enforced policies** | No | Yes | No | Yes |
| **Air-gap capable** | Yes | No (Azure/cloud) | Partial | Yes |
| **Per-agent scope** | No | Partial | No | Yes |
| **Audit trail** | No | Partial | Yes | Yes |
| **Single binary** | — | No | No | Yes |
| **Kubernetes required** | No | Yes | Yes | No |

---

## License

**This template** is [MIT](LICENSE). Fork it, modify it, use it however you like.

**[systemprompt-core](https://github.com/systempromptio/systemprompt-core)** is [BSL-1.1](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE): free for evaluation, testing, and non-production use. Production use requires a commercial license. Each version converts to Apache 2.0 four years after publication. Licensing enquiries: [ed@systemprompt.io](mailto:ed@systemprompt.io).

---

<div align="center">

[![systemprompt.io](https://img.shields.io/badge/systemprompt.io-2b6cb0?style=for-the-badge)](https://systemprompt.io) &nbsp; [![Core](https://img.shields.io/badge/systemprompt--core-2b6cb0?style=for-the-badge)](https://github.com/systempromptio/systemprompt-core) &nbsp; [![Documentation](https://img.shields.io/badge/documentation-16a34a?style=for-the-badge)](https://systemprompt.io/documentation/) &nbsp; [![Guides](https://img.shields.io/badge/guides-f97316?style=for-the-badge)](https://systemprompt.io/guides) &nbsp; [![Discord](https://img.shields.io/badge/discord-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/wkAbSuPWpr)

<sub>Own how your organization uses AI. Every interaction governed and provable.</sub>

</div>
