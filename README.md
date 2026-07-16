<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://systemprompt.io/files/images/logo.svg">
  <source media="(prefers-color-scheme: light)" srcset="https://systemprompt.io/files/images/logo-dark.svg">
  <img src="https://systemprompt.io/files/images/logo-dark.svg" alt="systemprompt.io" width="380">
</picture>

# The self-owned AI control plane.

**The only AI infrastructure you actually own.** Most teams rent this layer: someone else's dashboard holds their prompts, their keys, and their audit trail. This is the version you compile and keep. One Rust binary, one PostgreSQL, four commands from `git clone` to governed inference. 43 scripted demos prove every claim on your own laptop.

[![Built on systemprompt-core](https://img.shields.io/badge/built%20on-systemprompt--core-2b6cb0?style=flat-square)](https://github.com/systempromptio/systemprompt-core)
[![Template · MIT](https://img.shields.io/badge/template-MIT-16a34a?style=flat-square)](LICENSE)
[![Core · BSL--1.1](https://img.shields.io/badge/core-BSL--1.1-2b6cb0?style=flat-square)](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75+-f97316?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![PostgreSQL 18+](https://img.shields.io/badge/postgres-18+-336791?style=flat-square&logo=postgresql&logoColor=white)](https://www.postgresql.org/)

[![Deploy on Railway](https://railway.com/button.svg)](https://railway.com/deploy/systempromptio-the-self-owned-ai-control?referralCode=AQ_ePp&utm_medium=integration&utm_source=template&utm_campaign=generic) &nbsp; [![Deploy to Render](https://render.com/images/deploy-to-render-button.svg)](https://render.com/deploy?repo=https://github.com/systempromptio/systemprompt-template) &nbsp; [![Deploy on Northflank](https://assets.northflank.com/deploy_to_northflank_smm_36700fb050.svg)](https://app.northflank.com/s/account/templates/new?data=6a58eb70982d53bd314abce3) &nbsp; [![Deploy on Zeabur](https://zeabur.com/button.svg)](https://zeabur.com/templates/OSPC37) &nbsp; <sub>[all install paths →](docs/README.md)</sub>

[**systemprompt.io**](https://systemprompt.io) · [**Documentation**](https://systemprompt.io/documentation/) · [**Guides**](https://systemprompt.io/guides) · [**Enterprise factsheet (PDF)**](https://systemprompt.io/files/documents/systemprompt-io-enterprise-factsheet.pdf) · [**Discord**](https://discord.gg/wkAbSuPWpr)

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-secrets.svg">
  <source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-secrets.svg">
  <img src="demo/recording/svg/output/dark/cap-secrets.svg" alt="An AI agent attempts to exfiltrate a GitHub PAT through a tool call. The secret-detection layer denies the call before the tool process spawns. One row is written to the audit table." width="820">
</picture>

<sub>Not a diagram. A live capture of <code>./demo/governance/06-secret-breach.sh</code>: an agent tries to exfiltrate a GitHub PAT through a tool argument. Denied in under 5 ms, before the tool process spawns. One audit row. The model never saw the key.</sub>

</div>

---

## Quick start

```bash
git clone https://github.com/systempromptio/systemprompt-template
cd systemprompt-template
just setup-local            # prompts: pick a provider (Gemini/Anthropic/OpenAI), enter its key
just start                  # serves governance + agents + MCP + admin on :8080
```

`setup-local` prompts for a provider key, or takes keys non-interactively (`just setup-local <anthropic_key> [openai_key] [gemini_key]`; the first becomes the default provider). Discover the CLI with `systemprompt --help`. All other install paths, including running a second clone on different ports, are in [docs/README.md](docs/README.md).

---

<details>
<summary><strong>For the CISO — one SQL query answers any AI audit</strong></summary>

<br>

Five properties, each one demonstrable on your laptop before any procurement call.

- **A single query answers every AI audit.** Every request, scope decision, tool call, model output, and cost lands in one 18-column Postgres table. Six correlation columns (UserId, SessionId, TaskId, TraceId, ContextId, ClientId) bind identity at construction time, so a row without a trace is a programming error.
- **Credentials physically cannot enter the context window.** The governance process is the parent of every MCP tool subprocess. Keys are decrypted from a ChaCha20-Poly1305 store and injected into the child's environment by `Command::spawn()`. The parent, which owns the LLM context, never writes the value. 35+ regex patterns deny any tool call that tries to pass a secret through arguments.
- **Self-hosted, air-gap capable, single artifact.** One Rust binary. One PostgreSQL. No Redis, no Kafka, no Kubernetes, no SaaS handoff. The same binary runs on a laptop, a VM, and an air-gapped appliance without modification. Zero outbound telemetry by default.
- **Policy-as-code on PreToolUse hooks.** Destructive operations, blocklists, department scoping, six-tier RBAC (Admin, User, Service, A2A, MCP, Anonymous). Rate limiting at 300 req/min per session with role multipliers. Every deny reason is structured and auditable.
- **Certifications-ready, not certification-marketing.** Tiered log retention from debug (1 day) through error (90 days). 10 identity lifecycle event variants. SIEM-ready JSON events for Splunk, ELK, Datadog, Sumo. Built for **SOC 2 Type II**, **ISO 27001**, **HIPAA**, and the **OWASP Agentic Top 10**.

</details>

<details>
<summary><strong>Run the proof — 43 scripted demos, 41 cost nothing</strong></summary>

<br>

Every claim in this README has a script that executes it against the live binary.

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
<summary><strong>The governance pipeline — five checks before any tool process spawns</strong></summary>

<br>

Every tool call passes five in-process checks, synchronously, in under 5 ms. Every decision lands in an 18-column audit row.

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
<summary><strong>Why agents cannot leak your keys — the code, twelve lines</strong></summary>

<br>

Not a policy that asks agents nicely. A process boundary: the parent that owns the LLM context never writes the credential value.

When a tool call passes the pipeline, `spawn_server()` decrypts credentials from the ChaCha20-Poly1305 store and injects them into the child process environment. Source: [`systemprompt-core/crates/domain/mcp/src/services/process/spawner.rs`](https://github.com/systempromptio/systemprompt-core/blob/main/crates/domain/mcp/src/services/process/spawner.rs).

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

Before spawn, secret detection scans tool arguments for 35+ credential patterns. A tool call that tries to pass a secret through the context window is blocked even if the agent has scope to run the tool. The hero recording above is the scripted proof: `./demo/governance/06-secret-breach.sh`.

</details>

<details>
<summary><strong>Performance — 3,308 req/s burst, p99 22.7 ms</strong></summary>

<br>

Governance that adds more than 1% latency gets bypassed. This one doesn't. Each request performs JWT validation, scope resolution, three rule evaluations, and an async audit write.

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
<summary><strong>Your first five minutes — admin UI, audit trace, live denial</strong></summary>

<br>

- **http://localhost:8080** — admin UI, live audit table, session viewer.
- **`systemprompt analytics overview`** — conversations, tool calls, costs in microdollars, anomalies flagged above 2x/3x of rolling average.
- **`systemprompt infra logs audit <request-id> --full`** — the full trace for any request: identity, scope, rule evaluations, tool call, model output, cost. One query, one row, one answer.
- **Point Claude Code, Claude Desktop, or any MCP client at it.** Permissions follow the user, not the client. Try to exfiltrate a key through a tool argument and watch the secret-detection layer deny it before the tool process spawns.
- **`./demo/governance/06-secret-breach.sh`** — the scripted version of that denial, recorded above.

</details>

<details>
<summary><strong>Configuration & CLI — everything is a YAML diff, every task has a verb</strong></summary>

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
<summary><strong>More recordings — infrastructure, integrations, analytics, agents, compliance</strong></summary>

<br>

Each recording is a live capture of the named script running against the binary.

**Infrastructure** — one binary, one process, one database. Same artifact runs laptop to air-gap.

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-self-hosted.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-self-hosted.svg"><img src="demo/recording/svg/output/dark/infra-self-hosted.svg" alt="Self-hosted deployment" width="820"></picture>

<sub>All data on your infrastructure, zero outbound telemetry · <code>./demo/infrastructure/01-services.sh</code> · <a href="https://systemprompt.io/features/self-hosted-ai-platform">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-deploy-anywhere.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-deploy-anywhere.svg"><img src="demo/recording/svg/output/dark/infra-deploy-anywhere.svg" alt="Deploy anywhere" width="820"></picture>

<sub>Profile YAML promotes environments without rebuilding · <code>./demo/cloud/01-cloud-overview.sh</code> · <a href="https://systemprompt.io/features/deploy-anywhere">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-control-plane.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-control-plane.svg"><img src="demo/recording/svg/output/dark/infra-control-plane.svg" alt="Unified control plane" width="820"></picture>

<sub>Every operational surface has a CLI verb · <code>./demo/infrastructure/03-jobs.sh</code> · <a href="https://systemprompt.io/features/unified-control-plane">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/infra-open-standards.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/infra-open-standards.svg"><img src="demo/recording/svg/output/dark/infra-open-standards.svg" alt="Open standards" width="820"></picture>

<sub>MCP, OAuth 2.0, PostgreSQL, Git · zero proprietary protocols · <code>./demo/mcp/01-mcp-servers.sh</code> · <a href="https://systemprompt.io/features/no-vendor-lock-in">Feature</a></sub>

---

**MCP governance, analytics, closed-loop agents, compliance.**

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-mcp.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-mcp.svg"><img src="demo/recording/svg/output/dark/cap-mcp.svg" alt="MCP governance" width="820"></picture>

<sub>Each MCP server is an isolated OAuth2 resource server with per-server scope validation · <code>./demo/mcp/02-mcp-access-tracking.sh</code> · <a href="https://systemprompt.io/features/mcp-governance">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-analytics.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-analytics.svg"><img src="demo/recording/svg/output/dark/cap-analytics.svg" alt="Analytics and observability" width="820"></picture>

<sub>Nine analytics subcommands, anomaly detection, SIEM-ready JSON · <code>./demo/analytics/01-overview.sh</code> · <a href="https://systemprompt.io/features/analytics-and-observability">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-agents.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-agents.svg"><img src="demo/recording/svg/output/dark/cap-agents.svg" alt="Closed-loop agents" width="820"></picture>

<sub>Agents query their own error rate, cost, and latency via MCP tools and adjust · <code>./demo/agents/04-agent-tracing.sh</code> · <a href="https://systemprompt.io/features/closed-loop-agents">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/cap-compliance.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/cap-compliance.svg"><img src="demo/recording/svg/output/dark/cap-compliance.svg" alt="Compliance" width="820"></picture>

<sub>Tiered retention, 10 identity lifecycle events, SOC 2 / ISO 27001 / HIPAA / OWASP Agentic Top 10 · <code>./demo/users/03-session-management.sh</code> · <a href="https://systemprompt.io/features/compliance">Feature</a></sub>

---

**Integrations** — any provider, Claude Desktop, web publisher, extensions.

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-any-agent.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-any-agent.svg"><img src="demo/recording/svg/output/dark/int-any-agent.svg" alt="Any AI agent" width="820"></picture>

<sub>Anthropic, OpenAI, Gemini swap at the profile level · cost attribution in integer microdollars · <code>./demo/agents/01-list-agents.sh</code> · <a href="https://systemprompt.io/features/any-ai-agent">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-bridge.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-bridge.svg"><img src="demo/recording/svg/output/dark/int-bridge.svg" alt="Claude Desktop & Bridge" width="820"></picture>

<sub>Skills persist across sessions via OAuth2 · <code>./demo/skills/01-skill-lifecycle.sh</code> · <a href="https://systemprompt.io/features/bridge">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-web-publisher.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-web-publisher.svg"><img src="demo/recording/svg/output/dark/int-web-publisher.svg" alt="Web server & publisher" width="820"></picture>

<sub>Same binary serves your website, blog, and docs · systemprompt.io runs on this binary · <code>./demo/web/01-web-config.sh</code> · <a href="https://systemprompt.io/features/web-publisher">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-extensions.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-extensions.svg"><img src="demo/recording/svg/output/dark/int-extensions.svg" alt="Extensible architecture" width="820"></picture>

<sub>Your code compiles into your binary via the <code>Extension</code> trait · no runtime reflection · <code>./demo/skills/04-plugin-management.sh</code> · <a href="https://systemprompt.io/features/extensible-architecture">Feature</a></sub>

<picture><source media="(prefers-color-scheme: dark)" srcset="demo/recording/svg/output/dark/int-benchmark.svg"><source media="(prefers-color-scheme: light)" srcset="demo/recording/svg/output/light/int-benchmark.svg"><img src="demo/recording/svg/output/dark/int-benchmark.svg" alt="Governance benchmark" width="820"></picture>

<sub>3,308 req/s burst, p99 22.7 ms · <code>just benchmark</code></sub>

</details>

<details>
<summary><strong>Claude for Work, on your infrastructure</strong></summary>

<br>

Claude for Work ships with extension points for inference, identity, and audit. Point them at this binary and every prompt, tool call, and cost line lands in a Postgres row you own.

```
  Managed Device                 Enterprise Gateway              Upstream Inference
  (Bridge via MDM)               (this binary, your VPC)         (pluggable)
  ───────────────── ──────────▶  ─────────────────────  ──────▶  ─────────────────
  Credential helper              /v1/messages                    Anthropic direct
  Managed MCP list               Governance pipeline             Bedrock / Vertex
  Signed plugins                 Audit to Postgres               OpenAI / Groq
                                                                 On-prem vLLM / Qwen
                                                                 Air-gap capable
```

The same governance pipeline described above enforces scope, secrets, policy, and quota before a byte leaves your network, in-process against a cached entitlement table: p99 **22.7 ms**, <1% of AI response time.

### How it compares

| Dimension | Claude Enterprise | Cloud Custom | + systemprompt.io |
|---|---|---|---|
| **Data residency** | Anthropic infra | Cloud region | Your datacenter or air-gap |
| **Audit trail** | Anthropic-held | OTLP only | Prompt → tool → MCP → cost in your Postgres |
| **User revocation** | SSO / seat removal | Cloud IAM | IDP disable; next TTL fails closed |
| **Inference provider** | Anthropic only | Bedrock / Vertex (Claude) | Any `/v1/messages`, per-call routing |
| **MCP allowlist** | Anthropic-curated | Device-local config | One registry, per-principal policy |
| **Plugin catalogue** | Anthropic-hosted | Files on disk | Signed, scoped, versioned distribution |

Manual install works end-to-end today; signed installers and MDM packages land in a later release. Full walkthrough: [docs/bridge-install.md](docs/bridge-install.md).

</details>

<details>
<summary><strong>Route any model anywhere — the `/v1/messages` gateway</strong></summary>

<br>

`POST /v1/messages` at the Anthropic wire format. Every inference request flows through the same governance pipeline as every tool call. A route maps a requested model pattern to a provider you declared:

```yaml
gateway:
  enabled: true
  default_provider: anthropic
  routes:
    - model_pattern: "claude-*"
      provider: anthropic
    - model_pattern: "MiniMax-*"
      provider: minimax
```

Routes evaluate in order; first match wins. Anthropic is a transparent byte proxy; OpenAI-compatible providers get full request/response/SSE conversion. Provider declarations, CLI route configuration, route access control, and the extensible provider registry: [docs/gateway-routes.md](docs/gateway-routes.md).

</details>

<details>
<summary><strong>Prerequisites</strong></summary>

<br>

| Requirement | Purpose | Install |
|---|---|---|
| **Docker** | PostgreSQL runs in a container; `just setup-local` starts it | [docker.com](https://docs.docker.com/get-docker/) |
| **Rust 1.75+** | Compiles the workspace binary | [rustup.rs](https://rustup.rs/) |
| **`just`** | Task runner | [just.systems](https://just.systems/) |
| **`jq`, `yq`** | JSON and YAML processing in the scripts | `brew install jq yq` / `apt install jq yq` |
| **AI API keys** | At least one of Anthropic, OpenAI, or Gemini; the first key you supply becomes the default provider | Provider dashboards |
| **Ports 8080 + 5432** | HTTP + PostgreSQL | Free on localhost |

</details>

---

## License

**This template** is [MIT](LICENSE). Fork it, modify it, use it however you like.

**[systemprompt-core](https://github.com/systempromptio/systemprompt-core)** is [BSL-1.1](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE): free for evaluation, testing, and non-production use. Production use requires a commercial license. Each version converts to Apache 2.0 four years after publication. Licensing enquiries: [ed@systemprompt.io](mailto:ed@systemprompt.io).

---

<div align="center">

[![systemprompt.io](https://img.shields.io/badge/systemprompt.io-2b6cb0?style=for-the-badge)](https://systemprompt.io) &nbsp; [![Core](https://img.shields.io/badge/systemprompt--core-2b6cb0?style=for-the-badge)](https://github.com/systempromptio/systemprompt-core) &nbsp; [![Documentation](https://img.shields.io/badge/documentation-16a34a?style=for-the-badge)](https://systemprompt.io/documentation/) &nbsp; [![Guides](https://img.shields.io/badge/guides-f97316?style=for-the-badge)](https://systemprompt.io/guides) &nbsp; [![Discord](https://img.shields.io/badge/discord-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/wkAbSuPWpr)

<sub>You can rent your AI control plane, or you can compile it. Clone, build, run the 43 demos. Then decide.</sub>

</div>
