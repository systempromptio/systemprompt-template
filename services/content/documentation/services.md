---
title: "Services"
description: "Configure and manage the runtime components that power your SystemPrompt instance - agents, skills, MCP servers, content, AI, scheduled jobs, and web interface."
author: "SystemPrompt Team"
slug: "services"
keywords: "services, configuration, agents, mcp, scheduler, web, ai, skills, playbooks, content, runtime"
image: "/files/images/docs/services.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Services

**TL;DR:** Services are the runtime components that define how your SystemPrompt instance behaves. They manage agents, skills, MCP servers, content, AI providers, scheduled jobs, playbooks, and the web interface through YAML configuration files.

After reading this, you will understand exactly how services handle the runtime execution and evolution of your SystemPrompt instance. They manage the relationship between agents, skills, mcp-servers, playbooks, content, ai, scheduled jobs and the web interface of SystemPrompt.

## What Are Services?

Services are configuration-driven components that define the runtime behavior of your SystemPrompt application. Unlike extensions which contain Rust code that executes logic, services are pure YAML and Markdown files that describe what your application does and how it connects to the world.

Services define:

- **What** your application does through agents and their skills
- **How** it connects to external systems through AI providers and MCP servers
- **When** automated tasks run through the scheduler
- **What** users see through the web interface, navigation, and theme

The key distinction is that services contain no code. All Rust implementation lives in extensions. Services are the configuration layer that tells those extensions what to do.

## Service Types

SystemPrompt includes 9 service types, each handling a specific domain of functionality:

| Service | Directory | Purpose |
|---------|-----------|---------|
| [Config](/documentation/services/config) | `services/config/` | Aggregates all service configurations |
| [Agents](/documentation/services/agents) | `services/agents/` | AI agent definitions with A2A protocol |
| [Skills](/documentation/services/skills) | `services/skills/` | Reusable agent capabilities |
| [AI](/documentation/services/ai) | `services/ai/` | LLM provider configuration |
| [MCP](/documentation/services/mcp) | `services/mcp/` | MCP server hosting and configuration |
| [Content](/documentation/services/content) | `services/content/` | Content sources, categories, indexing |
| [Playbooks](/documentation/services/playbooks) | `services/playbook/` | Machine-executable instruction sets |
| [Scheduler](/documentation/services/scheduler) | `services/scheduler/` | Background jobs and cron scheduling |
| [Web](/documentation/services/web) | `services/web/` | Branding, navigation, templates, theme |

## Service Relationships

Services work together as an interconnected system. Understanding these relationships helps you configure your instance effectively.

```
                    ┌─────────────────────────────────────────┐
                    │              CONFIG SERVICE              │
                    │    Aggregates all service configs        │
                    └───────────────────┬─────────────────────┘
                                        │ includes
        ┌───────────────┬───────────────┼───────────────┬───────────────┐
        ▼               ▼               ▼               ▼               ▼
┌───────────────┐ ┌───────────┐ ┌───────────────┐ ┌───────────┐ ┌───────────────┐
│    AGENTS     │ │   SKILLS  │ │      AI       │ │    WEB    │ │   SCHEDULER   │
│ Agent defns   │◄─┤Capabilities│ │ LLM providers │ │  UI/Theme │ │ Cron jobs     │
└───────┬───────┘ └───────────┘ └───────┬───────┘ └─────┬─────┘ └───────────────┘
        │                               │               │
        │ uses tools                    │ connects      │ renders
        ▼                               ▼               ▼
┌───────────────────────────────────────────────────────────────────────────────┐
│                              MCP SERVERS                                       │
│               AI-accessible tools via Model Context Protocol                   │
└───────────────────────────────────────────────────────────────────────────────┘
                                        │
                                        │ stores
                                        ▼
┌───────────────────────────────────────────────────────────────────────────────┐
│                              CONTENT SERVICE                                   │
│            Blog, Documentation, Legal pages, Playbooks - indexed              │
└───────────────────────────────────────────────────────────────────────────────┘
```

The key relationships are:

- **Config orchestrates** - The config service aggregates all other service configurations via includes, creating a unified configuration hub that is validated at startup
- **Agents use Skills** - Agents reference skills by ID in their configuration. Skills define reusable capabilities that multiple agents can share
- **AI powers Agents** - The AI service configures which LLM providers are available to agents, with auto-discovery of MCP servers for tool access
- **MCP provides Tools** - MCP servers expose tools that agents can invoke during conversations through the Model Context Protocol
- **Content stores Playbooks** - Playbooks are stored as markdown files in the content service, synced to the database, and rendered through the web service
- **Scheduler automates** - Background jobs publish content, clean up sessions, and perform maintenance tasks on cron schedules
- **Web renders everything** - The web service provides templates, navigation, and theming for all content types

## Directory Structure

```
services/
├── config/               # Global configuration hub
│   └── config.yaml       # Aggregates all services via includes
├── agents/               # Agent definitions
│   └── welcome.yaml      # Default welcome agent
├── skills/               # Agent capabilities
│   ├── config.yaml       # Skills configuration
│   └── */                # Individual skill definitions
├── ai/                   # AI provider settings
│   └── config.yaml       # Provider configuration
├── mcp/                  # MCP servers
│   └── systemprompt.yaml # SystemPrompt MCP server
├── content/              # Content sources
│   ├── config.yaml       # Content source definitions
│   ├── blog/             # Blog articles
│   ├── documentation/    # Product documentation
│   ├── legal/            # Legal pages
│   └── playbooks/        # Playbook guides
├── playbook/             # Playbook source files
│   ├── guide/            # Getting started, meta
│   ├── cli/              # CLI operation playbooks
│   ├── build/            # Development playbooks
│   └── content/          # Content creation playbooks
├── scheduler/            # Background jobs
│   └── config.yaml       # Job scheduling
└── web/                  # Web interface
    ├── config.yaml       # Branding, typography
    ├── config/           # Navigation, homepage, features
    └── templates/        # HTML templates
```

## Configuration Pattern

All services follow the same configuration pattern:

1. **YAML-only** - Services contain no Rust code. All configuration is in YAML files with optional Markdown content.

2. **Environment variables** - Secrets and environment-specific values use the `${VAR_NAME}` syntax for substitution at runtime.

3. **Hot-reloadable** - Most configuration changes take effect without restarting the application. The scheduler, web navigation, and content changes reload automatically.

4. **Validated** - Schema validation runs at startup. Invalid configuration prevents the application from starting, with clear error messages about what needs to be fixed.

## Managing Services

After modifying service files, sync your changes to the database:

```bash
# Sync all configuration
systemprompt cloud sync local --all --direction to-db -y

# Sync specific service types
systemprompt cloud sync local agents --direction to-db -y
systemprompt cloud sync local skills --direction to-db -y
systemprompt cloud sync local mcp --direction to-db -y
```

To publish content changes:

```bash
systemprompt infra jobs run publish_pipeline
```

## Service Guides

Each service has detailed documentation:

| Service | What You Will Learn |
|---------|---------------------|
| [Agents](/documentation/services/agents) | Define agents, A2A protocol, system prompts, OAuth security |
| [AI](/documentation/services/ai) | Configure providers, MCP discovery, smart routing |
| [Analytics](/documentation/services/analytics) | Track costs, usage metrics, audit trails, dashboards |
| [Auth](/documentation/services/auth) | OAuth2 authentication, WebAuthn, session management |
| [Config](/documentation/services/config) | Aggregate configurations, global settings, includes pattern |
| [Content](/documentation/services/content) | Content sources, categories, indexing, sitemap |
| [Files](/documentation/services/files) | File uploads, storage, serving, CDN integration |
| [MCP](/documentation/services/mcp) | Host MCP servers, OAuth authentication, tool exposure |
| [Playbooks](/documentation/services/playbooks) | Playbook structure, categories, CLI access |
| [Scheduler](/documentation/services/scheduler) | Job definitions, cron scheduling, automation |
| [Skills](/documentation/services/skills) | Create skills, skill schema, assign to agents |
| [Users](/documentation/services/users) | Multi-tenant architecture, user management, permissions |
| [Web](/documentation/services/web) | Branding, navigation, templates, theme customization |
| [Workflows](/documentation/services/workflows) | Agent-playbook-skill orchestration, automation patterns |