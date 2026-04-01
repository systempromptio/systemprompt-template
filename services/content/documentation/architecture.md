---
title: "Platform Architecture"
description: "Technical architecture overview: single Rust binary, narrow-waist governance layer, CLI-first design, extension system, stateless scaling, and flexible deployment models for enterprise AI governance."
author: "systemprompt.io"
slug: "architecture"
keywords: "architecture, rust, cli, extensions, deployment, enterprise, narrow waist"
kind: "guide"
public: true
tags: ["architecture", "enterprise", "technical"]
published_at: "2026-03-19"
updated_at: "2026-03-20"
after_reading_this:
  - "Understand the three-layer architecture: Core, Extensions, and Services"
  - "Explain the narrow-waist design that sits between client and backend stacks"
  - "Describe how stateless design enables horizontal scaling"
  - "Choose the right deployment model for your infrastructure"
related_docs:
  - title: "Scaling Architecture"
    url: "/documentation/scaling"
  - title: "Rate Limiting & Compliance"
    url: "/documentation/rate-limiting"
  - title: "Access Control"
    url: "/documentation/access-control"
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "MCP Servers"
    url: "/documentation/mcp-servers"
  - title: "Presentation"
    url: "/documentation/presentation"
---

# Platform Architecture

> **See this in the presentation:** [Slide 11: Personalization & Ownership](/documentation/presentation#slide-11)

**TL;DR:** The platform is a single Rust binary with PostgreSQL as its only dependency. It acts as the **narrow waist** between client stacks (agents, UIs, tools) and backend stacks (LLMs, APIs, data) — governing every AI interaction without imposing a specific framework on either side. Three layers — Core (library), Extensions (Rust), Services (YAML/Markdown) — compile into one binary that deploys as a sidecar, standalone service, centralized gateway, or embedded library. CLI-first design exposes every operation across 8 domains. Profile-based deployment means the same binary runs in local development and production with different YAML configs.

## The Narrow Waist

Enterprise AI infrastructure is fragmented. Different teams adopt different agent frameworks, model providers, and tool ecosystems. The governance layer cannot dictate a stack — it must sit between all of them.

```
┌─────────────────────────────────────────────────────────────┐
│                      Client Stacks                           │
│  Claude Code, Custom Agents, Chat UIs, A2A Clients,         │
│  MCP Clients, REST Consumers, Internal Tools                 │
├─────────────────────────────────────────────────────────────┤
│               ▼  systemprompt (narrow waist)  ▼              │
│  Governance: auth, access control, rate limiting,            │
│  audit, cost tracking, compliance, agent registry            │
├─────────────────────────────────────────────────────────────┤
│                      Backend Stacks                          │
│  LLM Providers, MCP Servers, Databases, APIs,                │
│  Internal Services, Data Pipelines, Vector Stores            │
└─────────────────────────────────────────────────────────────┘
```

The platform does not replace anything above or below it. It governs the boundary — authenticating requests, enforcing access policies, tracking costs, rate limiting traffic, and logging everything for audit. AI implementations are fragmented and complex. This adapts to whatever architecture exists.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        CLI Interface                         │
│  systemprompt <domain> <subcommand> [args]                  │
│  8 domains: core, infra, admin, cloud, analytics,           │
│             web, plugins, build                              │
├─────────────────────────────────────────────────────────────┤
│                      HTTP / API Layer                        │
│  REST endpoints, OAuth 2.0, JWT auth, security headers      │
├──────────┬──────────┬──────────┬────────────────────────────┤
│  Web     │  MCP     │  CLI     │  Custom Extensions         │
│  Ext     │  Ext     │  Ext     │  (your code here)          │
├──────────┴──────────┴──────────┴────────────────────────────┤
│                     Core Library                             │
│  Skills, Agents, Plugins, Content, Hooks, Artifacts,        │
│  Access Control, Secrets, Analytics, Jobs                    │
├─────────────────────────────────────────────────────────────┤
│                     PostgreSQL                               │
│  Single database, connection pooling (SQLx async),           │
│  migrations, full-text search                                │
└─────────────────────────────────────────────────────────────┘
```

## Three-Layer Architecture

The platform separates concerns into three distinct layers. Each layer has clear boundaries and rules.

### Layer 1: Core (Library)

The core is a Rust library that provides all platform primitives: skills, agents, plugins, content management, hooks, artifacts, access control, secrets, analytics, and job scheduling. The core is a **git submodule** and is read-only — you never modify it directly.

The core exposes a trait-based extension system. Extensions implement traits to add capabilities without modifying core code.

### Layer 2: Extensions (Rust Code)

Extensions are Rust modules that compile into the single binary alongside the core. All custom Rust code lives in the `extensions/` directory. The platform ships with three built-in extensions:

| Extension | Purpose |
|-----------|---------|
| **Web** | Admin dashboard, content rendering, static assets, template compilation |
| **MCP** | MCP server management, tool registry, OAuth per server |
| **CLI** | Command-line interface, all 8 domains, help system |

Extensions register capabilities through the core's trait system: routes, assets, CLI commands, hooks, data providers, and rendering templates. When you build the platform, all extensions compile into the single binary.

### Layer 3: Services (Configuration)

The services layer is YAML and Markdown only — no Rust code. This is where you configure the platform for your specific deployment:

| Directory | Contents |
|-----------|----------|
| `services/agents/` | Agent YAML configs |
| `services/skills/` | Skill definitions in Markdown |
| `services/plugins/` | Plugin manifests bundling agents, skills, and MCP servers |
| `services/content/` | Documentation, pages, and content in Markdown with YAML frontmatter |
| `.systemprompt/profiles/` | Deployment profiles (local, production) |
| `services/mcp/` | MCP server configurations |

This separation means your team configures agents and skills without touching Rust code, and your developers extend the platform without breaking configurations.

## Deployment Model Flexibility

The single-binary architecture supports multiple deployment models. Choose based on your infrastructure requirements and team topology.

| Model | Description | Best For |
|-------|-------------|----------|
| **Sidecar** | Instance alongside each agent in the same pod/task | Per-agent governance, minimal network hops |
| **Standalone service** | Instances behind a load balancer | Centralized governance, standard enterprise deployment |
| **Centralized multi-tenant** | Single deployment serving multiple teams/units | Unified governance with per-team isolation |
| **Embedded library** | Core compiled directly into your existing Rust binary | Maximum performance, no separate service boundary |

All models share the same PostgreSQL database for centralized policy enforcement and audit. The deployment model is an infrastructure decision — no application code changes required.

## CLI-First Design

Every operation is available from the command line. The CLI follows a consistent pattern:

```
systemprompt <domain> <subcommand> [args]
```

### The 8 Domains

| Domain | Purpose | Example Commands |
|--------|---------|-----------------|
| **core** | Skills, content, files, contexts, plugins, hooks, artifacts | `core skills list`, `core plugins list` |
| **infra** | Services, database, jobs, logs | `infra services status`, `infra logs view` |
| **admin** | Users, agents, config, setup, session | `admin agents list`, `admin users list` |
| **cloud** | Auth, deploy, sync, secrets, tenant, domain | `cloud deploy`, `cloud sync` |
| **analytics** | Overview, conversations, agents, tools, requests, sessions, content, traffic, costs | `analytics costs summary` |
| **web** | Content-types, templates, assets, sitemap, validate | `web assets list`, `web sitemap generate` |
| **plugins** | Extensions, MCP servers, capabilities | `plugins mcp logs <server>` |
| **build** | Build core workspace and MCP extensions | `build` |

Every domain supports `--help` for discoverability:

```bash
systemprompt --help              # Top-level help
systemprompt core --help         # Domain help
systemprompt core skills --help  # Subcommand help
```

The CLI is not a wrapper around an API — it operates directly on the same code paths as the HTTP layer. This means the CLI and the dashboard always behave identically.

## Single Binary + PostgreSQL

The entire platform compiles to a single Rust binary. PostgreSQL is the only external dependency.

| Component | Role |
|-----------|------|
| **Rust binary** | HTTP server, CLI, all extensions, MCP management, job scheduler |
| **PostgreSQL** | Persistent storage, full-text search, migrations, connection pooling |

There is no Redis, no Kafka, no message queue, no microservice mesh, no container orchestration requirement. This simplifies deployment, reduces operational burden, and eliminates an entire category of infrastructure failures.

### Why Rust

- **Performance** — Zero-cost abstractions, no garbage collector, predictable latency
- **Memory safety** — No null pointer exceptions, no buffer overflows, no data races
- **Single binary** — No runtime dependencies, no interpreter, no JVM
- **Async I/O** — Tokio runtime for non-blocking I/O, efficient connection handling
- **SQLx** — Compile-time checked SQL queries with async connection pooling

## Extension System

Extensions add capabilities to the platform by implementing core traits. The extension system supports:

| Extension Type | What It Adds |
|---------------|-------------|
| **Routes** | HTTP endpoints for APIs and pages |
| **Assets** | CSS, JavaScript, images, fonts for the web interface |
| **CLI commands** | New commands and domains in the CLI |
| **Hooks** | Event handlers that fire on platform events (tool calls, logins, etc.) |
| **Data providers** | Custom data sources for agents and skills |
| **Rendering** | Template engines and content renderers |
| **Jobs** | Background tasks and scheduled operations |
| **Schemas** | Database migrations and schema extensions |

All extensions compile into the single binary. There is no plugin hot-loading at runtime — this is a deliberate design choice for security and predictability. When you add an extension, you rebuild the binary.

## Profile-Based Deployment

The same binary runs in every environment. Profiles control the configuration:

```
.systemprompt/profiles/
  local/
    profile.yaml      # Development settings
  production/
    profile.yaml      # Production settings
```

### What Profiles Control

| Setting | Local | Production |
|---------|-------|------------|
| **Host** | `127.0.0.1` | `0.0.0.0` |
| **Port** | `8080` | `8080` |
| **HTTPS** | Disabled | Enabled |
| **CORS** | Permissive | Strict (origin-locked) |
| **Security headers** | Disabled | Full (HSTS, frame deny, etc.) |
| **Log level** | Debug | Normal (JSON output) |
| **Rate limits** | Disabled | Enabled with tier multipliers |
| **Database** | Local PostgreSQL | Production PostgreSQL with external access |
| **Secrets** | File-based | Environment variable source |

Switching environments requires only a different profile — no code changes, no rebuild.

## Stateless Design

The platform stores no session state in the application layer. Every request is independently authenticated and authorized.

| Design Decision | Implication |
|----------------|-------------|
| **JWT authentication** | Tokens carry identity and claims. No server-side session store. |
| **No session affinity** | Any request can hit any instance. Load balancers need no sticky sessions. |
| **Database as state** | All persistent state lives in PostgreSQL. Application instances are interchangeable. |
| **Horizontal scaling** | Add instances behind a load balancer. No coordination between instances. |

This means scaling from 1 instance to many requires zero application changes — only infrastructure changes (more containers, bigger database).

## Security Headers

The production profile enables a full suite of security headers by default:

| Header | Value | Purpose |
|--------|-------|---------|
| **HSTS** | `max-age=63072000; includeSubDomains; preload` | Force HTTPS for 2 years, including all subdomains |
| **X-Frame-Options** | `DENY` | Prevent clickjacking — page cannot be embedded in frames |
| **X-Content-Type-Options** | `nosniff` | Prevent MIME type sniffing attacks |
| **Referrer-Policy** | `strict-origin-when-cross-origin` | Limit referrer information leakage |
| **Permissions-Policy** | `camera=(), microphone=(), geolocation=()` | Disable browser APIs not needed by the platform |

These headers are configured in the production profile and applied to every HTTP response.

## Self-Hosted Deployment

The platform runs in your infrastructure. There is no SaaS component, no external API dependency for core operations, and no data leaving your network unless you configure external AI providers.

| Aspect | Detail |
|--------|--------|
| **Deployment** | Container or embedded binary, non-root user, health checks |
| **Network** | Runs inside your VPC/network boundary |
| **Compliance** | Your data stays in your infrastructure, under your compliance controls |
| **Updates** | Pull new binary, run migrations, restart |
| **Source access** | Full source code with modification rights |

### Production Container

The production Docker image runs as a non-root user with a health check endpoint. The binary, services configuration, web assets, and storage are all packaged into a single container image.

```
/app/
  bin/systemprompt        # The single binary
  services/               # YAML and Markdown configuration
  web/                    # Compiled web assets
  storage/                # Files, CSS, uploads
```
