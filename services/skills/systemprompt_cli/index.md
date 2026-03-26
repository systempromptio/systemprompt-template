---
name: "systemprompt CLI"
description: "Comprehensive guide to systemprompt CLI commands, domains, and workflows"
---

# systemprompt CLI Reference

You use the systemprompt CLI to manage skills, content, infrastructure, analytics, and more. All operations follow the pattern:

```
systemprompt <domain> <subcommand> [args]
```

Use `--help` on any command to discover available subcommands and options. Always explore with `--help` before attempting a command you are unsure about.

## Domains

| Domain | Purpose | Key Subcommands |
|--------|---------|-----------------|
| `core` | Skills, content, files, contexts, plugins, hooks | `skills`, `content`, `files`, `contexts`, `plugins`, `hooks`, `agents`, `artifacts` |
| `infra` | Infrastructure management | `services`, `db`, `jobs`, `logs` |
| `admin` | Administration | `users`, `agents`, `config`, `setup`, `session` |
| `cloud` | Cloud deployment and sync | `auth`, `deploy`, `sync`, `secrets`, `tenant`, `domain`, `profile`, `status`, `restart`, `init`, `dockerfile`, `db` |
| `analytics` | Metrics and reporting | `overview`, `conversations`, `agents`, `tools`, `requests`, `sessions`, `content`, `traffic`, `costs` |
| `web` | Web service configuration | `content-types`, `templates`, `assets`, `sitemap`, `validate` |
| `plugins` | Extensions and MCP servers | `list`, `show`, `run`, `validate`, `config`, `capabilities`, `mcp` |
| `build` | Build workspace and extensions | `core`, `mcp` |

## Quick Start

```bash
# Discover commands
systemprompt --help
systemprompt core --help
systemprompt core skills --help

# List and view skills
systemprompt core skills list
systemprompt core skills show <skill_id>

# View analytics
systemprompt analytics overview

# Check services
systemprompt infra services status
```

## Skills Management

| Command | Purpose |
|---------|---------|
| `systemprompt core skills list` | List all configured skills |
| `systemprompt core skills show <id>` | Show skill details and instructions |
| `systemprompt core skills create` | Create a new skill |
| `systemprompt core skills edit <id>` | Edit skill configuration |
| `systemprompt core skills delete <id>` | Delete a skill |
| `systemprompt core skills status` | Show database sync status |
| `systemprompt core skills sync` | Sync skills between disk and database |

## Content Management

| Command | Purpose |
|---------|---------|
| `systemprompt core content list` | List content items |
| `systemprompt core content show <id>` | Show content details |
| `systemprompt core files list` | List uploaded files |
| `systemprompt core files upload <path>` | Upload a file |

## Agent Management

| Command | Purpose |
|---------|---------|
| `systemprompt core agents list` | List configured agents |
| `systemprompt core agents show <id>` | Show agent details |
| `systemprompt admin agents list` | List agents (admin view) |

## Infrastructure

| Command | Purpose |
|---------|---------|
| `systemprompt infra services status` | Check service health |
| `systemprompt infra services start` | Start services |
| `systemprompt infra services stop` | Stop services |
| `systemprompt infra db status` | Database status |
| `systemprompt infra jobs list` | List background jobs |
| `systemprompt infra logs view --level error --since 1h` | View recent errors |

## Analytics

| Command | Purpose |
|---------|---------|
| `systemprompt analytics overview` | Dashboard overview |
| `systemprompt analytics conversations` | Conversation analytics |
| `systemprompt analytics agents` | Agent performance |
| `systemprompt analytics tools` | Tool usage |
| `systemprompt analytics costs` | Cost analytics |
| `systemprompt analytics requests` | AI request analytics |
| `systemprompt analytics sessions` | Session analytics |
| `systemprompt analytics content` | Content performance |
| `systemprompt analytics traffic` | Traffic analytics |

## Plugin and MCP Server Management

| Command | Purpose |
|---------|---------|
| `systemprompt plugins list` | List all extensions |
| `systemprompt plugins show <id>` | Show extension details |
| `systemprompt plugins run <ext> <command>` | Run a CLI extension command |
| `systemprompt plugins mcp list` | List MCP servers |
| `systemprompt plugins mcp logs <server>` | View MCP server logs |

## Debugging Workflow

1. **Find errors** -- `systemprompt infra logs view --level error --since 1h`
2. **Find failed requests** -- `systemprompt infra logs request list --limit 10`
3. **Get full context** -- `systemprompt infra logs audit <request-id> --full`
4. **Check MCP logs** -- `systemprompt plugins mcp logs <server-name>`

## Discovery Pattern

When you need to perform a task, always use `--help` to find the right command:

```bash
systemprompt --help                    # Top-level domains
systemprompt core --help               # Core subcommands
systemprompt core skills --help        # Skills subcommands
systemprompt core skills create --help # Create options
```

Never guess a command. If `--help` does not show the subcommand you expect, it does not exist. Use the available commands shown in the help output.

## Related Skills

For detailed operations in specific areas, load the relevant specialized skill:

| Skill | Command |
|-------|---------|
| Log Management | `core skills show systemprompt_admin_logs` |
| User Management | `core skills show systemprompt_admin_user_management` |
| Agent Management | `core skills show systemprompt_admin_agent_management` |
| Analytics | `core skills show systemprompt_admin_analytics` |
| Service Management | `core skills show systemprompt_admin_services` |
| Job Scheduling | `core skills show systemprompt_admin_jobs` |
| Database Management | `core skills show systemprompt_admin_database` |
| Skill Creation | `core skills show systemprompt_admin_skill_creation` |
