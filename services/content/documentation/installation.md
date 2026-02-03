---
title: "Installation"
description: "Clone the systemprompt-template to get a complete running Rust binary with A2A agents, MCP servers, and a static-generated homepage ready to extend."
author: "SystemPrompt Team"
slug: "installation"
keywords: "installation, setup, rust, template, a2a agents, mcp servers, production runtime"
image: "/files/images/docs/installation.svg"
kind: "guide"
public: true
tags: []
published_at: "2025-01-27"
updated_at: "2026-02-02"
---

# Installation

systemprompt.io is the production runtime for agentic products—an embeddable Rust library that enables your agents to run, learn, and adapt. One `cargo build --release` produces a 50MB binary containing everything: OAuth2 server, WebAuthn authentication, MCP host, agent orchestrator, scheduler, file storage, and analytics.

No Docker compose. No sidecars. Clone, build, run.

## Built for Superintelligence

systemprompt.io is designed to be operated by AI agents, not just humans. Our playbooks are machine-native guides—deterministic, self-repairing, and executable.

**For AI agents assisting with setup**, read the CLAUDE.md file in the template repository. This contains comprehensive machine-readable instructions optimized for AI-assisted development.

**For deep machine instructions on all operations**, use our playbooks:

```bash
systemprompt core playbooks show guide_start
```

The `guide_start` playbook is required reading—it's the master index linking to all operational playbooks.

## Prerequisites

Before installing, ensure you have:

- **Rust 1.75+** — Install from rustup.rs (see Links section)
- **Git** — For cloning repositories
- **GitHub CLI** (recommended) — `gh` for template creation
- **PostgreSQL** — The only external dependency (see database options below)

### Verify Prerequisites

```bash
rustc --version    # Should output 1.75.0 or higher
git --version
gh --version       # Optional but recommended
```

## Quick Start (Recommended)

Clone the systemprompt-template to get started. The template includes a complete working project with example agents, MCP servers, and configuration.

### 1. Create Your Project

**Option A: GitHub CLI (Recommended)**

```bash
gh repo create my-ai --template systempromptio/systemprompt-template --clone --private
cd my-ai
git submodule update --init --recursive
```

**Option B: GitHub Web UI**

Click **"Use this template"** on the systemprompt-template repository, then:

```bash
git clone --recursive https://github.com/YOUR_USERNAME/my-ai.git
cd my-ai
```

### 2. Build the Binary

```bash
SQLX_OFFLINE=true cargo build --release -p systemprompt-cli
```

The `SQLX_OFFLINE=true` flag is required for the first build since no database exists yet.

### 3. Setup Your Profile

```bash
# Login to systemprompt.io Cloud (enables profile management)
systemprompt cloud auth login
```

**Login is manual only.** This command opens your browser for GitHub or Google OAuth. AI agents cannot run this command—you must complete authentication yourself. Registration is free and required for the license grant that enables CLI usage.

```bash
# Create a local tenant (will prompt for database URL)
systemprompt cloud tenant create --type local

# Create your profile
systemprompt cloud profile create local

# Run database migrations
systemprompt infra db migrate
```

### 4. Start Services

```bash
just start
```

Or manually:

```bash
systemprompt infra services start --all
```

## What You Get

After installation, you have a complete running system on `http://localhost:8080`:

| Component | Description |
|-----------|-------------|
| **A2A Agents** | Out-of-the-box agent orchestration with Google's A2A protocol. Automatic discovery, capability negotiation, and secure cross-agent messaging. |
| **MCP Servers** | Production-ready MCP hosting with OAuth2 protection. Connect Claude Desktop, Claude Code, or ChatGPT to your tools securely. |
| **Static Homepage** | A static-generated homepage ready to customize. Edit YAML in `services/web/`, rebuild, and see changes immediately. |
| **OAuth2/OIDC Server** | Full authorization server with PKCE flows, WebAuthn passwordless auth, and user isolation. |
| **CLI Control Plane** | The same CLI that works locally also works in production. Every action auditable. |

## Verify Installation

```bash
# Check service status
systemprompt infra services status

# Check database connection
systemprompt infra db status

# List available agents
systemprompt admin agents list
```

Visit `http://localhost:8080` to see your homepage.

## Extend and Customize

The template has three core directories that define your application:

### `.systemprompt/` — Credentials & Cloud Management

The `.systemprompt/` directory manages your authentication and environment configuration. When you run `systemprompt cloud auth login`, your credentials are stored here, enabling the CLI to access your tenants in both local development and production.

```text
.systemprompt/
├── credentials.json     # Your cloud API credentials (gitignored)
├── tenants.json         # Registry of your tenants (gitignored)
└── profiles/            # Environment-specific configuration
    └── local/
        ├── profile.yaml # Server, paths, security settings
        └── secrets.json # DATABASE_URL, API keys (gitignored)
```

This directory is your personal credential store. It's gitignored by default—each developer and environment has their own.

See the Configuration documentation for profile and secrets management.

### `services/` — Config as Code

The `services/` directory contains all application configuration as YAML and Markdown files. No Rust code here—just declarative configuration that can be edited, version-controlled, and synced between environments.

| Directory | Purpose |
|-----------|---------|
| `services/agents/` | AI agent definitions with system prompts and skills |
| `services/mcp/` | MCP server configurations |
| `services/skills/` | Reusable skill definitions for agents |
| `services/content/` | Blog posts, documentation (Markdown) |
| `services/web/` | Theme, branding, navigation |
| `services/ai/` | AI provider configuration |
| `services/scheduler/` | Background job scheduling |

Changes to `services/` can be synced to your database with `systemprompt cloud sync local --all --direction to-db -y`. This makes configuration portable across environments.

See the Services documentation for detailed information on each service type.

### `extensions/` — Rust Crates

The `extensions/` directory contains Rust crates that wrap and extend the core framework. This is where you write custom code to add deterministic tool use, database access, API routes, background jobs, MCP servers, or anything else that requires compiled Rust.

```text
extensions/
├── web/                 # Web extension (API routes, schemas, jobs)
│   ├── Cargo.toml
│   └── src/
│       ├── extension.rs # Extension trait implementation
│       ├── api/         # Custom API endpoints
│       ├── models/      # Database models
│       └── jobs/        # Background jobs
├── cli/                 # CLI extensions (custom commands)
└── mcp/                 # MCP server extensions (tool servers)
```

Extensions implement traits from `systemprompt-core` and are compiled into your binary. They have full access to the database, domain services, and can implement:

- **API routes** — Custom HTTP endpoints with authentication
- **Database schemas** — SQLx migrations and models
- **Background jobs** — Scheduled or triggered async tasks
- **MCP servers** — Tool servers for Claude, ChatGPT, and other clients
- **CLI commands** — Custom CLI subcommands

See the Extensions documentation for building custom functionality.

## Alternative: Install from crates.io

For adding systemprompt.io to an existing Rust project:

```bash
cargo install systemprompt-cli
```

This installs the `systemprompt` binary to your Cargo bin directory. You'll still need to create configuration files and a project structure. The template approach above is recommended for new projects.

## Cloud Deployment

For managed PostgreSQL and one-click deployment:

```bash
# Create a cloud tenant (provisions managed PostgreSQL)
systemprompt cloud tenant create --region iad

# Create production profile
systemprompt cloud profile create production

# Deploy
systemprompt cloud deploy --profile production
```

See the Cloud Deployment documentation for details.

## Database Options

PostgreSQL is the only external dependency. You need exactly one thing: a `DATABASE_URL` connection string with credentials stored in your profile's secrets. It doesn't matter where PostgreSQL runs or who hosts it—the SystemPrompt binary uses this URL to manage your tenant's data.

| Option | Best For | Example DATABASE_URL |
|--------|----------|---------------------|
| **Docker** | Local dev | `postgres://postgres:postgres@localhost:5432/systemprompt` |
| **Local Install** | Development | `postgres://user:pass@localhost:5432/systemprompt` |
| **Neon** | Free tier, serverless | `postgres://user:pass@ep-xxx.neon.tech/systemprompt?sslmode=require` |
| **Supabase** | Free tier, dashboard | `postgres://postgres:pass@db.xxx.supabase.co:5432/postgres?sslmode=require` |
| **AWS RDS** | Enterprise | `postgres://admin:pass@mydb.xxx.rds.amazonaws.com:5432/systemprompt?sslmode=require` |
| **SystemPrompt Cloud** | Production, zero config | Managed automatically |

When you run `systemprompt cloud tenant create --type local`, you're prompted for this URL. It's stored in `.systemprompt/profiles/<name>/secrets.json` (gitignored).

See the Database Configuration documentation for detailed setup instructions.

---

## Related Resources

**Playbooks:**
- `systemprompt core playbooks show build_installation` — Installation steps
- `systemprompt core playbooks show guide_start` — Required first read
- `systemprompt core playbooks show cli_session` — Session management
- `systemprompt core playbooks show cli_cloud` — Cloud setup

**Source Code:**
See the Related Code section for links to the systemprompt-template and AI agent instructions (CLAUDE.md).