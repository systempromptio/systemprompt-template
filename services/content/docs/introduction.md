---
title: "Documentation"
description: "Learn how to build and customize your SystemPrompt application"
author: "Author"
slug: "introduction"
keywords: "documentation, systemprompt, getting-started, guide"
image: ""
kind: "guide"
public: true
tags: ["documentation", "getting-started"]
published_at: "2025-01-01"
---

# Documentation

Welcome to your project documentation. This guide covers everything you need to know to build, customize, and deploy your application.

## Overview

Your project is built on SystemPrompt, an extensible framework for AI-powered applications. It includes:

- **Agent Framework**: Build intelligent agents with custom capabilities
- **Extension System**: Add functionality through modular Rust extensions
- **Content Management**: Built-in blog and documentation support
- **MCP Integration**: Model Context Protocol servers for AI tool access

## Project Structure

```
your-project/
├── core/                    # SystemPrompt core (git submodule)
├── extensions/              # Your Rust extensions
│   ├── blog/               # Blog extension
│   └── mcp/                # MCP servers
│       ├── admin/          # Admin analytics server
│       ├── infrastructure/ # Deployment server
│       └── system-tools/   # File system tools
├── services/               # Configuration (YAML/Markdown only)
│   ├── agents/             # Agent definitions
│   ├── config/             # Service configuration
│   ├── content/            # Blog and docs
│   ├── skills/             # Agent skills
│   └── web/                # Theme and branding
└── src/                    # Application entry point
```

## Quick Start

### Prerequisites

- Rust 1.75+
- Docker (for PostgreSQL)
- Just command runner

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

Define your AI agents in `services/agents/`. Each agent has:

- **System Prompt**: Instructions for the AI
- **Skills**: Capabilities the agent can use
- **MCP Servers**: Tool access for the agent

### Theme

Customize your theme in `services/web/config.yaml`:

- Colors (light and dark mode)
- Typography
- Spacing and layout
- Branding (logos, favicon)

### Content

Add content in `services/content/`:

- **blog/**: Blog posts in Markdown
- **docs/**: Documentation pages
- **legal/**: Legal pages (privacy, terms)

## Building Extensions

Extensions live in `extensions/` and must implement the `Extension` trait:

```rust
impl Extension for MyExtension {
    fn metadata(&self) -> ExtensionMetadata { ... }
    fn schemas(&self) -> Vec<SchemaDefinition> { ... }
    fn router(&self, ctx: &ExtensionContext) -> Option<Router> { ... }
    fn jobs(&self) -> Vec<Arc<dyn Job>> { ... }
}

register_extension!(MyExtension);
```

See `CLAUDE.md` for detailed Rust coding standards.

## Deployment

Deploy to cloud using the Infrastructure MCP server or the CLI:

```bash
just deploy
```

This builds a release binary, creates a Docker image, and deploys to your configured cloud provider.
