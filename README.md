<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://systemprompt.io/files/images/logo.svg">
  <source media="(prefers-color-scheme: light)" srcset="https://systemprompt.io/files/images/logo-dark.svg">
  <img src="https://systemprompt.io/files/images/logo-dark.svg" alt="systemprompt.io" width="400">
</picture>

### Evaluate systemprompt.io. Run it yourself.

[![Built on systemprompt-core](https://img.shields.io/badge/built%20on-systemprompt--core-blue)](https://github.com/systempromptio/systemprompt-core)
[![Template License: MIT](https://img.shields.io/badge/Template-MIT-green.svg)](LICENSE)
[![Core License: BSL-1.1](https://img.shields.io/badge/Core-BSL--1.1-blue.svg)](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-18+-336791.svg)](https://www.postgresql.org/)

[Website](https://systemprompt.io) · [About](https://systemprompt.io/about) · [Documentation](https://systemprompt.io/documentation/) · [Discord](https://discord.gg/wkAbSuPWpr)

</div>

---

This is the **local evaluation template** for [systemprompt.io](https://systemprompt.io), the touchpoint between your AI and everything it does.

Clone it, build it, run it on your own machine. Full governance pipeline, admin dashboard, MCP servers, skill marketplace, and demo scripts. Bring your own AI provider key.

> **Evaluation only.** This template exists so you can run the stack locally and see what it does. Production deployment is a licensed offering — contact [ed@systemprompt.io](mailto:ed@systemprompt.io) for a commercial licence.

The template code is MIT licensed. The underlying [systemprompt-core](https://github.com/systempromptio/systemprompt-core) library is BSL-1.1: free for evaluation, testing, and non-production use. Production use requires a [commercial license](mailto:ed@systemprompt.io).

---

## Quick Start

### Prerequisites

- **Rust 1.75+**: [rustup.rs](https://rustup.rs/)
- **just**: command runner (`cargo install just`)
- **Docker**: for local PostgreSQL ([docker.com](https://www.docker.com/))
- **`jq`** and **`yq`**: used by `just` recipes to read profile/secrets files
- **An AI provider API key**: at least one of Anthropic, OpenAI, or Gemini. The marketplace MCP server will not start without a key — you can add more later.
- **Free ports**: `8080` (HTTP) and `5432` (Postgres) by default. If either is in use, pass overrides to `setup-local` — see [Running multiple clones](#running-multiple-clones).

### 1. Create Your Project

```bash
gh repo create my-project --template systempromptio/systemprompt-template --clone
cd my-project
```

### 2. Build

```bash
just build
```

### 3. Set up local profile + Postgres

```bash
just setup-local <anthropic_key> <openai_key> <gemini_key>
```

Pass only the keys you have — leave the others as empty strings (`""`). Run with no arguments to be prompted interactively. This writes `.systemprompt/profiles/local/{profile.yaml,secrets.json}`, brings up Postgres in Docker, and runs the publish pipeline.

### 4. Start

```bash
just start
```

Visit **http://localhost:8080** to see the dashboard, admin panel, and governance pipeline in action.

### Running multiple clones

Each clone gets its own Docker containers and volumes (project name is derived from the clone's absolute path). To run two clones side-by-side, give the second one different ports at setup time:

```bash
just setup-local <anthropic_key> "" "" 8081 5433
```

---

## What You Can Evaluate

### Governance Pipeline

Synchronous four-layer evaluation on every tool call: scope check, secret scan, blocklist, rate limit. Sub-5ms p50 latency. See it execute in real time.

### Admin Dashboard

Full web dashboard with user management, skill editing, plugin distribution, analytics, and agent configuration. Runs from the same binary.

### MCP Servers

Pre-configured MCP servers for CLI execution and skill management. Connect Claude Code, Claude Desktop, or any MCP-compatible client.

### Skill Marketplace

Browse, install, create, and fork skills. Plugin bundles with governed distribution by role and department.

### Demo Scripts

The `demo/` directory contains 40+ executable scripts organised into categories, each exercising a different surface of the platform. Scripts within a category are numbered — run them in order.

```bash
./demo/00-preflight.sh          # Environment and connectivity checks
./demo/01-seed-data.sh          # Seed analytics/logs/traces
./demo/governance/01-happy-path.sh
./demo/agents/01-list-agents.sh
# ... etc
```

| Category | Demonstrates |
|----------|--------------|
| `demo/governance/` | Governance pipeline: approvals, denials, secret breach, rate limits, hooks |
| `demo/agents/` | Agent lifecycle, config, messaging, tracing, registry |
| `demo/mcp/` | MCP servers, access tracking, tool execution |
| `demo/skills/` | Skill lifecycle, content, files, plugins, contexts |
| `demo/infrastructure/` | Services, database, jobs, logs, config |
| `demo/analytics/` | Overview, agents, cost, requests, sessions, traffic, tools |
| `demo/users/` | User CRUD, roles, sessions, IP bans |
| `demo/web/` | Web config, sitemap validation |
| `demo/performance/` | Request tracing, load testing |

See [`demo/README.md`](demo/README.md) for the full catalogue and [`demo/AGENTS.md`](demo/AGENTS.md) for the LLM-targeted runbook.

---

## Project Structure

```
your-project/
├── extensions/              # Your Rust code (compile-time extensions)
│   ├── web/                 # Web publishing, themes, SSR
│   └── mcp/                 # MCP server implementations
│
├── services/                # Configuration (YAML/Markdown only)
│   ├── agents/              # Agent definitions with OAuth scopes
│   ├── skills/              # Skill configurations
│   ├── plugins/             # Claude Code plugins
│   ├── ai/                  # Provider configs (Anthropic, OpenAI, Gemini)
│   └── web/                 # Theme, branding, navigation
│
├── demo/                    # Executable demo scripts
├── storage/files/           # Static assets (CSS, JS, images)
├── Cargo.toml               # Workspace manifest
├── justfile                 # Development commands
└── CLAUDE.md                # AI assistant instructions
```

---

## Commands

| Command | Description |
|---------|-------------|
| `just build` | Build the project |
| `just setup-local [keys] [http_port] [pg_port]` | Create local profile, start Docker Postgres, run publish pipeline |
| `just start` | Start all services |
| `just publish` | Compile templates, bundle CSS/JS, copy assets |
| `just db-up` / `just db-down` / `just db-reset` | Manage the local Postgres container |

---

## CLI

```bash
systemprompt --help              # Top-level help
systemprompt core skills list    # List skills
systemprompt admin agents list   # List agents
systemprompt infra logs view     # View logs
systemprompt analytics overview  # Analytics dashboard
```

---

## How systemprompt.io Works

systemprompt.io is a single compiled Rust binary that sits between your AI agents and everything they touch. Every tool call authenticated, authorised, rate-limited, logged, and costed.

- One language (Rust), one database (PostgreSQL), one binary (~50MB)
- Self-hosted, air-gap capable, provider-agnostic
- MCP, A2A, OAuth2/OIDC, WebAuthn
- Sub-5ms governance overhead per tool call

Read the [full about page](https://systemprompt.io/about) for the story behind the code.

---

## License & Production Use

**This template** is MIT licensed. Fork it, modify it, use it however you like — **for local evaluation**.

**systemprompt-core** (the underlying library) is [BSL-1.1](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE). Free for evaluation, testing, and non-production use. **Production use requires a commercial licence.** Converts to Apache 2.0 four years after each version is published.

For licensing enquiries and production deployment: [ed@systemprompt.io](mailto:ed@systemprompt.io)

---

<div align="center">

**[systemprompt.io](https://systemprompt.io)** · **[systemprompt-core](https://github.com/systempromptio/systemprompt-core)** · **[About](https://systemprompt.io/about)** · **[Discord](https://discord.gg/wkAbSuPWpr)**

</div>
