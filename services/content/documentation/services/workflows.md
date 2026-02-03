---
title: "Workflows"
description: "Define once, execute anywhere. Skills and playbooks provide YAML-based automation that both humans and AI agents can run."
author: "SystemPrompt Team"
slug: "services/workflows"
keywords: "workflows, skills, playbooks, automation, yaml"
image: "/files/images/docs/services-workflows.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Workflows

Workflows in SystemPrompt are built on two primitives: **skills** and **playbooks**. Skills define what an agent can do. Playbooks define how to do it. Together they create a YAML-based automation system that works identically whether executed by a human via CLI or by an AI agent via MCP.

The key insight is that automation should be declarative, not imperative. You define the outcome you want, not the steps to get there. The runtime handles execution, error recovery, and state management.

## The Skills and Playbooks System

SystemPrompt's workflow system separates capability declaration from execution instructions. This separation enables several powerful patterns that traditional scripting cannot achieve.

**Skills** are atomic capabilities. Each skill has an identifier, a description, and metadata about what it can do. Skills are assigned to agents, giving them specific abilities. When an agent receives a task, it checks its assigned skills to determine if it can help.

**Playbooks** are executable instructions. Each playbook contains step-by-step guidance for completing a specific task. Playbooks can reference other playbooks, creating composable automation chains. When an agent needs to perform a task, it reads the relevant playbook and follows the instructions.

The `systemprompt-content` crate manages skill definitions and storage. Skills are stored in the database and synchronized from YAML files in `services/skills/`. The `systemprompt-scheduler` crate handles scheduled execution of workflows through background jobs.

## Define Reusable Skills

Skills live in `services/skills/` as YAML files. Each skill defines a capability that can be assigned to one or more agents. The skill schema includes an identifier, human-readable name, description, tags for categorization, and example prompts that demonstrate usage.

```yaml
# services/skills/content_writing.yaml
id: content_writing
name: "Content Writing"
description: "Writing, editing, and improving text content for blogs, documentation, and marketing materials."
tags:
  - writing
  - editing
  - content
  - marketing
examples:
  - "Write a blog post about AI agents"
  - "Edit this documentation for clarity"
  - "Create a product description"
```

Skills are referenced by agents in their configuration. An agent with the `content_writing` skill can handle requests that match the skill's description and examples. The A2A protocol uses skill metadata for capability negotiation between agents.

Sync skills to the database after creating or modifying them:

```bash
systemprompt cloud sync local skills --direction to-db -y
```

List all registered skills:

```bash
systemprompt core skills list
```

## Create Executable Playbooks

Playbooks are machine-readable instruction sets stored in `services/playbook/`. Unlike documentation that describes concepts, playbooks provide deterministic, executable steps. They are written for AI agents to follow, though humans can execute them too.

Each playbook has a category prefix that indicates its purpose:

| Prefix | Purpose | Example |
|--------|---------|---------|
| `guide_` | Entry points, getting started | `guide_start` |
| `cli_` | CLI operations | `cli_agents`, `cli_deploy` |
| `build_` | Development standards | `build_extension-checklist` |
| `content_` | Content creation | `content_blog`, `content_linkedin` |

Playbooks follow a consistent structure with frontmatter metadata and markdown instructions:

```yaml
---
title: "Deploy to Production"
description: "Step-by-step guide for deploying a SystemPrompt application to production."
category: cli
---

# Deploy to Production

## Prerequisites

- Active cloud profile configured
- Database migrations applied
- All tests passing

## Steps

1. Build the release binary
2. Run pre-deployment checks
3. Execute deployment command
4. Verify deployment status

## Commands

Build for release:

\`\`\`bash
cargo build --release
\`\`\`

Deploy to cloud:

\`\`\`bash
systemprompt cloud deploy --profile production
\`\`\`
```

The playbook system is self-repairing. When an agent encounters an error while following a playbook, it can modify the playbook to prevent the same error in the future. This creates a feedback loop where playbooks improve over time through actual usage.

## Run Workflows via CLI or AI Agents

Workflows can be triggered through multiple interfaces. The CLI provides direct access for human operators. MCP tools enable AI agents to execute the same workflows programmatically. Both interfaces use the same underlying execution engine.

**CLI execution:**

```bash
# List all playbooks
systemprompt core playbooks list

# Show a specific playbook
systemprompt core playbooks show guide_start

# Raw markdown output for piping
systemprompt core playbooks show cli_deploy --raw
```

**AI agent execution:**

When an MCP-connected AI client needs to perform a task, it can request the relevant playbook through the SystemPrompt MCP server. The agent reads the playbook, follows the instructions, and executes CLI commands through the MCP tool interface.

The closed-loop architecture means agents can also query their own execution history. An agent can ask "what playbooks have I run today?" or "what errors occurred during my last deployment?" This self-awareness enables adaptive behavior without human intervention.

**Scheduled execution:**

Workflows can be scheduled to run automatically using the scheduler service. Define jobs in `services/scheduler/config.yaml`:

```yaml
scheduler:
  enabled: true
  jobs:
    - name: publish_content
      extension: core
      job: publish_content
      schedule: "0 */30 * * * *"
```

The scheduler uses standard cron syntax. Jobs execute in the background and log their output for audit purposes.

## Workflow Patterns

Several patterns emerge from combining skills and playbooks:

**Skill-gated execution**: An agent only attempts tasks that match its assigned skills. This prevents capability overreach and ensures tasks route to appropriate agents.

**Playbook composition**: Complex workflows reference simpler playbooks. The `guide_start` playbook, for example, references `cli_session` for authentication and `cli_agents` for agent management.

**Error recovery**: When a playbook step fails, the agent can read error-handling instructions from the playbook or fall back to general troubleshooting playbooks.

**Audit trail**: Every workflow execution is logged with trace IDs. You can reconstruct exactly what happened during any workflow run.

## Configuration Reference

| Item | Location | Description |
|------|----------|-------------|
| Skills | `services/skills/*.yaml` | Skill definitions |
| Playbooks | `services/playbook/**/*.md` | Playbook instructions |
| Scheduler | `services/scheduler/config.yaml` | Job scheduling |
| Sync | `systemprompt cloud sync local` | Database synchronization |

## CLI Reference

Workflows are managed through skills, playbooks, and jobs commands.

| Command | Description |
|---------|-------------|
| `systemprompt core skills list` | List all registered skills |
| `systemprompt core skills show <name>` | Show skill details |
| `systemprompt core skills create` | Create a new skill |
| `systemprompt core skills edit <name>` | Edit an existing skill |
| `systemprompt core skills delete <name>` | Delete a skill |
| `systemprompt core skills sync` | Sync skills to/from database |
| `systemprompt core playbooks list` | List all playbooks |
| `systemprompt core playbooks show <id>` | Show playbook content |
| `systemprompt core playbooks sync` | Sync playbooks to/from database |
| `systemprompt infra jobs list` | List scheduled jobs |
| `systemprompt infra jobs run <name>` | Run a job immediately |
| `systemprompt infra jobs history` | Show job execution history |

See `systemprompt core skills --help`, `systemprompt core playbooks --help`, and `systemprompt infra jobs --help` for detailed options.