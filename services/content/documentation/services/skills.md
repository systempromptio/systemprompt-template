---
title: "Skills Service"
description: "Define reusable agent capabilities through skills. Skills provide tagged, discoverable actions that multiple agents can share."
author: "SystemPrompt Team"
slug: "services/skills"
keywords: "skills, capabilities, agents, tags, examples, configuration"
image: "/files/images/docs/services-skills.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Skills Service

**TL;DR:** Skills are reusable capabilities that define what agents can do. Each skill has an id, name, description, tags, and examples. Multiple agents can share the same skills, making it easy to build specialized agents from a common capability library.

## The Problem

Agents need to know what they can do. Without structure, each agent would need its own list of capabilities, leading to duplication and inconsistency. When you improve a capability, you would need to update every agent that uses it.

Skills solve this by defining capabilities once and letting multiple agents reference them. A skill like "content_writing" can be used by a blog agent, a documentation agent, and a general assistant. Improvements to the skill benefit all agents that use it.

Skills also help with discovery. Users and other systems can query available skills by tag or category. This makes it possible to find the right agent for a task without knowing every agent's specific capabilities.

## How Skills Work

Skills are defined as YAML files in `services/skills/`. Each skill lives in its own subdirectory with a `config.yaml` file. The main skills configuration at `services/skills/config.yaml` uses includes to aggregate all individual skill definitions.

When an agent is defined, it references skills by id. At runtime, the agent's system prompt includes information about its skills, and the agent can describe what it can do based on its assigned skills.

Skills are not code. They are metadata that describes capabilities. The actual behavior comes from the agent's system prompt, the AI model, and any tools available through MCP servers.

## Directory Structure

```
services/skills/
├── config.yaml                    # Aggregates all skills
├── general_assistance/
│   └── config.yaml                # General assistance skill
└── content_writing/
    └── config.yaml                # Content writing skill
```

Each skill directory contains a `config.yaml` file defining that skill. The naming convention is snake_case for directories and ids.

## Skill Schema

Every skill has these fields:

<details>
<summary>Complete skill schema</summary>

```yaml
skill:
  id: skill_identifier           # Unique identifier (snake_case)
  name: "Human Readable Name"    # Display name
  description: "What this skill enables"  # Detailed description
  version: "1.0.0"               # Semantic version
  enabled: true                  # Whether skill is active

  tags:                          # Discoverable tags
    - tag1
    - tag2

  examples:                      # Example prompts
    - "Example user request 1"
    - "Example user request 2"

  metadata:                      # Optional metadata
    category: "category_name"
    difficulty: "beginner"       # beginner, intermediate, advanced
```

</details>

### Required Fields

- **id** - Unique identifier in snake_case. This is how agents reference the skill.
- **name** - Human-readable display name shown in interfaces.
- **description** - Clear explanation of what this skill enables.
- **enabled** - Boolean to activate or deactivate the skill.

### Optional Fields

- **version** - Semantic version for tracking changes.
- **tags** - Array of tags for discovery and categorization.
- **examples** - Array of example prompts that demonstrate the skill.
- **metadata** - Arbitrary key-value pairs for additional categorization.

## Creating a Skill

To create a new skill:

1. Create a directory in `services/skills/` with your skill id as the name
2. Create `config.yaml` in that directory with the skill definition
3. Add an include line in `services/skills/config.yaml`
4. Sync skills to the database

<details>
<summary>Example: Creating a code review skill</summary>

Create `services/skills/code_review/config.yaml`:

```yaml
skill:
  id: code_review
  name: "Code Review"
  description: "Reviews code for bugs, style issues, and improvements"
  version: "1.0.0"
  enabled: true

  tags:
    - code
    - review
    - development
    - quality

  examples:
    - "Review this function for bugs"
    - "What could be improved in this code?"
    - "Check this PR for issues"

  metadata:
    category: "development"
    difficulty: "intermediate"
```

Add to `services/skills/config.yaml`:

```yaml
includes:
  - general_assistance/config.yaml
  - content_writing/config.yaml
  - code_review/config.yaml
```

</details>

## Assigning Skills to Agents

Skills are assigned to agents in the agent's configuration file. Each agent lists the skill ids it should have:

```yaml
# In services/agents/your-agent.yaml
card:
  skills:
    - id: general_assistance
    - id: content_writing
    - id: code_review
```

The agent's system prompt should describe how to use these skills. The skill examples help the AI understand what kinds of requests each skill handles.

An agent can have any number of skills. More specialized agents might have just one or two skills, while general-purpose agents might have many.

## Managing Skills

Use the CLI to manage skills:

```bash
# List all skills
systemprompt admin skills list

# Show skill details
systemprompt admin skills show general_assistance

# Sync skills from disk to database
systemprompt cloud sync local skills --direction to-db -y

# Sync skills from database to disk
systemprompt cloud sync local skills --direction from-db
```

## Built-in Skills

SystemPrompt includes two default skills:

### General Assistance

The `general_assistance` skill handles questions, explanations, and general tasks. It is a catch-all for requests that do not fit a more specific skill.

### Content Writing

The `content_writing` skill handles writing, editing, and improving text content. It includes blog posts, documentation, emails, and other written content.

These skills can be modified or disabled. You can also create additional skills tailored to your specific use cases.

## Service Relationships

- **Agents** reference skills by id in their configuration
- **Config** service includes skills through the aggregation pattern
- Skills are metadata only and do not execute code themselves
- Skill tags enable discovery through the CLI and API

## CLI Reference

| Command | Description |
|---------|-------------|
| `systemprompt core skills list` | List configured skills |
| `systemprompt core skills show <id>` | Show skill details |
| `systemprompt core skills create` | Create new skill |
| `systemprompt core skills edit <id>` | Edit skill configuration |
| `systemprompt core skills delete <id>` | Delete a skill |
| `systemprompt core skills status` | Show database sync status |
| `systemprompt core skills sync` | Sync skills between disk and database |

See `systemprompt core skills <command> --help` for detailed options.

## Troubleshooting

**Skill not appearing in agent** -- Verify the skill id in the agent configuration matches the skill definition exactly. Check that the skill is enabled.

**Skill not loading** -- Check that the skill's config.yaml has valid YAML syntax and includes all required fields. Verify the include path in the main skills config.yaml.

**Sync fails** -- Ensure you have write permissions and the database is accessible. Check for validation errors in the skill definition.