# systemprompt-template

**Production-ready AI infrastructure in 5 minutes.**

[![Built on systemprompt-core](https://img.shields.io/badge/built%20on-systemprompt--core-blue)](https://github.com/systempromptio/systemprompt-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

## What You Get

This template extends [systemprompt-core](https://github.com/systempromptio/systemprompt-core) with production-ready patterns:

## Quick Start

### Prerequisites

- **Rust 1.75+** - https://rustup.rs/
- **Docker** (for local PostgreSQL) OR systemprompt.io Cloud account

### 1. Create Your Project

**Option A: GitHub CLI (Recommended)**

```bash
gh repo create my-project --template systempromptio/systemprompt-template --clone --private
cd my-project
```

**Option B: Manual Clone**

```bash
git clone https://github.com/systempromptio/systemprompt-template.git my-project
cd my-project
```

### 2. Build

```bash
just build
```

### 3. Setup CLI Access

The `systemprompt` binary is built to `target/debug/` (or `target/release/`). Choose one:

**Option A: Use `just` wrapper (Recommended)**
```bash
# All commands work via just:
just cli infra services status
just cli core playbooks list
```

**Option B: Add to PATH**
```bash
# Add to your shell config (~/.bashrc or ~/.zshrc):
export PATH="$PATH:/var/www/html/systemprompt-template/target/debug"

# Then reload:
source ~/.bashrc  # or source ~/.zshrc

# Now use directly:
systemprompt infra services status
```

**Option C: Create alias**
```bash
alias systemprompt='/var/www/html/systemprompt-template/target/debug/systemprompt'
```

### 4. Setup Database

```bash
# Start local PostgreSQL
docker run -d --name systemprompt-db \
  -e POSTGRES_DB=systemprompt \
  -e POSTGRES_USER=systemprompt \
  -e POSTGRES_PASSWORD=systemprompt \
  -p 5432:5432 \
  postgres:18-alpine

# Login to systemprompt.io (free, enables profile management)
just login

# Setup local tenant and profile (follow interactive prompts)
just tenant
just profile
just start
```

**Or for Cloud (managed PostgreSQL):**

```bash
just login
just tenant create --region iad
just profile create production
just migrate
just start
```

### 5. Verify

Visit http://localhost:8080

## The Playbook System

**Don't guess commands. Read the playbook.**

68 playbooks guide every operation - from CLI commands to building extensions. They're machine-readable documentation that eliminates guesswork.

### Access Playbooks

```bash
# List all playbooks
systemprompt core playbooks list

# Start here - always
systemprompt core playbooks show guide_start

# Read any playbook
systemprompt core playbooks show <playbook_id>
```

### Playbook Categories

| Prefix | Purpose | Examples |
|--------|---------|----------|
| `guide_*` | Start here - onboarding and master indexes | `guide_start`, `guide_coding-standards` |
| `cli_*` | CLI command references | `cli_agents`, `cli_cloud`, `cli_deploy` |
| `build_*` | Development and extension building | `build_extension-checklist`, `build_mcp-checklist` |
| `content_*` | Content creation workflows | `content_blog` |

### The Workflow

1. **Find the playbook**: `systemprompt core playbooks list`
2. **Read it**: `systemprompt core playbooks show <id>`
3. **Follow exactly** - every command is tested
4. **If wrong, fix it** - playbooks self-repair

## Project Structure

```
systemprompt-template/
├── extensions/              # YOUR RUST CODE
│   ├── web/                 # Web publishing extension
│   ├── soul/                # Memory system extension
│   └── mcp/                 # MCP servers
│       ├── systemprompt/    # Main MCP server
│       ├── soul/            # Soul memory MCP
│       └── content-manager/ # Content operations MCP
├── services/                # CONFIG ONLY (YAML/Markdown)
│   ├── agents/              # Agent configurations
│   ├── skills/              # Skill definitions
│   ├── mcp/                 # MCP server configs
│   ├── playbook/            # Operational playbooks
│   ├── content/             # Markdown content
│   └── web/                 # Theme and branding
├── Cargo.toml               # Workspace - depends on systemprompt crate
└── justfile                 # Development commands
```

**Key rules:**
- All Rust code goes in `extensions/`
- All config (YAML/Markdown) goes in `services/`
- Core functionality comes from `systemprompt` crate (Cargo dependency)

## Adding Your Own

### Add an Agent

1. Create `services/agents/my-agent.yaml`
2. Add to `services/config/config.yaml` includes
3. Restart services

### Add a Skill

1. Create directory `services/skills/my-skill/`
2. Add `config.yaml` with skill definition
3. Sync: `systemprompt core skills sync --direction to-db -y`

### Build an Extension

See playbook: `systemprompt core playbooks show build_extension-checklist`

### Build an MCP Server

See playbook: `systemprompt core playbooks show build_mcp-checklist`

## Built on systemprompt-core

This template extends [systemprompt-core](https://github.com/systempromptio/systemprompt-core) - production infrastructure for AI agents.

**Core provides:**
- Complete runtime (API + agents + MCP servers)
- Authentication (OAuth2, WebAuthn)
- Database layer (PostgreSQL)
- MCP protocol implementation
- A2A agent-to-agent communication

**Template adds:**
- Production-ready extensions (Web, Soul)
- Pre-configured agents and skills
- 68 operational playbooks
- Theme and branding system
- Content management

## Cloud Deployment

```bash
# Build and deploy
just deploy --profile production

# Or step by step
cargo build --release -p systemprompt-cli
systemprompt cloud deploy --profile production
```

See playbook: `systemprompt core playbooks show cli_deploy`

## Development Commands

| Command | Description |
|---------|-------------|
| `just build` | Build (auto-detects offline mode) |
| `just start` | Start all services |
| `just migrate` | Run database migrations |
| `just login` | Authenticate with cloud |
| `just deploy` | Build and deploy to cloud |

## Updating Core

Update the systemprompt crate:

```bash
cargo update systemprompt
just build
```

## License

This template is MIT licensed.

It depends on [systemprompt-core](https://github.com/systempromptio/systemprompt-core) which is licensed under FSL-1.1-ALv2.

## Links

- [systemprompt-core](https://github.com/systempromptio/systemprompt-core) - The foundation
- [Documentation](https://systemprompt.io/documentation) - Full docs
- [Issues](https://github.com/systempromptio/systemprompt-template/issues) - Report bugs
