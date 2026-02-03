---
title: "Playbooks"
description: "Deterministic instruction rails for AI agents. Eliminate hallucination by giving your superagent exact, tested commands for every SystemPrompt operation."
author: "SystemPrompt Team"
slug: "playbooks"
keywords: "playbooks, ai agents, deterministic, automation, mcp, claude code"
image: "/files/images/docs/playbooks.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Playbooks

**TL;DR:** Playbooks are the rail-tracks of AI. They provide structured, deterministic guidance to superintelligence — eliminating hallucination by giving agents exact, tested commands for every operation. After reading this section, you'll understand how to plug playbooks into your superagent to automate almost all of your SystemPrompt development and use.

## Why Playbooks Exist

LLMs hallucinate. Ask Claude to "list all agents" and it might guess `systemprompt agents --list`, `sp list-agents`, or fabricate flags that don't exist. Each failed command wastes tokens, breaks workflows, and erodes trust.

Playbooks solve this by providing **pre-tested, deterministic commands** for every operation. Instead of guessing, agents read the playbook and execute the exact command that works:

```json
{ "command": "admin agents list" }
```

No ambiguity. No hallucination. The command either succeeds or triggers the self-repair protocol.

## How Playbooks Work

Playbooks are injected into agents at two levels: via MCP server instructions for MCP-connected agents, and via CLAUDE.md for Claude Code sessions.

<details>
<summary>Playbook injection flow diagram</summary>

```text
┌─────────────────────────────────────────────────────────────────┐
│                    AGENT STARTUP                                │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ MCP Server / CLAUDE.md Instruction Injection                    │
│ "MANDATORY: Before ANY task, run 'core playbooks                │
│  show guide_start' to load required playbook guide"             │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ Agent Loads guide_start Playbook                                │
│ Returns: Master index, task→playbook mapping, CLI syntax        │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ Agent Identifies Task Domain                                    │
│ Example: "Build an extension" → build_extension-checklist       │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ Agent Loads Domain Playbook                                     │
│ Returns: Step-by-step commands, verification steps              │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ Agent Executes Commands Deterministically                       │
│ No guessing, no improvisation — follows playbook exactly        │
└─────────────────────────────────────────────────────────────────┘
```

</details>

The key is the **mandatory instruction injection**. Agents cannot skip playbooks because the MCP server or CLAUDE.md makes loading `guide_start` a prerequisite for all tasks.

## Connecting Playbooks to Your Agent

### MCP Server Integration

The SystemPrompt MCP server injects playbook instructions automatically. In the server initialization code (see Related Code section for the source):

```rust
instructions: Some(
    "MANDATORY: Before ANY task, run 'core playbooks show guide_start' \
     to load the required playbook guide. Agents MUST load and follow \
     playbooks before executing tasks."
)
```

Any agent connecting to the SystemPrompt MCP server receives this instruction in the `ServerInfo` response. The agent then uses the `systemprompt` tool to load playbooks:

```json
{ "tool": "systemprompt", "arguments": { "command": "core playbooks show guide_start" } }
```

### Claude Code Integration

For Claude Code sessions, the same pattern works via `CLAUDE.md`:

```markdown
# SystemPrompt

**Everything is done through playbooks.** Read `guide_start` first.

## MANDATORY: Read Playbook Before Any Task

systemprompt core playbooks show guide_start

Then read the specific playbook for your task and follow it exactly.
```

Claude Code reads `CLAUDE.md` at session start, sees the mandatory instruction, and loads the playbook before attempting any operations.

## Playbook Taxonomy

Playbooks are organized into four categories by purpose:

| Category | Prefix | Purpose | Example Playbooks |
|----------|--------|---------|-------------------|
| **Guide** | `guide_*` | Onboarding and meta-documentation | `guide_start`, `guide_playbook` |
| **CLI** | `cli_*` | All CLI operations and workflows | `cli_agents`, `cli_deploy`, `cli_cloud` |
| **Build** | `build_*` | Development standards and patterns | `build_extension-checklist`, `build_rust-standards` |
| **Content** | `content_*` | Content creation workflows | `content_blog`, `content_linkedin` |

### Guide Playbooks

Start here. `guide_start` is the master index linking to all other playbooks. `guide_playbook` covers authoring standards for writing new playbooks.

### CLI Playbooks

One playbook per CLI domain. `cli_agents` covers agent management, `cli_cloud` covers cloud setup, `cli_deploy` covers deployment. Each playbook contains exact commands with all flags documented.

### Build Playbooks

Development standards and checklists. `build_extension-checklist` walks through creating a new Rust extension. `build_mcp-checklist` covers MCP server development.

### Content Playbooks

Workflow automation for content creation. `content_blog` provides the complete flow for writing and publishing blog posts.

## Viewing and Syncing Playbooks

### List All Playbooks

```bash
systemprompt core playbooks list
```

### Filter by Category

```bash
systemprompt core playbooks list --category cli
systemprompt core playbooks list --category build
```

### Read a Playbook

```bash
systemprompt core playbooks show guide_start
systemprompt core playbooks show cli_agents
```

### Raw Markdown Output

```bash
systemprompt core playbooks show cli_deploy --raw
```

### Sync Playbooks to Database

After editing playbook files, sync to the database:

```bash
systemprompt core playbooks sync --direction to-db -y
```

Playbooks are stored in `services/playbook/` and synced to the database for MCP access.

## The Self-Repair Protocol

Playbooks are living documents. When a command fails, agents don't retry or guess — they fix the playbook immediately.

### When to Self-Repair

- A command returns an error
- CLI syntax has changed
- A flag doesn't exist
- The playbook references outdated behavior

### Self-Repair Workflow

1. **Stop current task** — Do not continue with broken instructions
2. **Verify correct CLI syntax**:
   ```bash
   systemprompt <domain> <subcommand> --help
   ```
3. **Edit the playbook file** in `services/playbook/`
4. **Sync the updated playbook**:
   ```bash
   systemprompt core playbooks sync --direction to-db -y
   ```
5. **Verify the fix**:
   ```bash
   systemprompt core playbooks show <playbook_id>
   ```
6. **Resume task** with corrected playbook

This creates a feedback loop: playbooks that fail get fixed, so the system improves over time. Broken commands cannot accumulate.

## Anatomy of a Playbook

Playbooks are Markdown files with YAML frontmatter and JSON command blocks. The directory structure organizes playbooks by category, and the filename determines the playbook ID used in CLI commands.

### File Structure

<details>
<summary>Playbook directory layout</summary>

```text
services/playbook/
├── guide/          # guide_* playbooks
│   ├── start.md    # guide_start
│   └── playbook.md # guide_playbook
├── cli/            # cli_* playbooks
│   ├── agents.md   # cli_agents
│   └── deploy.md   # cli_deploy
├── build/          # build_* playbooks
│   └── extension-checklist.md
└── content/        # content_* playbooks
    └── blog.md
```

</details>

The filename becomes the playbook ID: `cli/agents.md` becomes `cli_agents`.

### Frontmatter

Each playbook begins with YAML frontmatter defining its metadata. The category field determines which section the playbook appears in when listing.

<details>
<summary>Example frontmatter</summary>

```yaml
---
title: "Agents Management Playbook"
description: "Create, configure, and communicate with AI agents."
keywords:
  - agents
  - a2a
  - messaging
category: cli
---
```

</details>

### Command Format

Commands are encoded as JSON for deterministic parsing:

```json
{ "command": "admin agents list --enabled" }
{ "command": "admin agents show <name>" }
{ "command": "admin agents message <name> -m \"task\" --blocking" }
```

No prose, no ambiguity. Agents parse and execute directly.

## Key Playbooks

See the Related Playbooks section for direct links to the most commonly used playbooks:

| Task | Playbook ID | Description |
|------|-------------|-------------|
| Start here | `guide_start` | Master index, required first read |
| Session/auth | `cli_session` | Login, profiles, authentication |
| Cloud setup | `cli_cloud` | Tenants, credentials, profiles |
| Deploy | `cli_deploy` | Production deployment |
| Agent operations | `cli_agents` | A2A messaging, agent config |
| Build extension | `build_extension-checklist` | Rust extension development |
| Build MCP server | `build_mcp-checklist` | MCP server development |
| Write blog post | `content_blog` | Content creation workflow |

## Determinism Guarantees

Playbooks ensure deterministic execution through multiple mechanisms. Each mechanism reinforces the others, creating a system where agent behavior becomes predictable and auditable. When an agent follows a playbook, every command is traceable back to a tested, version-controlled source.

| Mechanism | How It Works |
|-----------|--------------|
| **MCP Injection** | Server instructions make playbook loading mandatory |
| **JSON Commands** | No prose, no ambiguity — parseable by agents |
| **Self-Repair Loop** | Broken commands trigger immediate fixes |
| **Version Control** | Playbooks in git, changes tracked, rollback possible |
| **Testing Protocol** | Commands validated with `--help` before commit |
| **Sync Mechanism** | Database sync ensures all agents see consistent state |