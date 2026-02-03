<div align="center">

# systemprompt-template

**From chat to autonomous agent mesh in 3 commands.**

[![Built on systemprompt-core](https://img.shields.io/badge/built%20on-systemprompt--core-blue)](https://github.com/systempromptio/systemprompt-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-18+-336791.svg)](https://www.postgresql.org/)
[![MCP](https://img.shields.io/badge/MCP-compatible-purple.svg)](https://modelcontextprotocol.io/)
[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2.svg)](https://discord.gg/wkAbSuPWpr)

[Documentation](https://systemprompt.io/documentation) · [Discord](https://discord.gg/wkAbSuPWpr) · [Issues](https://github.com/systempromptio/systemprompt-template/issues)

</div>

---

This template gives you a complete AI agent infrastructure running locally. Start it, open `localhost:8080`, and experience the evolution from simple chat to autonomous agent orchestration — with live, runnable examples at each level.

Built on [systemprompt.io](https://systemprompt.io) — a 50MB production-ready Rust library with authentication, memory, MCP hosting, and observability built in. You own the code. You extend it. You deploy it anywhere.

> **This is a library, not a platform.** You build your own binary by extending the core. No vendor lock-in. Self-hosted with PostgreSQL.

---

## Quick Start

### Prerequisites

- **Rust 1.75+** — [rustup.rs](https://rustup.rs/)
- **Docker** — for local PostgreSQL
- **just** — command runner (`cargo install just`)

### 1. Create Your Project

```bash
gh repo create my-project --template systempromptio/systemprompt-template --clone --private
cd my-project
```

Or download directly:

```bash
git clone --recursive https://github.com/systempromptio/systemprompt-template.git my-project
cd my-project
```

### 2. Build

```bash
just build
```

### 3. Setup & Start

```bash
just db-up      # Start PostgreSQL
just login      # Authenticate with systemprompt.io cloud
just profile    # Create local profile
just start      # Launch services
```

### 4. Open

Visit **http://localhost:8080**

---

## What You'll See

When you open the homepage, you'll see the **"Welcome to the endgame"** dashboard:

```
┌─────────────────────────────────────────────────────────────────┐
│  Welcome to the endgame                                          │
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

The homepage demonstrates the four levels of AI capability. This template takes you from Level 1 to Level 4:

### Level 1: Chat — The Copy-Paste Era

Single-turn conversations. You ask, AI answers, you copy and paste.

```bash
systemprompt admin agents message welcome -m "Hello!" --blocking
```

### Level 2: Tools — The Function-Calling Era

AI invokes tools. It reads data, calls APIs, writes files.

```bash
systemprompt plugins mcp tools
systemprompt plugins mcp call content-manager create_blog_post -a '{"skill_id": "announcement_writing", "slug": "welcome-post", "description": "AI tools demo", "keywords": ["mcp"], "instructions": "Write a welcome announcement."}'
```

### Level 3: Agents — The Autonomous Era

Define the goal — agents find the path. Multi-step reasoning with tool use.

```bash
systemprompt admin agents message welcome -m "Create an announcement blog" --blocking --timeout 120
systemprompt infra jobs run publish_pipeline
```

### Level 4: Mesh — The Orchestration Era

Superagents coordinate specialized agents. Load a playbook in Claude Code and let the mesh deliver.

```bash
systemprompt core playbooks show content_blog
# Then in Claude Code: "Run the content_blog playbook and publish the results"
```

**Level 4 is the endgame** — agents that regenerate, learn from performance metrics, and orchestrate other agents.

---

## What's Included

| Component | Count | Description |
|-----------|-------|-------------|
| **Extensions** | 4 | Web publishing, Soul memory, MCP servers, CLI tools |
| **MCP Servers** | 3 | `systemprompt` (CLI), `soul` (memory), `content-manager` |
| **Playbooks** | 96 | Machine-readable operational guides |
| **Agent** | 1 | Pre-configured welcome assistant |
| **Skills** | 5+ | General assistance, content writing, technical writing |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  Your Project (this template)                                    │
│                                                                  │
│  extensions/              ← Your Rust code                       │
│  ├── web/                   Web publishing, themes, SEO          │
│  ├── soul/                  Long-term memory system              │
│  ├── mcp/                   MCP servers (3 included)             │
│  └── cli/                   CLI extensions                       │
│                                                                  │
│  services/                ← Your configuration (YAML/Markdown)   │
│  ├── agents/                Agent definitions                    │
│  ├── skills/                Skill configurations                 │
│  ├── playbook/              96 operational playbooks             │
│  └── web/                   Themes, templates, homepage          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  core/ (git submodule - READ ONLY)                               │
│                                                                  │
│  systemprompt-core provides:                                     │
│  ├── API server + CLI                                            │
│  ├── Agent runtime + A2A protocol                                │
│  ├── MCP server hosting                                          │
│  ├── Auth (OAuth2 + WebAuthn)                                    │
│  ├── Memory systems                                              │
│  └── Observability + logging                                     │
└─────────────────────────────────────────────────────────────────┘
```

**Key rules:**
- Rust code → `extensions/`
- Configuration → `services/`
- Core is **read-only** (git submodule)

---

## Project Structure

```
systemprompt-template/
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
├── core/                    # READ-ONLY - systemprompt-core submodule
├── Cargo.toml               # Workspace manifest
└── justfile                 # Development commands
```

---

## Commands

| Command | Description |
|---------|-------------|
| `just build` | Build the project |
| `just start` | Start all services |
| `just login` | Authenticate with cloud |
| `just profile` | Create/select profile |
| `just deploy` | Build and deploy to cloud |
| `just db-up` | Start local PostgreSQL |
| `just db-down` | Stop local PostgreSQL |
| `just migrate` | Run database migrations |

---

## Playbooks

Every operation has a playbook. Don't guess — read the playbook first.

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
| `config_*` | Configuration references |
| `domain_*` | Domain-specific operations |

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

This template extends [systemprompt-core](https://github.com/systempromptio/systemprompt-core) — a production-ready AI agent orchestration library that provides:

- **Complete runtime**: API server, agent runtime, MCP hosting
- **Authentication**: OAuth2 for APIs, WebAuthn for passwordless login
- **Memory systems**: Long-term, short-term, and working memory
- **A2A protocol**: Agent-to-agent communication and orchestration
- **Observability**: Full request logging and audit trails

Works with Claude Code, Claude Desktop, ChatGPT, and any MCP-compatible tool.

---

## License

MIT — see [LICENSE](LICENSE)

Depends on [systemprompt-core](https://github.com/systempromptio/systemprompt-core) (FSL-1.1-ALv2).

---

<div align="center">

**[Documentation](https://systemprompt.io/documentation)** · **[Discord](https://discord.gg/wkAbSuPWpr)** · **[systemprompt-core](https://github.com/systempromptio/systemprompt-core)** · **[Issues](https://github.com/systempromptio/systemprompt-template/issues)**

Questions or issues? Join us on [Discord](https://discord.gg/wkAbSuPWpr) for help.

</div>
