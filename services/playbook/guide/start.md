---
title: "START HERE - Playbook Guide"
description: "REQUIRED READING. Read this playbook FIRST before any task. Master index linking to all playbooks."
author: "SystemPrompt"
slug: "guide-start"
keywords: "start, index, required, first, master, guide, playbooks, getting-started"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# START HERE - Playbook Guide

**THIS IS REQUIRED READING.** Read this playbook FIRST before starting ANY task.

> **Read playbooks**: `systemprompt core playbooks show <playbook_id>`

---

## How SystemPrompt Works

### The Flow

```
+------------------------------------------------------------------+
|                         USER                                      |
|                           |                                       |
|                           v                                       |
|                    +-------------+                                |
|                    |  PLAYBOOK   |  <-- User reads playbook       |
|                    +------+------+      to learn how to           |
|                           |             interact with agent       |
|                           v                                       |
|                    +-------------+                                |
|                    |    CLI      |  <-- User sends task           |
|                    +------+------+      via CLI to agent          |
|                           |                                       |
|                           v                                       |
|  +------------------------------------------------------------+  |
|  |                       AGENT                                 |  |
|  |                         |                                   |  |
|  |           +-------------+-------------+                     |  |
|  |           v             v             v                     |  |
|  |      +--------+   +----------+   +---------+               |  |
|  |      | SKILLS |   | MCP      |   | TOOLS   |               |  |
|  |      |        |   | SERVERS  |   |         |               |  |
|  |      +--------+   +----------+   +---------+               |  |
+------------------------------------------------------------------+
```

### 1. Agents Execute Tasks

Agents are AI workers that perform tasks using:
- **Skills** - Reusable capabilities (e.g., "write blog post", "analyze code")
- **MCP Servers** - External tools and integrations
- **Tools** - Built-in functions available to the agent

### 2. Users Interact via CLI + Playbooks

**Users should NOT guess how to interact with agents.** Instead:

1. **Find the agent's playbook** - Each agent has a corresponding playbook
2. **Read the playbook** - Learn the correct commands and workflow
3. **Use the CLI** - Send tasks to agents following the playbook

| Agent | Playbook | Purpose |
|-------|----------|---------|
| content | `content_blog`, `content_linkedin`, etc. | Content creation |
| blog | `content_blog` | Blog post generation |
| Any agent | `cli_agents` | General agent interaction |

### IMPORTANT: Interacting with Agents

**If you are asked to interact with an agent, you MUST:**

1. **Find the playbook for that agent**:
   ```bash
   systemprompt core playbooks list --category content   # For content agents
   systemprompt core playbooks list                      # List all playbooks
   ```

2. **Read the playbook**:
   ```bash
   systemprompt core playbooks show <playbook_id>
   ```

3. **Follow the playbook instructions** - The playbook contains:
   - How to send messages to the agent
   - Required parameters and format
   - Expected responses
   - Error handling

4. **Use the CLI to interact** (example from playbook):
   ```bash
   systemprompt admin agents message <agent_name> -m "Your task here" --blocking
   ```

**DO NOT improvise agent interactions. The playbook has the correct commands.**

### 3. Edit & Update via CLI

Skills, agent instructions, and playbooks can all be edited using the CLI:

```bash
# AGENTS - Edit agent configuration and instructions
systemprompt admin agents list                    # List all agents
systemprompt admin agents show <name>             # View agent config
systemprompt admin agents edit <name>             # Edit agent config
systemprompt admin agents create                  # Create new agent

# SKILLS - Edit skill definitions
systemprompt core skills list                     # List all skills
systemprompt core skills show <name>              # View skill details
systemprompt core skills edit <name>              # Edit skill
systemprompt core skills create                   # Create new skill
systemprompt core skills sync --direction to-db -y  # Sync to database

# PLAYBOOKS - Edit playbook instructions
systemprompt core playbooks list                  # List all playbooks
systemprompt core playbooks show <id>             # View playbook
# Edit playbook files directly in: services/playbook/
systemprompt core playbooks sync --direction to-db -y  # Sync to database
```

### File Locations

| Type | Location | Sync Command |
|------|----------|--------------|
| Agents | `services/agents/*.yaml` | Auto-loaded on restart |
| Skills | `services/skills/*.yaml` | `systemprompt core skills sync --direction to-db -y` |
| Playbooks | `services/playbook/**/*.md` | `systemprompt core playbooks sync --direction to-db -y` |

---

## MANDATORY: Read Before Any Task

**Agents MUST follow this workflow for EVERY task:**

1. **Read this playbook first** - `systemprompt core playbooks show guide_start`
2. **Identify the task domain** - Find the relevant playbook below
3. **Read the domain playbook** - `systemprompt core playbooks show <playbook_id>`
4. **Follow the playbook steps exactly** - Do not improvise
5. **Verify success** - Use the playbook's verification steps

**DO NOT skip steps. DO NOT guess commands. ALWAYS use playbooks. ALWAYS fix playbooks immediately if bugs are encountered**

---

## Guide Pathways

**Guides link to detailed playbooks. Start with the appropriate guide for your task.**

```
guide_start (this playbook)
    |
    +-> CLI Operations
    |       +-> cli_session, cli_agents, cli_services, cli_database...
    |
    +-> Building Code
    |       +-> guide_coding-standards
    |               +-> build_rust-standards (Rust patterns)
    |               +-> build_extension-checklist (Extensions)
    |               +-> build_mcp-checklist (MCP servers)
    |
    +-> Content Creation
    |       +-> content_blog, content_linkedin, content_twitter...
    |
    +-> Discord Integration
    |       +-> guide_discord
    |               +-> cli_discord (CLI commands)
    |               +-> cli_jobs (Gateway job)
    |
    +-> Writing Playbooks
            +-> guide_playbook
                    +-> guide_documentation
```

| Task Type | Start With | Then Read |
|-----------|------------|-----------|
| CLI commands | `guide_start` | `cli_*` playbooks |
| Writing Rust code | `guide_coding-standards` | `build_rust-standards` |
| Building extensions | `guide_coding-standards` | `build_extension-checklist` |
| Building MCP servers | `guide_coding-standards` | `build_mcp-checklist` |
| Creating content | `guide_start` | `content_*` playbooks |
| Discord integration | `guide_discord` | `cli_discord`, `cli_jobs` |
| Writing playbooks | `guide_playbook` | `guide_documentation` |

---

## Master Playbook Index

### Guide (Start Here)

| Playbook ID | Description |
|-------------|-------------|
| `guide_start` | **THIS PLAYBOOK** - Required first read, links to everything |
| `guide_coding-standards` | **Coding standards** - Rust patterns, code style, mandatory for all code |
| `guide_discord` | **Discord integration** - Send/receive messages, gateway setup |
| `guide_playbook` | Playbook authoring standards |
| `guide_recipes` | Quick recipes for common tasks |
| `guide_documentation` | Documentation authoring standards |

### CLI Operations

| Playbook ID | Description | Use For |
|-------------|-------------|---------|
| `cli_session` | Session & authentication | Login, profiles, auth |
| `cli_agents` | Agent management | List, message, configure agents |
| `cli_services` | Service lifecycle | Start, stop, restart services |
| `cli_database` | Database operations | Queries, status, tables |
| `cli_logs` | Log management & SQL queries | View, stream, trace, direct SQL |
| `cli_analytics` | Analytics, tracking & bot detection | Stats, costs, traffic, sessions |
| `cli_cloud` | Cloud setup & management | Auth, tenants, profiles, secrets |
| `cli_jobs` | Job scheduler | List, run scheduled jobs |
| `cli_users` | User management | List, roles, permissions |
| `cli_config` | Configuration | View, update config |
| `cli_files` | File operations | Upload, list files |
| `cli_skills` | Skill management | List, sync skills |
| `cli_contexts` | Context management | Conversation contexts |
| `cli_plugins` | Plugin management | MCP servers, extensions |
| `cli_build` | Build system | Compile, test |
| `cli_deploy` | Deployment | Deploy to environments |
| `cli_sync` | Data sync | Sync between environments |
| `cli_web` | Web frontend | Web service config |

### Build & Development

| Playbook ID | Description | Use For |
|-------------|-------------|---------|
| `build_architecture` | Extension architecture | Understanding the system |
| `build_extension-checklist` | Extension checklist | Building extensions |
| `build_extension-review` | Extension review | Reviewing extensions |
| `build_extension-cli` | CLI extension checklist | Building CLI extensions |
| `build_mcp-checklist` | MCP server checklist | Building MCP servers |
| `build_mcp-review` | MCP server review | Reviewing MCP servers |
| `build_rust-standards` | Rust coding standards | Code style & patterns |
| `build_crate` | Umbrella crate usage | Using systemprompt crate |
| `build_web-content` | Web content | Markdown, frontmatter |
| `build_web-templates` | Web templates | Handlebars, variables |
| `build_web-assets` | Web assets | CSS, JS, fonts |

### Content Creation

| Playbook ID | Description | Use For |
|-------------|-------------|---------|
| `content_blog` | Blog posts | Writing blog content |
| `content_linkedin` | LinkedIn posts | LinkedIn content |
| `content_twitter` | Twitter/X posts | Twitter content |
| `content_medium` | Medium articles | Medium content |
| `content_substack` | Substack posts | Substack content |
| `content_reddit` | Reddit posts | Reddit content |
| `content_hackernoon` | HackerNoon articles | HackerNoon content |

### Domain Operations

| Playbook ID | Description | Use For |
|-------------|-------------|---------|
| `domain_agents-operations` | Agent lifecycle management | Creating, configuring agents |
| `domain_agents-troubleshooting` | Diagnose agent issues | Debugging agents |
| `domain_ai-providers` | AI provider configuration | Setting up Anthropic, OpenAI, Gemini |
| `domain_ai-troubleshooting` | Diagnose AI issues | Debugging AI providers |
| `domain_mcp-servers` | MCP server management | Configuring MCP servers |
| `domain_mcp-troubleshooting` | Diagnose MCP issues | Debugging MCP |
| `domain_content-management` | Content lifecycle | Managing content sources |
| `domain_content-troubleshooting` | Diagnose content issues | Debugging content |
| `domain_skills-development` | Skill creation | Building agent skills |
| `domain_skills-troubleshooting` | Diagnose skill issues | Debugging skills |
| `domain_scheduler-jobs` | Job scheduling | Automating background tasks |
| `domain_analytics-monitoring` | Analytics workflows | Monitoring usage and costs |

---

## How to Use Playbooks

```bash
# List all available playbooks
systemprompt core playbooks list

# Read a specific playbook (full content)
systemprompt core playbooks show guide_start
systemprompt core playbooks show cli_session
systemprompt core playbooks show build_architecture

# Raw markdown output (for piping)
systemprompt core playbooks show cli_agents --raw

# Filter list by category
systemprompt core playbooks list --category cli
systemprompt core playbooks list --category build
systemprompt core playbooks list --category content
```

---

## Task to Playbook Quick Reference

| Task | Read This Playbook |
|------|-------------------|
| Login / authentication | `systemprompt core playbooks show cli_session` |
| Cloud setup | `systemprompt core playbooks show cli_cloud` |
| Start/stop services | `systemprompt core playbooks show cli_services` |
| Send message to agent | `systemprompt core playbooks show cli_agents` |
| Query database | `systemprompt core playbooks show cli_database` |
| View logs / SQL queries | `systemprompt core playbooks show cli_logs` |
| Analytics & tracking | `systemprompt core playbooks show cli_analytics` |
| Deploy application | `systemprompt core playbooks show cli_deploy` |
| Create blog post | `systemprompt core playbooks show content_blog` |
| Build extension | `systemprompt core playbooks show build_extension-checklist` |
| Build MCP server | `systemprompt core playbooks show build_mcp-checklist` |
| Understand architecture | `systemprompt core playbooks show build_architecture` |
| Create new agent | `systemprompt core playbooks show domain_agents-operations` |
| Debug agent issues | `systemprompt core playbooks show domain_agents-troubleshooting` |
| Configure AI providers | `systemprompt core playbooks show domain_ai-providers` |
| Debug AI issues | `systemprompt core playbooks show domain_ai-troubleshooting` |
| Configure MCP servers | `systemprompt core playbooks show domain_mcp-servers` |
| Debug MCP issues | `systemprompt core playbooks show domain_mcp-troubleshooting` |
| Manage content | `systemprompt core playbooks show domain_content-management` |
| Create skills | `systemprompt core playbooks show domain_skills-development` |
| Schedule jobs | `systemprompt core playbooks show domain_scheduler-jobs` |
| Monitor analytics | `systemprompt core playbooks show domain_analytics-monitoring` |

---

## Playbook Structure

Every playbook contains:

- **Title & Description** - What the playbook does
- **Prerequisites** - Required setup (e.g., active session)
- **Step-by-step commands** - Exact commands to run
- **Verification** - How to confirm success
- **Troubleshooting** - Common issues and fixes
- **Quick Reference** - Command summary table

---

## Rules for Agents

1. **ALWAYS read `guide_start` first** - This playbook
2. **ALWAYS read the domain playbook** - Before executing any task
3. **NEVER guess commands** - Only use commands from playbooks
4. **NEVER skip steps** - Follow playbooks in order
5. **ALWAYS verify** - Use verification steps from playbooks
6. **Self-repair playbooks** - If a playbook is wrong, fix it before continuing

---

## MANDATORY: Self-Repair

**If a playbook command fails or doesn't match CLI behavior, you MUST fix the playbook BEFORE continuing with your task.**

### When to Self-Repair

- A command in the playbook returns an error
- The CLI syntax has changed
- A command flag doesn't exist
- The playbook references outdated behavior

### Self-Repair Workflow

1. **Stop your current task** - Do not continue with broken instructions
2. **Verify the correct CLI syntax**:
   ```bash
   systemprompt <domain> <subcommand> --help
   ```
3. **Edit the playbook file directly** - Playbooks are in `services/playbook/`
4. **Sync the updated playbook**:
   ```bash
   systemprompt core playbooks sync --direction to-db -y
   ```
5. **Verify the fix**:
   ```bash
   systemprompt core playbooks show <playbook_id>
   ```
6. **Resume your task** - Now using the corrected playbook

### Playbook File Locations

```
services/playbook/
├── guide/          # guide_* playbooks (onboarding, meta)
├── cli/            # cli_* playbooks (all CLI operations)
├── build/          # build_* playbooks (development standards)
└── content/        # content_* playbooks (content creation)
```

### Example: Fixing a Broken Command

```bash
# 1. Command in playbook fails
systemprompt admin agents list --verbose
# Error: unknown flag --verbose

# 2. Check correct syntax
systemprompt admin agents list --help

# 3. Edit the playbook file
# Edit: services/playbook/cli/agents.md
# Change: --verbose -> -v (or remove if not supported)

# 4. Sync changes
systemprompt core playbooks sync --direction to-db -y

# 5. Verify fix
systemprompt core playbooks show cli_agents

# 6. Resume task with corrected command
```

**DO NOT continue with a broken playbook. Fix it first.**

---

## Sync Playbooks

If playbooks are missing or outdated:

```bash
# Sync from disk to database
systemprompt core playbooks sync --direction to-db -y

# Verify sync
systemprompt core playbooks list
```