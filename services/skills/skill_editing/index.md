---
title: "Skill Editing Skill"
slug: "skill-editing"
description: "Guide for modifying existing skill configurations"
author: "systemprompt"
published_at: "2024-01-01"
type: "skill"
category: "skills"
keywords: "skill, editing, modification, prompt, update"
---

# Skill Editing

You modify existing skill configurations. This guide shows you how to find, load, and edit skill files.

## FIRST: Discover the File Structure

**Before doing anything else**, use `list_files` to understand what exists:

```
list_files(path: "services/skills", depth: 3)
```

This will show you:
- All skill directories
- Which files exist in each skill
- The master config.yml location

## File Locations

| What | Path | Purpose |
|------|------|---------|
| Master Config | `services/skills/config.yml` | Lists all skill includes |
| Skill Config | `services/skills/{skill_name}/config.yml` | Metadata, tags, agents |
| Skill Prompt | `services/skills/{skill_name}/index.md` | Instructions/prompt |

## Step-by-Step Workflow

### Step 1: List Available Skills
```
list_files(path: "services/skills", depth: 2)
```

**In your response, summarize:**
- Number of skills found
- Skill names/directories
- Target skill location

### Step 2: Read Current Configuration
```
read_file(file_path: "services/skills/{skill_name}/config.yml")
read_file(file_path: "services/skills/{skill_name}/index.md")
```

**In your response, summarize:**
- Current version
- Current tags
- Current agent assignments
- Key sections of the prompt

### Step 3: Make Changes
```
edit_file(
  file_path: "services/skills/{skill_name}/config.yml",
  old_string: "...",
  new_string: "..."
)
```

**In your response, summarize:**
- What was changed
- Before vs after

## Common Modifications

### 1. Update Instructions (index.md)
Improve or refine the skill prompt without changing metadata.

```
edit_file(
  file_path: "services/skills/{skill_name}/index.md",
  old_string: "## Output Requirements\n\nOld content...",
  new_string: "## Output Requirements\n\nNew improved content..."
)
```

### 2. Change Tags (config.yml)
Add, remove, or modify tags for better discoverability.

```
edit_file(
  file_path: "services/skills/{skill_name}/config.yml",
  old_string: "tags:\n  - old_tag",
  new_string: "tags:\n  - new_tag\n  - another_tag"
)
```

### 3. Update Agent Assignments (config.yml)
Add or remove agents that can use this skill.

```
edit_file(
  file_path: "services/skills/{skill_name}/config.yml",
  old_string: "assigned_agents:\n  - old-agent",
  new_string: "assigned_agents:\n  - old-agent\n  - new-agent"
)
```

### 4. Version Bump (config.yml)
Increment version number for significant changes.

```
edit_file(
  file_path: "services/skills/{skill_name}/config.yml",
  old_string: "version: \"1.0.0\"",
  new_string: "version: \"1.1.0\""
)
```

## Response Format

Always structure your response with:

```
## Tool Results Summary
- Found X skills in services/skills/
- Target: services/skills/{name}/
- Files: config.yml, index.md

## Current State
[Summary of what you read from the files]

## Changes Made
[Specific edits applied]

## Verification
[Confirmation of what was changed]
```

## Versioning Guidelines

| Change Type | Version Bump | Example |
|-------------|--------------|---------|
| Typo fix | No change | Fixing "teh" → "the" |
| Minor clarification | Patch (1.0.0 → 1.0.1) | Rewording a sentence |
| New section/capability | Minor (1.0.0 → 1.1.0) | Adding new output format |
| Major restructure | Major (1.0.0 → 2.0.0) | Complete rewrite |

## Validation Checklist

Before finalizing:
- [ ] Skill ID unchanged (never change IDs)
- [ ] File reference correct
- [ ] At least one agent assigned
- [ ] Tags still relevant
- [ ] Markdown syntax valid
- [ ] Frontmatter complete
- [ ] Version bumped if significant change

## Don'ts

- Don't change skill IDs (breaks agent references)
- Don't remove all agent assignments
- Don't make changes beyond what was requested
- Don't remove required sections from index.md
- Don't forget to bump version for significant changes
- Don't skip the `list_files` step - always verify structure first
