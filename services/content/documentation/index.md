---
title: "Documentation"
description: "Technical documentation for systemprompt.io - an embedded Rust library for building production AI infrastructure"
author: "systemprompt.io"
slug: ""
keywords: "systemprompt, documentation, ai agents, mcp, a2a, rust, production infrastructure"
image: "/files/images/docs/overview.svg"
kind: "guide"
public: true
tags: ["documentation", "overview"]
published_at: "2025-01-27"
updated_at: "2026-01-31"
after_reading_this:
  - "Navigate to the correct documentation section for your task"
  - "Identify which code files implement each major component"
  - "Access relevant playbooks for hands-on operations"
related_playbooks:
  - title: "Getting Started Guide"
    url: "/playbooks/guide-start"
  - title: "Architecture Overview"
    url: "/playbooks/build-architecture"
  - title: "Session Management"
    url: "/playbooks/cli-session"
  - title: "Configuration Management"
    url: "/playbooks/cli-config"
  - title: "Service Lifecycle"
    url: "/playbooks/cli-services"
  - title: "Agent Operations"
    url: "/playbooks/cli-agents"
  - title: "MCP Server Registration"
    url: "/playbooks/cli-plugins"
  - title: "Cloud Setup"
    url: "/playbooks/cli-cloud"
  - title: "Deployment Workflow"
    url: "/playbooks/cli-deploy"
  - title: "Configuration Sync"
    url: "/playbooks/cli-sync"
  - title: "Extension Development"
    url: "/playbooks/build-extension-checklist"
  - title: "MCP Server Development"
    url: "/playbooks/build-mcp-checklist"
related_code:
  - title: "Application Entry Point"
    url: "https://github.com/systempromptio/systemprompt-template/blob/main/src/main.rs#L1-L10"
  - title: "Extension Registration"
    url: "https://github.com/systempromptio/systemprompt-core/blob/main/crates/shared/extension/src/lib.rs"
  - title: "Force Extension Link (prevents LTO stripping)"
    url: "https://github.com/systempromptio/systemprompt-template/blob/main/src/lib.rs#L19-L20"
  - title: "Profile Configuration"
    url: "https://github.com/systempromptio/systemprompt-template/blob/main/.systemprompt/profiles/local/profile.yaml"
  - title: "Bootstrap Sequence"
    url: "https://github.com/systempromptio/systemprompt-template/blob/main/extensions/mcp/systemprompt/src/main.rs#L14-L16"
  - title: "Config Loader"
    url: "https://github.com/systempromptio/systemprompt-template/blob/main/extensions/web/src/config_loader.rs#L9-L47"
  - title: "Master Config"
    url: "https://github.com/systempromptio/systemprompt-template/blob/main/services/config/config.yaml"
  - title: "AI Provider Settings"
    url: "https://github.com/systempromptio/systemprompt-template/blob/main/services/ai/config.yaml"
  - title: "Content Ingestion Job"
    url: "https://github.com/systempromptio/systemprompt-template/blob/main/extensions/web/src/jobs/ingestion.rs#L10-L98"
  - title: "Web Extension Implementation"
    url: "https://github.com/systempromptio/systemprompt-template/blob/main/extensions/web/src/extension.rs#L39-L280"
  - title: "MCP Server Manifest"
    url: "https://github.com/systempromptio/systemprompt-template/blob/main/extensions/mcp/systemprompt/manifest.yaml"
  - title: "Cloud Config Section"
    url: "https://github.com/systempromptio/systemprompt-template/blob/main/.systemprompt/profiles/local/profile.yaml#L66-L73"
  - title: "CLI Execution with Auth"
    url: "https://github.com/systempromptio/systemprompt-template/blob/main/extensions/mcp/systemprompt/src/cli.rs#L28-L47"
related_docs:
  - title: "Getting Started"
    url: "/documentation/getting-started"
  - title: "Coding Standards"
    url: "/documentation/getting-started/coding-standards"
  - title: "Installation"
    url: "/documentation/installation"
  - title: "Licensing"
    url: "/documentation/licensing"
  - title: "Playbooks"
    url: "/documentation/playbooks"
  - title: "Configuration Overview"
    url: "/documentation/config"
  - title: "Profiles"
    url: "/documentation/config/profiles"
  - title: "Database Setup"
    url: "/documentation/config/database"
  - title: "Secrets Management"
    url: "/documentation/config/secrets"
  - title: "Services Overview"
    url: "/documentation/services"
  - title: "Agents"
    url: "/documentation/services/agents"
  - title: "MCP Servers"
    url: "/documentation/services/mcp"
  - title: "AI Providers"
    url: "/documentation/services/ai"
  - title: "Content"
    url: "/documentation/services/content"
  - title: "Scheduler"
    url: "/documentation/services/scheduler"
  - title: "Web Configuration"
    url: "/documentation/services/web"
  - title: "Extensions Overview"
    url: "/documentation/extensions"
  - title: "Rust Extensions"
    url: "/documentation/extensions/rust"
  - title: "Web Extensions"
    url: "/documentation/extensions/web"
  - title: "CLI Extensions"
    url: "/documentation/extensions/cli"
  - title: "MCP Extensions"
    url: "/documentation/extensions/mcp"
  - title: "Background Jobs"
    url: "/documentation/extensions/jobs"
  - title: "Deployment"
    url: "/documentation/config/deployment"
  - title: "Sync"
    url: "/documentation/config/sync"
  - title: "Domains"
    url: "/documentation/config/domains"
links:
  - title: "GitHub Repository"
    url: "https://github.com/systempromptio/systemprompt-core"
  - title: "Project Template"
    url: "https://github.com/systempromptio/systemprompt-template"
  - title: "Releases"
    url: "https://github.com/systempromptio/systemprompt-core/releases"
---

# systemprompt.io Documentation

systemprompt.io is an embedded Rust library for building production AI infrastructure. Clone the template, wrap your logic around it, and the CLI handles the rest—local development, cloud deployment, and everything in between.

**Quick start:**

```bash
# Clone the template
gh repo create my-ai --template systempromptio/systemprompt-template --clone

# Build
cargo build --release

# Run
just start
```

The same CLI that works locally also works in production. Your AI clients connect via MCP, your agents coordinate via A2A, and everything is authenticated and auditable out of the box.

## Documentation Sections

| Section | Purpose |
|---------|---------|
| **Getting Started** | Quick start, coding standards, playbooks |
| Config | Profiles, database, secrets |
| Services | Agents, MCP, AI providers |
| Extensions | Custom Rust code |
| Cloud | Deploy, sync, domains |

---

## Getting Started

The Getting Started section is your entry point for all development work.

| Page | Description |
|------|-------------|
| [Getting Started Overview](/documentation/getting-started) | Quick start guide, project structure, first steps |
| [Coding Standards](/documentation/getting-started/coding-standards) | Rust patterns, mandatory requirements, anti-patterns |
| [Installation](/documentation/installation) | Detailed installation and database setup |
| [Licensing](/documentation/licensing) | BSL-1.1 license, pricing, and what you can build |
| [Playbooks](/documentation/playbooks) | Machine-readable instruction sets |

**For AI agents and automated workflows**, start with the playbooks:

```bash
systemprompt core playbooks show guide_start
systemprompt core playbooks show guide_coding-standards
```

The CLI binary is the primary interface. On startup, `main.rs` calls `__force_extension_link()` which references each extension's `PREFIX` constant - this prevents the linker from stripping extensions during LTO optimization. The `cli::run()` function then initializes the library with all registered extensions.

---

## Config

The `.systemprompt/` directory contains deployment and environment configuration, kept separate from runtime services. Profiles define database connections, server settings, JWT secrets, and cloud credentials. The `ProfileBootstrap::init()` function in the MCP server entry point loads the active profile, followed by `SecretsBootstrap::init()` for credentials and `Config::init()` for service configuration.

The Config section includes the Configuration Overview (directory structure and configuration flow), Profiles (environment-specific settings for local, staging, and production), Database (PostgreSQL setup options), and Secrets (API keys and credentials management).

---

## Services

The `services/` directory contains YAML configuration for all runtime components. No Rust code lives here—only configuration files and Markdown content. The `config_loader.rs` module loads navigation from `services/web/config/navigation.yaml`, homepage sections from `homepage.yaml`, and feature pages from `features/*.yaml`. Services are hot-reloadable: edit YAML, sync to database, changes take effect without restart.

The Services section covers Agents (A2A protocol cards and skill assignments), MCP (tool server hosting with OAuth scopes), AI (provider configuration for Anthropic, OpenAI, Gemini), Content (blog posts, documentation, content indexing), Scheduler (background jobs with cron scheduling), and Web (branding, navigation, theme configuration).

---

## Extensions

Extensions add custom functionality via Rust code in `extensions/`. The library uses the `inventory` crate for compile-time registration—call `register_extension!(MyExtension)` and your extension is automatically discovered at startup. The `Extension` trait defines hooks for page data providers, component renderers, database schemas, API routes, background jobs, and required assets.

The Extensions section includes the Overview (extension types and discovery mechanism), Rust Extensions (API routes, database schemas, background jobs), Web Extensions (page data providers, static generation, assets), CLI Extensions (custom CLI commands with clap integration), MCP Extensions (tool servers implementing the MCP protocol), and Background Jobs (scheduled tasks and async processing).

---

## Cloud

systemprompt.io Cloud provides managed infrastructure for production deployments. The cloud configuration in `profile.yaml` defines `credentials_path` and `tenants_path` for authentication. The `cli.rs` module executes CLI commands with the auth token injected via environment variables (`SYSTEMPROMPT_AUTH_TOKEN`), enabling secure cloud operations from MCP servers.

Cloud documentation covers Deployment (one-command deploy with CI/CD integration), Sync (push/pull configuration between local and cloud), Domains (custom domains with automatic TLS via Let's Encrypt), and Secrets (cloud credential management and rotation).

---

## External Resources

The source code and project template are available on GitHub. See the Related Code and Links sections for direct access to repositories and releases. All playbooks are accessible via the CLI with `systemprompt core playbooks list`.
