<div align="center">

<img src="https://foodles.com/files/images/logo-dark.svg" alt="Foodles — Smart Corporate Catering" width="400">

**Production AI agent mesh in 3 commands.**

[![Built on systemprompt-core](https://img.shields.io/badge/built%20on-systemprompt--core-blue)](https://github.com/systempromptio/systemprompt-core)
[![License: BSL-1.1](https://img.shields.io/badge/License-BSL--1.1-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-18+-336791.svg)](https://www.postgresql.org/)
[![MCP](https://img.shields.io/badge/MCP-compatible-purple.svg)](https://modelcontextprotocol.io/)

[Documentation](https://foodles.com/documentation) · [Plugin Marketplace](https://github.com/systempromptio/systemprompt-enterprise-demo-marketplace) · [Issues](https://github.com/foodles/enterprise-demo/issues)

</div>

---

A production-grade AI agent infrastructure that compiles into a single Rust binary with auth, memory, MCP hosting, and observability built in. One dependency: PostgreSQL.

Built on the [systemprompt](https://foodles.com) platform for Foodles. You own the code. You extend it. You deploy it anywhere.

> **This is a library, not a framework.** No vendor lock-in. Self-hosted with PostgreSQL.

---

## Quick Start

### Prerequisites

- **Rust 1.75+**: [rustup.rs](https://rustup.rs/)
- **just**: command runner (`cargo install just`)
- **Docker**: for local PostgreSQL ([docker.com](https://www.docker.com/))

### 1. Create Your Project

```bash
gh repo create my-agent-platform --template foodles/enterprise-demo --clone
cd my-agent-platform
```

> **Just want the plugin?** Install the [marketplace plugin](https://github.com/systempromptio/systemprompt-enterprise-demo-marketplace) directly into Claude Code or Cowork — no build required. Get your `SYSTEMPROMPT_URL` and `SYSTEMPROMPT_TOKEN` from the admin dashboard install widget.

### 2. Build

```bash
just build
```

### 3. Login & Setup

```bash
just login      # Authenticate with foodles.com cloud
just tenant     # Create your tenant
```

### 4. Start

```bash
just start
```

Visit **http://localhost:8080**

---

## What You'll See

When you open the homepage, you'll see the enterprise demo dashboard:

```
┌─────────────────────────────────────────────────────────────────┐
│  Enterprise AI Platform Demo                                      │
│                                                                  │
│  Agents that regenerate, learn and orchestrate agents.           │
│  Run by superagents like Claude Code. Directed by you.           │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  systemprompt online                                      │   │
│  │  ├── MCP Services: 3 running                              │   │
│  │  └── Active Agents: 1 ready                               │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  [Level 1: Chat] [Level 2: Tools] [Level 3: Agents] [Level 4]   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

Each level card shows **runnable commands** you can copy and execute to experience that level of AI capability.

---

## The AI Evolution

The homepage demonstrates the four levels of AI capability. This demo takes you from Level 1 to Level 4:

### Level 1: Chat / The Copy-Paste Era

Single-turn conversations. You ask, AI answers, you copy and paste.

```bash
systemprompt admin agents message welcome -m "Hello!" --blocking
```

### Level 2: Tools / The Function-Calling Era

AI invokes tools. It reads data, calls APIs, writes files.

```bash
systemprompt plugins mcp tools
systemprompt plugins mcp call content-manager create_blog_post -a '{"skill_id": "announcement_writing", "slug": "welcome-post", "description": "AI tools demo", "keywords": ["mcp"], "instructions": "Write a welcome announcement."}'
```

### Level 3: Agents / The Autonomous Era

Define the goal, agents find the path. Multi-step reasoning with tool use.

```bash
systemprompt admin agents message welcome -m "Create an announcement blog" --blocking --timeout 120
systemprompt infra jobs run publish_pipeline
```

### Level 4: Mesh / The Orchestration Era

Superagents coordinate specialized agents. Load a playbook in Claude Code and let the mesh deliver.

```bash
systemprompt core playbooks show content_blog
# Then in Claude Code: "Run the content_blog playbook and publish the results"
```

**Level 4 is the endgame**: agents that regenerate, learn from performance metrics, and orchestrate other agents.

---

## What's Included

See the full documentation at **[foodles.com/documentation](https://foodles.com/documentation)** for:
- [Extensions](https://foodles.com/documentation/extensions/): Web publishing, memory systems, MCP servers
- [Services](https://foodles.com/documentation/services/): Agents, AI providers, authentication, analytics
- [Configuration](https://foodles.com/documentation/config/): Profiles, secrets, tenants, deployment
- [CLI Reference](https://foodles.com/documentation/reference/cli/): Complete command reference

---

## Playbooks

Playbooks are **machine-readable instruction guides** that eliminate AI hallucination. Instead of letting agents guess CLI syntax (and waste tokens on invented flags), playbooks provide pre-tested, deterministic commands for every operation.

**The problem they solve:** LLMs frequently invent non-existent CLI commands. Playbooks encode verified commands in JSON format, ensuring agents execute what actually works.

```bash
# List all playbooks
systemprompt core playbooks list

# Start here, the master index
systemprompt core playbooks show guide_start

# Show any playbook
systemprompt core playbooks show <playbook_id>
```

### Playbook Categories

| Category | Prefix | Purpose |
|----------|--------|---------|
| **Guide** | `guide_*` | Onboarding, meta-documentation, entry points |
| **CLI** | `cli_*` | All command-line operations |
| **Build** | `build_*` | Development standards and checklists |
| **Content** | `content_*` | Content creation workflows |

### Essential Playbooks

| Playbook | Description |
|----------|-------------|
| `guide_start` | Master index, read this first |
| `guide_coding-standards` | Rust patterns and code standards |
| `cli_agents` | Agent management commands |
| `cli_deploy` | Deployment procedures |
| `build_extension-checklist` | Building Rust extensions |
| `content_blog` | Blog publishing workflow |

### Self-Repair Protocol

When a command fails, agents fix the playbook rather than retry with guesses. This creates a feedback loop where instructions improve over time.

Learn more: **[foodles.com/playbooks](https://foodles.com/playbooks)**

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  Your Project (this demo)                                        │
│                                                                  │
│  extensions/              <- Your Rust code                       │
│  ├── web/                   Web publishing, themes, SEO          │
│  ├── soul/                  Long-term memory system              │
│  ├── mcp/                   MCP servers (3 included)             │
│  └── cli/                   CLI extensions                       │
│                                                                  │
│  services/                <- Your configuration (YAML/Markdown)   │
│  ├── agents/                Agent definitions                    │
│  ├── skills/                Skill configurations                 │
│  ├── playbook/              96 operational playbooks             │
│  └── web/                   Themes, templates, homepage          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  systemprompt-core (Cargo dependency)                            │
│                                                                  │
│  Provides:                                                       │
│  ├── API server + CLI                                            │
│  ├── Agent runtime + A2A protocol                                │
│  ├── MCP server hosting                                          │
│  ├── Auth (OAuth2 + WebAuthn)                                    │
│  ├── Memory systems                                              │
│  └── Observability + logging                                     │
└─────────────────────────────────────────────────────────────────┘
```

**Key rules:**
- Rust code -> `extensions/`
- Configuration -> `services/`
- Core is a **Cargo dependency** (from crates.io)

---

## Project Structure

```
enterprise-demo/
├── extensions/              # Your Rust code
│   ├── web/                 # Web publishing extension
│   ├── soul/                # Memory system extension
│   ├── mcp/                 # MCP servers
│   │   ├── systemprompt/    #   CLI execution (admin-only)
│   │   ├── soul/            #   Memory operations
│   │   └── content-manager/ #   Content creation
│   └── cli/                 # CLI extensions
│
├── services/                # Configuration (YAML/Markdown only)
│   ├── agents/              # Agent definitions
│   ├── skills/              # Skill configurations
│   ├── mcp/                 # MCP server configs
│   ├── playbook/            # 96 operational playbooks
│   └── web/                 # Theme, templates, homepage config
│
├── Cargo.toml               # Workspace manifest (systemprompt-core via Cargo)
└── justfile                 # Development commands
```

---

## Commands

| Command | Description |
|---------|-------------|
| `just build` | Build the project |
| `just login` | Authenticate with cloud |
| `just tenant` | Create tenant (database, profile, migrations) |
| `just start` | Start all services |
| `just deploy` | Build and deploy to cloud |
| `just migrate` | Run database migrations |

---

## Dev Plugin for Claude Code

The `systemprompt-dev` plugin provides coding standards, extension guides, and 6 specialized agents for development with Claude Code.

### Generate the plugin

```bash
systemprompt core plugins generate --id systemprompt-dev
```

This outputs skills and agent prompts to `.generated/systemprompt-dev/` which Claude Code can read as context.

### Run Claude Code with dev skills

```bash
claude --prompt "Read .generated/systemprompt-dev/skills/dev-rust-standards/SKILL.md and enforce these standards on extensions/"
```

Or load all dev skills at once:

```bash
claude --prompt "$(cat .generated/systemprompt-dev/skills/*/SKILL.md)"
```

### Available skills

| Skill | Description |
|-------|-------------|
| `dev-rust-standards` | Rust coding, clippy, testing, and architecture standards |
| `dev-frontend-standards` | JavaScript, CSS, accessibility, event hub, build pipeline |
| `dev-architecture-standards` | Layer architecture, module boundaries, extension registration, plugin structure |
| `dev-ext-data-providers` | PageDataProvider, ContentDataProvider, FrontmatterProcessor traits |
| `dev-ext-rendering` | ComponentRenderer, TemplateProvider, TemplateDataExtender, PagePrerenderer traits |
| `dev-ext-infrastructure` | Jobs, Schemas, Router, Assets, Storage Paths, Config Validation |
| `dev-ext-feeds` | RssFeedProvider, SitemapProvider traits |
| `dev-ext-providers` | LlmProvider, ToolProvider, MCP server integration |
| `dev-ext-hooks` | Hook catalog, lifecycle events, hook scripts |

### Available agents

| Agent | Role |
|-------|------|
| `standards` | Scans for architecture, Rust, and frontend violations |
| `architect` | Designs implementations using extension traits |
| `overseer` | Orchestrates other agents as tech lead |
| `quality_gate` | Verifies all standards pass after implementation |
| `rust_impl` | Writes and fixes Rust code |
| `frontend_impl` | Writes and fixes JS/CSS/HTML |

---

## What's Next?

### Customize & Develop

Build agents, skills, and MCP servers tailored to your needs.

- Add new agents in `services/agents/`
- Define custom skills in `services/skills/`
- Build MCP servers in `extensions/mcp/`
- See: `systemprompt core playbooks show build_getting-started`

### Deploy Securely

Ship to production with OAuth, HTTPS, and full audit trails.

```bash
just login
just tenant create --region iad
just profile create production
just deploy --profile production
```

See: `systemprompt core playbooks show cli_deploy`

### Automate & Schedule

Run jobs on schedules and build agentic workflows.

- Cron-based job scheduling
- Event-driven triggers
- Agentic workflow pipelines

See: `systemprompt core playbooks show cli_jobs`

---

## Built on systemprompt-core

This demo extends [systemprompt-core](https://github.com/systempromptio/systemprompt-core), a production-ready AI agent orchestration library that provides:

- **Complete runtime**: API server, agent runtime, MCP hosting
- **Authentication**: OAuth2 for APIs, WebAuthn for passwordless login
- **Memory systems**: Long-term, short-term, and working memory
- **A2A protocol**: Agent-to-agent communication and orchestration
- **Observability**: Full request logging and audit trails

Works with Claude Code, Claude Desktop, ChatGPT, and any MCP-compatible tool.

---

## License

BSL-1.1, see [LICENSE](LICENSE)

Depends on [systemprompt-core](https://github.com/systempromptio/systemprompt-core) (FSL-1.1-ALv2).

---

<div align="center">

**[Documentation](https://foodles.com/documentation)** · **[systemprompt-core](https://github.com/systempromptio/systemprompt-core)** · **[Issues](https://github.com/foodles/enterprise-demo/issues)**

Questions or issues? Contact Foodles for support.

</div>
