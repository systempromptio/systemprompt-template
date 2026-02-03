---
title: "Welcome to systemprompt.io"
description: "Production infrastructure for AI agents. Self-hosted or cloud. The missing layer between AI frameworks and production deployment."
slug: "welcome-to-systemprompt"
kind: "blog"
public: true
author: "systemprompt.io"
published_at: "2026-02-03"
tags: ["announcement", "getting-started", "infrastructure"]
keywords: "systemprompt, AI agents, production infrastructure, MCP, A2A, OAuth2"
---

# Welcome to systemprompt.io

**Production infrastructure for AI agents. Self-hosted or cloud.**

The missing layer between AI frameworks and production deployment. Not another SDK - complete infrastructure with authentication, permissions, and multi-agent orchestration built on open standards (MCP, A2A, OAuth2).

## Your AI assistant shouldn't live under your desk

Personal AI assistants are finally real. But there's a gap between a demo and a product your users can actually use.

systemprompt.io bridges that gap:

- **Auth that works**: OAuth2/OIDC + WebAuthn
- **Permissions that scale**: Per-user, per-agent, per-tool scopes
- **Deployment that's real**: One command to cloud or self-host
- **Multi-agent that coordinates**: A2A protocol with shared state

The Jarvis dream is here. We handle the infrastructure.

## Core Capabilities

- **Complete Runtime**: Web API + agent processes + MCP servers with shared auth and database
- **Open Standards**: MCP, A2A, OAuth2, WebAuthn - no vendor lock-in
- **Agent-Executable CLI**: Your AI manages infrastructure directly via the same CLI you use
- **Native Rust**: Async-first on Tokio, zero-cost abstractions
- **Self-Hosted or Cloud**: Docker locally, or deploy to isolated VM with managed database
- **100% Extensible**: Build proprietary Rust extensions on the open core

## What You Get

A complete platform with built-in:

- **User Authentication**: OAuth2/OIDC, sessions, roles, and permissions
- **File Storage**: Upload, serve, and manage files with metadata
- **Content Management**: Markdown ingestion, search, and publishing
- **AI Integration**: Multi-provider LLM support with request logging
- **Analytics**: Session tracking, metrics, and usage reporting
- **Agent Orchestration**: A2A protocol for agent-to-agent communication
- **MCP Servers**: Tool and resource providers for AI clients

## Quick Start

### Prerequisites

- Rust 1.75+
- Docker (for local PostgreSQL) **OR** systemprompt.io Cloud account

### Local Setup

```bash
# Start PostgreSQL in Docker
docker run -d --name systemprompt-db \
  -e POSTGRES_DB=systemprompt \
  -e POSTGRES_USER=systemprompt \
  -e POSTGRES_PASSWORD=systemprompt \
  -p 5432:5432 \
  postgres:16

# Login to systemprompt.io Cloud (free account)
systemprompt cloud auth login

# Create a local tenant
systemprompt cloud tenant create --type local

# Create and configure your profile
systemprompt cloud profile create local

# Run database migrations
systemprompt infra db migrate

# Start services
systemprompt infra services start --all
```

## Native MCP Client Support

Works out of the box with any MCP client - Claude Code, Claude Desktop, ChatGPT, and more.

```json
{
  "mcpServers": {
    "my-server": {
      "url": "https://my-tenant.systemprompt.io/api/v1/mcp/my-server/mcp",
      "transport": "streamable-http"
    }
  }
}
```

Your AI can now manage your entire infrastructure: deploy updates, query analytics, manage users, and orchestrate agents - all through natural conversation.

## Get Started

Check out the [documentation](/documentation) to learn more, or dive into the [getting started guide](/documentation/getting-started) to begin building.

Welcome to the future of AI infrastructure.
