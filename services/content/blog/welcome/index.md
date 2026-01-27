---
title: "systemprompt.io — Production Infrastructure for AI Agents"
description: "Self-hosted or cloud. The complete platform for deploying AI agents, MCP servers, and multi-agent orchestration on open standards."
author: "systemprompt"
slug: "welcome"
keywords: "systemprompt, mcp, ai agents, agent registry, mcp server, cli, a2a, oauth2, self-hosted, cloud"
image: ""
kind: "article"
public: true
tags: ["welcome", "getting-started", "mcp", "agents"]
published_at: "2025-01-01"
updated_at: "2025-01-01"
---

# systemprompt.io

**Production infrastructure for AI agents. Self-hosted or cloud.**

The missing layer between AI frameworks and production deployment. Not another SDK — complete infrastructure with authentication, permissions, and multi-agent orchestration built on open standards (MCP, A2A, OAuth2).

---

## Core Features

### Agent Registry

Define, deploy, and manage AI agents as config. Each agent gets an A2A-compliant card, OAuth2 security, and a discoverable endpoint — no code changes required.

```
/.well-known/agent-card.json      # Default agent card
/.well-known/agent-cards          # List all agents
/.well-known/agent-cards/{name}   # Specific agent card
/api/v1/agents/registry           # Full registry with status
```

Agents are defined in YAML with granular permissions:

```yaml
# services/agents/welcome.yaml
agents:
  welcome:
    name: "welcome"
    enabled: true
    card:
      protocolVersion: "0.3.0"
      capabilities:
        streaming: true
      security:
        - oauth2: ["anonymous"]
```

### MCP Server Registry

Host MCP (Model Context Protocol) servers with built-in auth, discovery, and HTTP-native transports. Every MCP server is accessible to any compliant client — Claude Code, Claude Desktop, ChatGPT, and more.

```
/api/v1/mcp/registry              # All MCP servers with endpoints
/api/v1/mcp/{name}/mcp            # Streamable HTTP transport
```

MCP servers are also config-driven:

```yaml
# services/mcp/systemprompt.yaml
mcp_servers:
  systemprompt:
    binary: "systemprompt-mcp"
    port: 5010
    endpoint: "http://localhost:8080/api/v1/mcp/systemprompt/mcp"
    enabled: true
    oauth:
      required: true
      scopes: ["admin"]
```

### Config as Code

Your entire infrastructure lives in the `services/` directory:

| Directory | Purpose |
|-----------|---------|
| `services/agents/` | Agent definitions with OAuth scopes and A2A cards |
| `services/mcp/` | MCP servers with per-tool permissions |
| `services/skills/` | Reusable agent capabilities |
| `services/content/` | Markdown content (blog, docs, legal) |
| `services/scheduler/` | Cron jobs and background tasks |
| `services/web/` | Theme, branding, navigation |

All config changes deploy instantly — no code changes or rebuilds required.

### Built-in Platform Services

Everything you need, out of the box:

- **Authentication** — OAuth2/OIDC + WebAuthn passwordless auth
- **Permissions** — Role-based, per-agent, per-tool scopes
- **File Storage** — Upload, serve, and manage files with metadata
- **Content Management** — Markdown ingestion, search, and publishing
- **AI Integration** — Multi-provider LLM support (Anthropic, OpenAI, Gemini)
- **Analytics** — Session tracking, metrics, and usage reporting
- **Scheduling** — Cron-based deterministic job execution

---

## CLI Quick Start

The `systemprompt` CLI is the universal interface for managing agents, MCP servers, content, and infrastructure. The same CLI works locally during development and in production on your cloud instance.

### Install

```bash
# From crates.io
cargo install systemprompt-cli

# Or build from source
git clone https://github.com/systempromptio/systemprompt-core
cd systemprompt-core && cargo build --release -p systemprompt-cli
```

### Setup (Local)

```bash
# Start PostgreSQL
docker run -d --name systemprompt-db \
  -e POSTGRES_DB=systemprompt \
  -e POSTGRES_USER=systemprompt \
  -e POSTGRES_PASSWORD=systemprompt \
  -p 5432:5432 postgres:16

# Authenticate (free account)
systemprompt cloud auth login

# Create a local tenant and profile
systemprompt cloud tenant create --type local
systemprompt cloud profile create local

# Migrate and start
systemprompt infra db migrate
systemprompt infra services start --all
```

### Setup (Cloud)

```bash
# Authenticate
systemprompt cloud auth login

# Create a cloud tenant (provisions managed DB + VM)
systemprompt cloud tenant create --region iad

# Create profile and deploy
systemprompt cloud profile create production
systemprompt cloud deploy --profile production
```

Your platform will be available at your tenant URL (e.g., `https://my-tenant.systemprompt.io`). Point your own domain via CNAME.

### Common Commands

```bash
# Services
systemprompt infra services start          # Start all services
systemprompt infra services status         # Check status

# Agents
systemprompt admin agents list --enabled   # List active agents
systemprompt admin agents message welcome -m "Hello"  # Message an agent

# Content
systemprompt core content list             # List all content
systemprompt core content ingest --source blog ./services/content/blog

# MCP
systemprompt plugins mcp                   # List MCP servers

# Database
systemprompt infra db status               # DB health
systemprompt infra db tables               # List tables

# Logs
systemprompt infra logs stream --level error
```

### Playbooks

The CLI includes 19 built-in playbooks for step-by-step guidance:

```bash
systemprompt_help { "command": "playbook" }           # List all playbooks
systemprompt_help { "command": "playbook session" }   # Specific playbook
```

Available: agents, analytics, build, cloud, config, content, contexts, database, deploy, files, jobs, logs, plugins, services, session, skills, sync, users, web.

---

## Connect Your MCP Client

systemprompt MCP servers work with any client that supports streamable HTTP transport. No local process management needed — connect directly over HTTP.

### Claude Desktop

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "systemprompt": {
      "url": "https://your-tenant.systemprompt.io/api/v1/mcp/systemprompt/mcp",
      "transport": "streamable-http"
    }
  }
}
```

### Claude Code

```bash
claude mcp add systemprompt \
  --transport http \
  https://your-tenant.systemprompt.io/api/v1/mcp/systemprompt/mcp
```

### Local Development

When running locally, connect to the local endpoint:

```json
{
  "mcpServers": {
    "systemprompt": {
      "url": "http://localhost:8080/api/v1/mcp/systemprompt/mcp",
      "transport": "streamable-http"
    }
  }
}
```

### Discovery

Use the registry endpoint to discover all available MCP servers and their connection details:

```bash
curl https://your-tenant.systemprompt.io/api/v1/mcp/registry
```

Each entry includes the server name, endpoint URL, description, and required OAuth scopes.

---

## Architecture

systemprompt uses a layered crate architecture where dependencies flow downward only:

```
┌───────────────────────────────────────────────────────┐
│  ENTRY: api, cli                                       │
├───────────────────────────────────────────────────────┤
│  APP: runtime, scheduler, generator, sync              │
├───────────────────────────────────────────────────────┤
│  DOMAIN: users, oauth, ai, agent, mcp, files, content  │
├───────────────────────────────────────────────────────┤
│  INFRA: database, events, security, config, logging    │
├───────────────────────────────────────────────────────┤
│  SHARED: models, traits, identifiers, extension        │
└───────────────────────────────────────────────────────┘
```

Extensions plug into the core without modifying it — implement Rust traits, register at runtime, and your code runs alongside the platform.

---

## Links

- [GitHub — systemprompt-core](https://github.com/systempromptio/systemprompt-core)
- [GitHub — systemprompt-template](https://github.com/systempromptio/systemprompt-template)
- [Documentation](https://docs.systemprompt.io)
- [Website](https://systemprompt.io)
