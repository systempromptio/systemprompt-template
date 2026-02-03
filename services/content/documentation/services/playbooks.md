---
title: "Playbooks Service"
description: "Playbooks provide machine-executable instruction sets for agents and users. They define deterministic, testable workflows for CLI operations, development, and content creation."
author: "SystemPrompt Team"
slug: "services/playbooks"
keywords: "playbooks, instructions, workflows, cli, automation, agents"
image: "/files/images/docs/services-playbooks.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Playbooks Service

**TL;DR:** Playbooks are machine-executable instruction sets that define exactly how to perform tasks. They contain CLI commands, validation steps, and quick reference tables. Agents read playbooks to know how to execute operations, and users read them to learn the correct procedures.

## The Problem

Agents need to know how to perform operations in SystemPrompt. They need exact commands, not vague instructions. "Set up authentication" is not helpful. "Run `systemprompt admin session login` and verify with `systemprompt admin session status`" is actionable.

Without playbooks, agents would need to discover commands through trial and error. Users would need to piece together procedures from scattered documentation. Inconsistent approaches would lead to errors and confusion.

Playbooks solve this by providing deterministic instructions that both agents and users can follow. When an agent needs to deploy an application, it reads the deploy playbook. When a user needs to create a blog post, they follow the blog content playbook.

## How Playbooks Work

Playbooks are Markdown files stored in `services/playbook/`. Each playbook focuses on a single domain or task. The filename becomes the playbook ID through a simple transformation: `cli/agents.md` becomes `cli_agents`.

Playbooks follow a strict structure: frontmatter with title and description, an H1 title, command sections with JSON-formatted commands, and a quick reference table at the end. This structure makes playbooks parseable by both humans and agents.

When an agent needs to perform an operation, it requests the relevant playbook through the CLI. The playbook provides the exact commands to execute, in order, with expected outputs and error handling.

## Directory Structure

```
services/playbook/
├── guide/           # Getting started, meta playbooks
│   ├── start.md     # guide_start - Entry point for all agents
│   ├── playbook.md  # guide_playbook - How to write playbooks
│   ├── recipes.md   # guide_recipes - Common patterns
│   └── documentation.md  # guide_documentation - Doc standards
├── cli/             # CLI operation playbooks
│   ├── agents.md    # cli_agents - Agent management
│   ├── services.md  # cli_services - Service lifecycle
│   ├── session.md   # cli_session - Authentication
│   ├── skills.md    # cli_skills - Skill management
│   ├── jobs.md      # cli_jobs - Background jobs
│   ├── deploy.md    # cli_deploy - Deployment
│   └── ...          # Additional CLI playbooks
├── build/           # Development playbooks
│   ├── architecture.md    # build_architecture - System design
│   ├── extension-checklist.md  # build_extension-checklist
│   ├── mcp-checklist.md   # build_mcp-checklist
│   ├── rust-standards.md  # build_rust-standards
│   └── ...          # Additional build playbooks
├── content/         # Content creation playbooks
│   ├── blog.md      # content_blog - Blog posts
│   ├── linkedin.md  # content_linkedin - LinkedIn posts
│   ├── twitter.md   # content_twitter - Twitter threads
│   └── ...          # Platform-specific playbooks
└── validation/      # Validation playbooks
    ├── cli.md       # validation_cli - CLI validation
    └── ...          # Additional validation playbooks
```

## Playbook Categories

### Guide Playbooks (guide_*)

Guide playbooks are the starting point. Every agent should read `guide_start` first. These playbooks explain the system, the playbook format itself, and common patterns.

- `guide_start` - Entry point for all agents, explains how to use playbooks
- `guide_playbook` - How to write and maintain playbooks
- `guide_documentation` - Documentation authoring standards

### CLI Playbooks (cli_*)

CLI playbooks cover all command-line operations. Each focuses on a specific domain: agents, services, content, jobs, and so on. These are the most commonly used playbooks.

- `cli_agents` - Creating, managing, and monitoring agents
- `cli_services` - Starting, stopping, and configuring services
- `cli_session` - Authentication and session management
- `cli_skills` - Skill creation and assignment
- `cli_deploy` - Deployment to cloud environments

### Build Playbooks (build_*)

Build playbooks define development standards and procedures. They cover architecture decisions, extension development, and code review processes.

- `build_architecture` - System architecture overview
- `build_extension-checklist` - Checklist for new extensions
- `build_mcp-checklist` - Checklist for new MCP servers
- `build_rust-standards` - Rust coding standards

### Content Playbooks (content_*)

Content playbooks guide content creation for various platforms. Each defines the format, style, and publishing workflow for its target platform.

- `content_blog` - SystemPrompt blog posts
- `content_linkedin` - LinkedIn posts and articles
- `content_twitter` - Twitter threads

## Accessing Playbooks

### Through CLI

```bash
# List all playbooks
systemprompt core playbooks list

# List by category
systemprompt core playbooks list --category cli

# Show a specific playbook
systemprompt core playbooks show cli_agents

# Show raw markdown (for piping)
systemprompt core playbooks show cli_agents --raw
```

### Through Web Interface

Playbooks are also available at `/playbooks/<playbook_id>` on the web interface. The content service indexes playbooks and makes them searchable.

### Through MCP

Agents access playbooks through MCP tool calls:

```json
{ "command": "core playbooks show cli_agents" }
```

## Syncing Playbooks

Playbooks are stored both on disk and in the database. Changes to playbook files should be synced to the database:

```bash
# Sync from disk to database
systemprompt core playbooks sync --direction to-db -y

# Sync from database to disk
systemprompt core playbooks sync --direction from-db

# Verify a playbook after sync
systemprompt core playbooks show cli_agents
```

The content publishing job also syncs playbooks as part of the content indexing process.

## Playbook Structure

Every playbook follows this structure:

<details>
<summary>Required playbook structure</summary>

```markdown
---
title: "Playbook Title"
description: "Single sentence. What it does."
keywords:
  - keyword1
  - keyword2
---

# Playbook Title

Single-line description matching frontmatter.

> **Help**: `{ "command": "..." }` via `systemprompt_help`
> **Requires**: Prerequisites (if any) -> See [Playbook](path.md)

---

## Section Name

Commands in JSON format:

{ "command": "domain subcommand args" }

---

## Quick Reference

| Task | Command |
|------|---------|
| Do X | `domain subcommand` |
```

</details>

The key elements are:
- **Frontmatter** with title, description, and keywords
- **Help block** showing how to get command help
- **Requires block** listing prerequisites with playbook links
- **Command sections** with JSON-formatted commands
- **Quick reference table** summarizing all commands

## Service Relationships

- **Content service** indexes playbooks and makes them searchable
- **Web service** renders playbooks on the web interface
- **MCP servers** expose playbooks to agents through tool calls
- **Agents** read playbooks to learn how to perform operations
- **Scheduler** runs content publishing which syncs playbooks

## CLI Reference

| Command | Description |
|---------|-------------|
| `systemprompt core playbooks list` | List playbooks |
| `systemprompt core playbooks show <id>` | Show full playbook content |
| `systemprompt core playbooks create` | Create new playbook |
| `systemprompt core playbooks edit <id>` | Edit playbook configuration |
| `systemprompt core playbooks delete <id>` | Delete a playbook |
| `systemprompt core playbooks sync` | Sync playbooks between disk and database |

See `systemprompt core playbooks <command> --help` for detailed options.

## Troubleshooting

**Playbook not found** -- Verify the playbook ID. IDs are formed from the path: `cli/agents.md` becomes `cli_agents`. Run `core playbooks list` to see all available playbooks.

**Playbook not updated after edit** -- Sync changes to the database with `core playbooks sync --direction to-db -y`. The content publish job also syncs playbooks.

**Command in playbook fails** -- The playbook may be outdated. Verify the command with `--help` and update the playbook file. Then sync to database.