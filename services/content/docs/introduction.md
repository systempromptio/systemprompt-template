---
title: "Documentation"
description: "Learn how to build, customize, and deploy your systemprompt application"
author: "systemprompt"
slug: "introduction"
keywords: "documentation, systemprompt, getting-started, guide, mcp, agents, cli"
image: ""
kind: "guide"
public: true
tags: ["documentation", "getting-started"]
published_at: "2025-01-01"
updated_at: "2025-01-01"
---

# Documentation

This guide covers building, customizing, and deploying your systemprompt application.

## Overview

Your project is built on [systemprompt-core](https://github.com/systempromptio/systemprompt-core), an open platform for production AI agent infrastructure. It provides:

- **Agent Registry** — Deploy A2A-compliant agents with OAuth2 security and discovery endpoints
- **MCP Server Registry** — Host MCP servers with HTTP-native transports, auth, and per-tool permissions
- **Extension System** — Add functionality through modular Rust extensions on the open core
- **Content Management** — Markdown ingestion, search, and publishing for blogs and docs
- **CLI** — Manage agents, MCP servers, content, and infrastructure from a single command

## Project Structure

```
your-project/
├── core/                    # systemprompt-core (git submodule, read-only)
├── extensions/              # Your Rust extensions
│   ├── blog/               # Blog extension
│   └── mcp/                # MCP servers
│       └── systemprompt/   # CLI MCP server
├── services/               # Configuration (YAML/Markdown only)
│   ├── agents/             # Agent definitions (A2A cards, OAuth scopes)
│   ├── config/             # Service configuration
│   ├── content/            # Blog and documentation content
│   ├── mcp/                # MCP server definitions
│   ├── skills/             # Agent skill definitions
│   ├── scheduler/          # Cron jobs and background tasks
│   └── web/                # Theme, branding, navigation
└── src/                    # Application entry point
```

## Quick Start

### Prerequisites

- Rust 1.75+
- Docker (for PostgreSQL) or a systemprompt.io Cloud account
- Just command runner (optional)

### Setup

```bash
# Clone with submodules
git clone --recursive your-repo-url
cd your-project

# Start database
just db-up

# Run migrations and sync content
just quickstart

# Start the server
just start
```

### Development Commands

```bash
just build              # Build all crates
just start              # Start the server
just migrate            # Run database migrations
just sync-local         # Sync content to database
just build-mcp          # Build MCP servers
```

## Configuration

### Agents

Define AI agents in `services/agents/`. Each agent has:

- **A2A Card** — Protocol version, capabilities, transport, security schemes
- **Skills** — Capabilities the agent can use
- **OAuth Scopes** — Who can interact with the agent
- **System Prompt** — Instructions for the underlying LLM

Discovery endpoints:

| Endpoint | Description |
|----------|-------------|
| `/.well-known/agent-card.json` | Default agent card |
| `/.well-known/agent-cards` | List all agents |
| `/api/v1/agents/registry` | Full registry with status |

### MCP Servers

Define MCP servers in `services/mcp/`. Each server has:

- **Binary** — The compiled Rust crate to run
- **Endpoint** — HTTP URL for streamable transport
- **OAuth** — Required scopes and audience restrictions

Discovery endpoint: `/api/v1/mcp/registry`

### Theme

Customize appearance in `services/web/config.yaml`:

- Colors (light and dark mode)
- Typography
- Spacing and layout
- Branding (logos, favicon)

### Content

Add content in `services/content/`:

- **blog/** — Blog posts in Markdown with frontmatter
- **docs/** — Documentation pages
- **legal/** — Legal pages (privacy, terms)

## Building Extensions

Extensions live in `extensions/` and implement the `Extension` trait:

```rust
use systemprompt_extension::*;

struct MyExtension;

impl Extension for MyExtension {
    fn metadata(&self) -> ExtensionMetadata { ... }
    fn schemas(&self) -> Vec<SchemaDefinition> { ... }
    fn router(&self, ctx: &ExtensionContext) -> Option<Router> { ... }
    fn jobs(&self) -> Vec<Arc<dyn Job>> { ... }
}

register_extension!(MyExtension);
```

Available extension traits:

| Trait | Purpose |
|-------|---------|
| `Extension` | Base — ID, name, version, dependencies |
| `SchemaExtension` | Database table definitions |
| `ApiExtension` | HTTP route handlers |
| `ConfigExtensionTyped` | Config validation at startup |
| `JobExtension` | Background job definitions |
| `ProviderExtension` | Custom LLM/tool provider implementations |

See `CLAUDE.md` for Rust coding standards.

## Deployment

### Local

```bash
systemprompt infra services start --all
```

### Cloud

```bash
systemprompt cloud auth login
systemprompt cloud tenant create --region iad
systemprompt cloud profile create production
systemprompt cloud deploy --profile production
```

Your platform will be available at your tenant URL (e.g., `https://my-tenant.systemprompt.io`).
