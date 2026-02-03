---
title: "Getting Started"
description: "Quick start guide for systemprompt.io - from clone to running service in minutes"
author: "systemprompt.io"
slug: "getting-started"
keywords: "getting started, quick start, installation, setup, first steps"
image: "/files/images/docs/getting-started.svg"
kind: "guide"
public: true
tags: ["getting-started", "overview"]
published_at: "2025-01-27"
updated_at: "2026-02-02"
after_reading_this:
  - "Clone the template and build a running binary"
  - "Understand the three-directory structure"
  - "Navigate to the appropriate documentation for your task"
  - "Use playbooks for all operations"
related_playbooks:
  - title: "Getting Started Guide"
    url: "/playbooks/guide-start"
  - title: "Coding Standards Guide"
    url: "/playbooks/guide-coding-standards"
  - title: "Session Management"
    url: "/playbooks/cli-session"
  - title: "Cloud Setup"
    url: "/playbooks/cli-cloud"
  - title: "Architecture Overview"
    url: "/playbooks/build-architecture"
related_code:
  - title: "systemprompt-template"
    url: "https://github.com/systempromptio/systemprompt-template"
  - title: "CLAUDE.md (AI Instructions)"
    url: "https://github.com/systempromptio/systemprompt-template/blob/main/CLAUDE.md"
  - title: "Application Entry Point"
    url: "https://github.com/systempromptio/systemprompt-template/blob/main/src/main.rs"
related_docs:
  - title: "Installation"
    url: "/documentation/installation"
  - title: "Coding Standards"
    url: "/documentation/getting-started/coding-standards"
  - title: "Configuration Overview"
    url: "/documentation/config"
  - title: "Services Overview"
    url: "/documentation/services"
  - title: "Extensions Overview"
    url: "/documentation/extensions"
links:
  - title: "GitHub Repository"
    url: "https://github.com/systempromptio/systemprompt-template"
  - title: "Rust Installation"
    url: "https://rustup.rs"
---

# Getting Started

systemprompt.io is an embedded Rust library for building production AI infrastructure. Clone the template, wrap your logic around it, and the CLI handles the rest.

## Prerequisites

- **Rust 1.75+** — Install from rustup.rs
- **Git** — For cloning repositories
- **PostgreSQL** — The only external dependency

```bash
rustc --version    # Should output 1.75.0 or higher
```

## Quick Start

### 1. Clone the Template

```bash
gh repo create my-ai --template systempromptio/systemprompt-template --clone --private
cd my-ai
git submodule update --init --recursive
```

### 2. Build

```bash
SQLX_OFFLINE=true cargo build --release -p systemprompt-cli
```

### 3. Setup Profile

```bash
systemprompt cloud auth login
systemprompt cloud tenant create --type local
systemprompt cloud profile create local
systemprompt infra db migrate
```

### 4. Start Services

```bash
just start
```

Visit `http://localhost:8080` to see your homepage.

## Project Structure

systemprompt.io projects have three core directories:

| Directory | Purpose | Contents |
|-----------|---------|----------|
| `.systemprompt/` | Credentials & cloud management | profiles, secrets, tenant config |
| `services/` | Config as code | agents, skills, content, web config |
| `extensions/` | Rust crates | custom code, API routes, MCP servers |

### .systemprompt/ — Credentials

Your personal credential store. Gitignored by default.

```text
.systemprompt/
├── credentials.json     # Cloud API credentials
├── tenants.json         # Registry of tenants
└── profiles/
    └── local/
        ├── profile.yaml # Environment settings
        └── secrets.json # DATABASE_URL, API keys
```

### services/ — Configuration

YAML and Markdown configuration. No Rust code here.

| Directory | Purpose |
|-----------|---------|
| `services/agents/` | AI agent definitions |
| `services/mcp/` | MCP server configs |
| `services/skills/` | Reusable skills |
| `services/content/` | Blog, docs (Markdown) |
| `services/web/` | Theme, navigation |
| `services/playbook/` | Operational playbooks |

### extensions/ — Rust Code

Custom Rust crates extending the core framework.

```text
extensions/
├── web/     # Web extension (API routes, schemas, jobs)
├── cli/     # CLI extensions (custom commands)
└── mcp/     # MCP server extensions (tool servers)
```

See the Coding Standards documentation for patterns and requirements.

## Playbooks

**Everything is done through playbooks.** Playbooks are machine-readable instruction sets.

```bash
# Required first read
systemprompt core playbooks show guide_start

# List all playbooks
systemprompt core playbooks list

# Read a specific playbook
systemprompt core playbooks show <playbook_id>
```

| Task | Playbook |
|------|----------|
| First read | `guide_start` |
| Coding standards | `guide_coding-standards` |
| CLI session | `cli_session` |
| Build extension | `build_extension-checklist` |
| Build MCP server | `build_mcp-checklist` |

## Next Steps

| Goal | Documentation |
|------|---------------|
| Detailed installation | Installation |
| Write code | Coding Standards |
| Configure services | Configuration Overview |
| Build extensions | Extensions Overview |
| Deploy to cloud | Cloud Deployment |

## Verification

```bash
# Check service status
systemprompt infra services status

# Check database connection
systemprompt infra db status

# List available agents
systemprompt admin agents list
```
