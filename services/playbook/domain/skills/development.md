---
title: "Skills Development"
description: "Create, configure, and manage agent skills as reusable capabilities."
author: "SystemPrompt"
slug: "domain-skills-development"
keywords: "skills, development, configuration, agents, capabilities"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Skills Development

Skill creation and management. Config: `services/skills/<id>/config.yaml`

> **Help**: `{ "command": "core playbooks show domain_skills-development" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session](../cli/session.md)

---

## Create Skill

Step 1: Create `services/skills/<id>/config.yaml`:

```yaml
skill:
  id: code_review
  name: "Code Review"
  description: "Review code for quality, bugs, security issues, and best practices"
  version: "1.0.0"
  enabled: true
  tags:
    - code
    - review
    - quality
    - development
    - security
  examples:
    - "Review this code for bugs"
    - "Check this function for security issues"
    - "What improvements can be made to this code?"
    - "Is this code following best practices?"
  metadata:
    category: "development"
    difficulty: "intermediate"
```

Step 2: Sync

{ "command": "core skills sync --direction to-db -y" }

Step 3: Verify

{ "command": "core skills list" }
{ "command": "core skills show code_review" }

---

## Assign to Agent

Step 1: Reference in `services/agents/<agent>.yaml`:

```yaml
agents:
  developer-assistant:
    card:
      skills:
        - id: "code_review"
          name: "Code Review"
          description: "Review code for quality and best practices"
          tags: ["code", "review"]
          examples:
            - "Review this code for bugs"
        - id: "general_assistance"
          name: "General Assistance"
          description: "Help with questions"
          tags: ["assistance"]
          examples:
            - "Help me understand this"
    metadata:
      systemPrompt: |
        You are a developer assistant with code review skills.
```

Step 2: Sync both

{ "command": "core skills sync --direction to-db -y" }
{ "command": "cloud sync local agents --direction to-db -y" }

Step 3: Verify

{ "command": "admin agents show developer-assistant" }

---

## Update Skill

{ "command": "core skills edit code_review" }

Or edit `services/skills/<id>/config.yaml`:

```yaml
skill:
  id: code_review
  name: "Code Review"
  description: "Comprehensive code review for quality, bugs, security, and performance"
  version: "1.1.0"
  enabled: true
  tags:
    - code
    - review
    - quality
    - security
    - performance
  examples:
    - "Review this code for bugs"
    - "Check for security issues"
    - "Review this pull request"
```

{ "command": "core skills sync --direction to-db -y" }
{ "command": "core skills show code_review" }

---

## Multiple Skills

Development skill set:

```yaml
skill:
  id: debugging
  name: "Debugging"
  description: "Help identify and fix bugs in code"
  version: "1.0.0"
  enabled: true
  tags:
    - debug
    - bugs
    - troubleshooting
  examples:
    - "Why is this code not working?"
    - "Help me debug this error"
```

```yaml
skill:
  id: refactoring
  name: "Refactoring"
  description: "Suggest improvements to code structure"
  version: "1.0.0"
  enabled: true
  tags:
    - refactor
    - clean-code
  examples:
    - "How can I improve this code?"
    - "Make this code more readable"
```

```yaml
skill:
  id: documentation
  name: "Documentation"
  description: "Write and improve code documentation"
  version: "1.0.0"
  enabled: true
  tags:
    - docs
    - readme
  examples:
    - "Write documentation for this function"
    - "Create a README for this project"
```

{ "command": "core skills sync --direction to-db -y" }
{ "command": "core skills list" }

---

## Configuration Reference

```yaml
skill:
  id: skill_id
  name: "Skill Name"
  description: "What the skill does"
  version: "1.0.0"
  enabled: true
  tags:
    - tag1
    - tag2
  examples:
    - "Example input 1"
    - "Example input 2"
  metadata:
    category: "category"
    difficulty: "beginner|intermediate|advanced"
```

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique identifier |
| `name` | Yes | Display name |
| `description` | Yes | What skill does |
| `version` | Yes | Semantic version |
| `enabled` | Yes | Active state |
| `tags` | No | Categorization |
| `examples` | No | Usage examples |
| `metadata` | No | Additional data |

---

## Best Practices

Naming:
- Use lowercase with underscores: `code_review`
- Be descriptive but concise

Descriptions:
- Start with verb: "Review...", "Help with..."
- Under 100 characters

Tags:
- Use lowercase
- Include category and action tags
- Limit to 5-7 per skill

Examples:
- Include 3-5 diverse examples
- Show different ways to invoke
- Cover common use cases

---

## Troubleshooting

- Skill not found: `{ "command": "core skills sync --direction to-db -y" }`, verify ID matches agent config
- Sync fails: `{ "command": "core skills validate" }`, check YAML syntax
- Skill disabled: Set `enabled: true` in config

---

## Quick Reference

| Task | Command |
|------|---------|
| List | `core skills list` |
| Show | `core skills show <id>` |
| Edit | `core skills edit <id>` |
| Sync | `core skills sync --direction to-db -y` |
| Validate | `core skills validate` |

---

## Related

-> See [Skills Troubleshooting](skills-troubleshooting.md)
-> See [Agent Operations](agents-operations.md)
-> See [Skills Service](/documentation/services/skills)