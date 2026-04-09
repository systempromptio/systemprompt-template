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

[Website](https://systemprompt.io) · [About](https://systemprompt.io/about) · [Documentation](https://systemprompt.io/documentation/) · [Live Demo](https://systemprompt.io/features/demo) · [Discord](https://discord.gg/wkAbSuPWpr)

</div>

---

This is the evaluation template for [systemprompt.io](https://systemprompt.io), the touchpoint between your AI and everything it does.

Clone it, build it, run it. Full governance pipeline, admin dashboard, MCP servers, skill marketplace, and demo scripts. No sign-up required.

The template code is MIT licensed. The underlying [systemprompt-core](https://github.com/systempromptio/systemprompt-core) library is BSL-1.1: free for evaluation, testing, and non-production use. Production use requires a [commercial license](mailto:ed@systemprompt.io).

---

## Quick Start

### Prerequisites

- **Rust 1.75+**: [rustup.rs](https://rustup.rs/)
- **just**: command runner (`cargo install just`)
- **Docker**: for local PostgreSQL ([docker.com](https://www.docker.com/))

### 1. Create Your Project

```bash
gh repo create my-project --template systempromptio/systemprompt-template --clone
cd my-project
```

### 2. Build

```bash
just build
```

### 3. Login & Setup

```bash
just login      # Authenticate with systemprompt.io cloud
just tenant     # Create your tenant
```

### 4. Start

```bash
just start
```

Visit **http://localhost:8080** to see the dashboard, admin panel, and governance pipeline in action.

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

The `demo/` directory contains executable scripts demonstrating every aspect of the governance pipeline:

| Script | What it demonstrates |
|--------|---------------------|
| `00-preflight.sh` | Environment and connectivity checks |
| `01-happy-path.sh` | Agent request with governance approval |
| `02-refused-path.sh` | Governance denial and audit trail |
| `03-audit-trail.sh` | Full request tracing |
| `04-governance-happy.sh` | Governance pipeline approval flow |
| `05-governance-denied.sh` | Governance pipeline denial flow |
| `06-governance-secret-breach.sh` | Secret detection and blocking |
| `07-mcp-access-tracking.sh` | MCP server access audit |
| `08-request-tracing.sh` | End-to-end request tracing |
| `09-agent-tracing.sh` | Agent execution tracing |

```bash
cd demo && bash 00-preflight.sh
```

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
| `just start` | Start all services |
| `just publish` | Compile templates, bundle CSS/JS, copy assets |
| `just login` | Authenticate with cloud |
| `just tenant` | Create tenant (database, profile, migrations) |
| `just deploy` | Build and deploy to cloud |

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

## License

**This template** is MIT licensed. Fork it, modify it, use it however you like.

**systemprompt-core** (the underlying library) is [BSL-1.1](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE). Free for evaluation, testing, and non-production use. Production use requires a commercial license. Converts to Apache 2.0 four years after each version is published.

For licensing enquiries: [ed@systemprompt.io](mailto:ed@systemprompt.io)

---

<div align="center">

**[systemprompt.io](https://systemprompt.io)** · **[systemprompt-core](https://github.com/systempromptio/systemprompt-core)** · **[About](https://systemprompt.io/about)** · **[Discord](https://discord.gg/wkAbSuPWpr)**

</div>
