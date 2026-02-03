---
title: "Skills Troubleshooting"
description: "Diagnose and fix skill issues: sync failures, missing skills, agent integration problems."
author: "SystemPrompt"
slug: "domain-skills-troubleshooting"
keywords: "skills, troubleshooting, debug, sync, agents"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Skills Troubleshooting

Diagnose and fix skill issues. Config: `services/skills/<id>/config.yaml`

> **Help**: `{ "command": "core playbooks show domain_skills-troubleshooting" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session](../cli/session.md)

---

## Diagnostic Checklist

{ "command": "core skills list" }
{ "command": "core skills sync --direction to-db --dry-run" }
{ "command": "core skills validate" }
{ "command": "infra logs --limit 50" }

---

## Issue: Skill Not Found by Agent

Symptoms: Agent doesn't recognize skill, "skill not found"

Step 1: Verify skill exists

{ "command": "core skills list" }

Step 2: Check skill details

{ "command": "core skills show <skill_id>" }

Step 3: Check agent config

{ "command": "admin agents show <agent_name>" }

Solutions:

Skill doesn't exist: Create `services/skills/<id>/config.yaml`:

```yaml
skill:
  id: my_skill
  name: "My Skill"
  description: "Does something useful"
  version: "1.0.0"
  enabled: true
```

{ "command": "core skills sync --direction to-db -y" }

ID mismatch: Ensure agent `skills.id` matches `skill.id` exactly

Not synced:

{ "command": "core skills sync --direction to-db -y" }

---

## Issue: Skill Sync Fails

Symptoms: Sync errors, changes not reflected

Step 1: Dry run

{ "command": "core skills sync --direction to-db --dry-run" }

Step 2: Validate

{ "command": "core skills validate" }

Step 3: Check YAML

```bash
cat services/skills/<skill_id>/config.yaml
```

Solutions:

YAML syntax error:

```yaml
skill:
  id: my_skill
  name: "My Skill"
  description: "Description here"
  version: "1.0.0"
  enabled: true
```

Missing required fields: Add all required fields (`id`, `name`, `description`, `version`, `enabled`)

Database issue:

{ "command": "infra db status" }
{ "command": "infra services restart" }

---

## Issue: Skill Disabled

Symptoms: Skill exists but agent doesn't use it

{ "command": "core skills show <skill_id>" }

Solution: Set `enabled: true`:

```yaml
skill:
  id: my_skill
  enabled: true
```

{ "command": "core skills sync --direction to-db -y" }

---

## Issue: Skill Version Mismatch

Symptoms: Old version in use, changes not taking effect

Check versions:

{ "command": "core skills show <skill_id>" }

```bash
cat services/skills/<skill_id>/config.yaml | grep version
```

Solution: Force sync:

{ "command": "core skills sync --direction to-db -y" }
{ "command": "core skills show <skill_id>" }

---

## Issue: Duplicate Skill IDs

Symptoms: Only one skill works, unexpected behavior

Check for duplicates:

```bash
grep -r "id: my_skill" services/skills/
```

Solution: Rename duplicate skills to unique IDs:

```yaml
skill:
  id: skill_a
```

```yaml
skill:
  id: skill_b
```

---

## Issue: Examples Not Working

Symptoms: Agent doesn't trigger skill from examples

{ "command": "core skills show <skill_id>" }

Solution: Add diverse examples:

```yaml
skill:
  examples:
    - "Example phrase one"
    - "Different example two"
    - "Third variation"
    - "Another way to ask"
```

{ "command": "core skills sync --direction to-db -y" }

---

## Issue: Tags Not Matching

Symptoms: Skill not found by tag search

{ "command": "core skills show <skill_id>" }
{ "command": "core skills list --tag development" }

Solution: Update tags:

```yaml
skill:
  tags:
    - lowercase
    - no-spaces
    - descriptive
```

---

## Validation

{ "command": "core skills validate" }

Common errors:

| Error | Cause | Fix |
|-------|-------|-----|
| Missing field | Required field not set | Add field |
| Invalid YAML | Formatting error | Fix syntax |
| Duplicate ID | Same ID twice | Rename skill |
| Invalid version | Not semantic | Use "1.0.0" |

---

## Quick Reference

| Problem | First Command |
|---------|---------------|
| Not found | `core skills list` |
| Sync fails | `core skills validate` |
| Disabled | `core skills show <id>` |
| Version | `core skills sync --direction to-db -y` |
| Any issue | `core skills validate` |

---

## Related

-> See [Skills Development](skills-development.md)
-> See [Agent Troubleshooting](agents-troubleshooting.md)
-> See [Skills Service](/documentation/services/skills)