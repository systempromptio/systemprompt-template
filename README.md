<div align="center">

# systemprompt-template

**Production-ready AI infrastructure in 5 minutes.**

[![Built on systemprompt-core](https://img.shields.io/badge/built%20on-systemprompt--core-blue)](https://github.com/systempromptio/systemprompt-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-18+-336791.svg)](https://www.postgresql.org/)
[![MCP](https://img.shields.io/badge/MCP-compatible-purple.svg)](https://modelcontextprotocol.io/)
[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2.svg)](https://discord.gg/wkAbSuPWpr)

[Getting Started](#quick-start) · [Documentation](https://systemprompt.io/documentation) · [Discord](https://discord.gg/wkAbSuPWpr) · [Playbooks](#playbooks)

**Questions or issues?** Join us on [Discord](https://discord.gg/wkAbSuPWpr) for help.

</div>

---

A starter template for [systemprompt.io](https://systemprompt.io) — the production infrastructure for AI agents. This template provides pre-built extensions, operational playbooks, and configured agents so you can ship faster.

## What's Included

| Component | Description |
|-----------|-------------|
| **2 Extensions** | Web publishing + Soul memory system |
| **3 MCP Servers** | CLI execution, memory ops, content management |
| **13 Skills** | Blog writing, technical docs, general assistance, and more |
| **99 Playbooks** | Machine-readable guides for every operation |
| **Pre-configured Agent** | Ready-to-use AI assistant |

---

## Quick Start

### Prerequisites

- **Rust 1.75+** — [rustup.rs](https://rustup.rs/)
- **Docker** — for local PostgreSQL

### 1. Create Your Project

```bash
gh repo create my-project --template systempromptio/systemprompt-template --clone --private
cd my-project
```

### 2. Build & Run

```bash
just build
just db-up
just login
just tenant
just profile
just start
```

### 3. Open

Visit **http://localhost:8080**

---

## Project Structure

```
├── extensions/          # Your Rust code
│   ├── web/             # Web publishing extension
│   ├── soul/            # Memory system extension
│   └── mcp/             # MCP servers (systemprompt, soul, content-manager)
│
├── services/            # Configuration (YAML/Markdown only)
│   ├── agents/          # Agent definitions
│   ├── skills/          # Skill configurations
│   ├── mcp/             # MCP server configs
│   ├── playbook/        # 99 operational playbooks
│   └── web/             # Theme and branding
│
├── Cargo.toml           # Workspace manifest
└── justfile             # Development commands
```

**Key rules:**
- Rust code → `extensions/`
- Configuration → `services/`
- Core is read-only (git submodule)

---

## Playbooks

Every operation has a playbook. Don't guess commands — read the playbook.

```bash
# List all playbooks
systemprompt core playbooks list

# Start here
systemprompt core playbooks show guide_start

# Read any playbook
systemprompt core playbooks show <playbook_id>
```

| Prefix | Purpose |
|--------|---------|
| `guide_*` | Entry points and onboarding |
| `cli_*` | CLI command references |
| `build_*` | Development standards |
| `content_*` | Content creation workflows |

---

## Extensions

### Web Extension

Blog publishing, documentation, navigation, and content management with SEO.

### Soul Extension

Long-term memory and context retention with Discord integration.

### MCP Servers

| Server | Purpose |
|--------|---------|
| `systemprompt` | Execute CLI commands (admin-only) |
| `soul` | Memory operations |
| `content-manager` | Content creation and management |

---

## Commands

| Command | Description |
|---------|-------------|
| `just build` | Build the project |
| `just start` | Start all services |
| `just migrate` | Run database migrations |
| `just login` | Authenticate with cloud |
| `just deploy` | Build and deploy to cloud |
| `just db-up` | Start local PostgreSQL |
| `just db-down` | Stop local PostgreSQL |

---

## Cloud Deployment

```bash
just login
just tenant create --region iad
just profile create production
just deploy --profile production
```

See playbook: `systemprompt core playbooks show cli_deploy`

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Your Project (this template)                           │
│  ├── extensions/       # Your Rust code                 │
│  └── services/         # Your YAML configuration        │
└─────────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│  core/ (git submodule - READ ONLY)                      │
│  └── systemprompt-core                                  │
│      ├── API server + CLI                               │
│      ├── Agent runtime + A2A protocol                   │
│      ├── MCP server hosting                             │
│      └── Auth (OAuth2 + WebAuthn)                       │
└─────────────────────────────────────────────────────────┘
```

---

## Built on systemprompt-core

This template extends [systemprompt-core](https://github.com/systempromptio/systemprompt-core) which provides:

- Complete runtime (API + agents + MCP)
- Authentication (OAuth2, WebAuthn)
- Database layer (PostgreSQL)
- A2A agent-to-agent protocol

---

## License

MIT — see [LICENSE](LICENSE)

Depends on [systemprompt-core](https://github.com/systempromptio/systemprompt-core) (FSL-1.1-ALv2).

---

<div align="center">

**[Documentation](https://systemprompt.io/documentation)** · **[Discord](https://discord.gg/wkAbSuPWpr)** · **[systemprompt-core](https://github.com/systempromptio/systemprompt-core)** · **[Issues](https://github.com/systempromptio/systemprompt-template/issues)**

</div>
