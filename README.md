<div align="center">

# systemprompt-template

**Production AI agent mesh in 3 commands.**

[![Built on systemprompt-core](https://img.shields.io/badge/built%20on-systemprompt--core-blue)](https://github.com/systempromptio/systemprompt-core)
[![License: BSL-1.1](https://img.shields.io/badge/License-BSL--1.1-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-18+-336791.svg)](https://www.postgresql.org/)
[![MCP](https://img.shields.io/badge/MCP-compatible-purple.svg)](https://modelcontextprotocol.io/)

[Documentation](https://systemprompt.io/documentation) · [Plugin Marketplace](https://github.com/systempromptio/systemprompt-enterprise-demo-marketplace) · [Issues](https://github.com/systempromptio/systemprompt-template/issues)

</div>

---

The canonical template for building enterprise AI agent platforms on [systemprompt-core](https://github.com/systempromptio/systemprompt-core). Clone it, rebrand it, ship it.

Compiles into a single Rust binary with auth, memory, MCP hosting, and observability built in. One dependency: PostgreSQL.

> **This is a library, not a framework.** No vendor lock-in. Self-hosted with PostgreSQL.

---

## Quick Start

### Prerequisites

- **Rust 1.75+**: [rustup.rs](https://rustup.rs/)
- **just**: command runner (`cargo install just`)
- **Docker**: for local PostgreSQL ([docker.com](https://www.docker.com/))

### 1. Create Your Project

```bash
gh repo create my-agent-platform --template systempromptio/systemprompt-template --clone
cd my-agent-platform
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

Visit **http://localhost:8080**

---

## Rebranding

This template ships with generic "Enterprise Demo" branding. To create a branded deployment for a client:

1. Read **[BRANDING.md](BRANDING.md)** for the complete checklist
2. Edit `services/web/config/theme.yaml` (name, domain, colors, logos)
3. Update CSS tokens in `storage/files/css/admin/01-tokens-primitives.css` and `storage/files/css/core/variables.css`
4. Replace image assets in `storage/files/images/`
5. Run `just publish` to rebuild

See existing branded deployments for reference:
- [systemprompt-dynapps](https://github.com/systempromptio/systemprompt-dynapps) (purple, Odoo partner)
- [systemprompt-agentic-accounting](https://github.com/systempromptio/systemprompt-agentic-accounting) (teal, AI accounting)
- [systemprompt-knowbe4](https://github.com/systempromptio/systemprompt-knowbe4) (orange, cybersecurity training)

---

## What's Included

### Agents

Three pre-configured agents with RBAC governance:

| Agent | Role | Access Level |
|-------|------|-------------|
| Associate | Revenue operations | Standard |
| Developer | Platform development | Extended |
| Admin | System administration | Full |

### MCP Servers

| Server | Purpose |
|--------|---------|
| `systemprompt` | CLI execution, artifact viewing |
| `marketplace` | Skill management, plugin sync |

### Extensions

| Extension | Purpose |
|-----------|---------|
| `extensions/web/` | Web publishing, themes, SSR, SEO |
| `extensions/mcp/` | MCP server implementations |

### Plugins

| Plugin | Purpose |
|--------|---------|
| `systemprompt-admin` | Platform administration |
| `systemprompt-dev` | Development toolkit (10 skills, 6 agents) |
| `common-skills` | Shared skills (Odoo, brand, content) |
| `sales-skills` | Sales CRM (reports, emails, health) |

---

## Architecture

```
your-project/
├── extensions/              # Your Rust code
│   ├── web/                 # Web publishing extension
│   └── mcp/                 # MCP servers
│       ├── systemprompt/    #   CLI execution (admin-only)
│       └── marketplace/     #   Skill management
│
├── .systemprompt/           # Runtime state (profiles, sessions, credentials)
│   └── profiles/            # Deployment profiles (local/production)
│
├── services/                # Configuration (YAML/Markdown only)
│   ├── agents/              # Agent definitions
│   ├── skills/              # Skill configurations
│   ├── plugins/             # Claude Code plugins
│   └── web/                 # Theme, templates, homepage config
│
├── storage/files/           # Static assets
│   ├── css/                 # CSS source (tokens, components)
│   ├── js/                  # JavaScript bundles
│   ├── admin/               # Admin templates (HBS)
│   └── images/              # Logos, favicons
│
├── Cargo.toml               # Workspace manifest
├── justfile                 # Development commands
├── BRANDING.md              # Rebranding checklist
└── CLAUDE.md                # AI assistant instructions
```

**Key rules:**
- Rust code goes in `extensions/`
- Configuration goes in `services/` (YAML/Markdown only)
- CSS goes in `storage/files/css/` (registered in `extensions/web/src/extension.rs`)
- `core/` is read-only (git submodule)
- Run `just publish` after changing templates, CSS, JS, or static files

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
| `just clippy` | Run Rust lints |

---

## CLI

```bash
systemprompt --help              # Top-level help
systemprompt core skills list    # List skills
systemprompt admin agents list   # List agents
systemprompt infra logs view     # View logs
systemprompt analytics overview  # Analytics dashboard
```

See [CLAUDE.md](CLAUDE.md) for full CLI structure.

---

## Demo Suite

The `demo/` directory contains executable scripts that demonstrate platform capabilities:

| Script | What it shows |
|--------|--------------|
| `01-happy-path.sh` | Agent request with governance approval |
| `02-refused-path.sh` | Governance denial and audit trail |
| `03-audit-trail.sh` | Full request tracing |
| `04-governance-happy.sh` | Multi-agent governance flow |
| `05-governance-denied.sh` | Secret access denied |

```bash
cd demo && bash 01-happy-path.sh
```

---

## Built on systemprompt-core

This template extends [systemprompt-core](https://github.com/systempromptio/systemprompt-core), which provides:

- **API server + CLI** with full command discovery
- **Agent runtime** with A2A protocol
- **MCP server hosting** (Model Context Protocol)
- **Authentication** (OAuth2 + WebAuthn)
- **Memory systems** (long-term, short-term, working)
- **Observability** with request logging and audit trails

Works with Claude Code, Claude Desktop, ChatGPT, and any MCP-compatible tool.

---

## License

BSL-1.1, see [LICENSE](LICENSE)

Depends on [systemprompt-core](https://github.com/systempromptio/systemprompt-core) (FSL-1.1-ALv2).

---

<div align="center">

**[systemprompt.io](https://systemprompt.io)** · **[systemprompt-core](https://github.com/systempromptio/systemprompt-core)** · **[Issues](https://github.com/systempromptio/systemprompt-template/issues)**

</div>
