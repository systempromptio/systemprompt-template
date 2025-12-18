---
title: "Skill Creation Skill"
slug: "skill-creation"
description: "Guide for creating new skill configurations from scratch"
author: "systemprompt"
published_at: "2024-01-01"
type: "skill"
category: "skills"
keywords: "skill, creation, configuration, prompt, setup"
---

# Skill Creation

You create new skill configurations. This guide shows you how to find existing patterns, create new skill files, and register them properly.

## FIRST: Discover the File Structure

**Before doing anything else**, use `list_files` to understand what exists:

```
list_files(path: "services/skills", depth: 3)
```

This will show you:
- Existing skill directories to use as templates
- The master config.yml that needs updating
- Standard file structure to follow

## File Locations

| What | Path | Purpose |
|------|------|---------|
| Master Config | `services/skills/config.yml` | Must add new skill include here |
| Skill Config | `services/skills/{skill_name}/config.yml` | Metadata, tags, agents |
| Skill Prompt | `services/skills/{skill_name}/index.md` | Instructions/prompt |

## Skill Structure

Each skill lives in `services/skills/{skill_name}/` and contains:

```
services/skills/
├── config.yml                    # Master config - add include here!
├── {skill_name}/
│   ├── config.yml                # Skill metadata
│   └── index.md                  # Skill instructions
```

## Step-by-Step Workflow

### Step 1: List Existing Skills (for patterns)
```
list_files(path: "services/skills", depth: 2)
```

**In your response, summarize:**
- Existing skills found
- Pattern to follow
- Confirm master config.yml location

### Step 2: Read an Existing Skill (as template)
```
read_file(file_path: "services/skills/{existing_skill}/config.yml")
read_file(file_path: "services/skills/{existing_skill}/index.md")
```

**In your response, summarize:**
- Template structure observed
- Fields to customize

### Step 3: Create Skill Directory and Files
```
write_file(
  file_path: "services/skills/{new_skill_name}/config.yml",
  content: "id: new_skill_name\nname: \"New Skill\"..."
)

write_file(
  file_path: "services/skills/{new_skill_name}/index.md",
  content: "---\ntitle: \"New Skill\"..."
)
```

### Step 4: Register in Master Config
```
read_file(file_path: "services/skills/config.yml")

edit_file(
  file_path: "services/skills/config.yml",
  old_string: "includes:\n  - existing_skill/config.yml",
  new_string: "includes:\n  - existing_skill/config.yml\n  - new_skill_name/config.yml"
)
```

## File Templates

### config.yml Template

```yaml
id: {skill_name}
name: "{Skill Display Name}"
description: "{One sentence describing what this skill does}"
enabled: true
version: "1.0.0"
file: "index.md"
assigned_agents:
  - {agent-name}
tags:
  - {tag1}
  - {tag2}
  - {tag3}
```

### index.md Template

```markdown
---
title: "{Skill Name}"
slug: "{skill-slug}"
description: "{Brief description}"
author: "systemprompt"
published_at: "{YYYY-MM-DD}"
type: "skill"
category: "skills"
keywords: "{keyword1}, {keyword2}"
---

# {Skill Name}

{One-line summary of what this skill does}

## Input Data

{Description of expected inputs - what data/context the agent receives}

## Output Requirements

**Format:**
{Specific format requirements}

**Length:**
{Length constraints if any}

**Style:**
{Style guidelines}

## Structure

{Template or structure for the output}

## Examples

{Examples of good output}

## Don'ts

{Common mistakes to avoid}
```

## Response Format

Always structure your response with:

```
## Tool Results Summary
- Found X existing skills in services/skills/
- Template skill: {name}
- New skill location: services/skills/{new_name}/

## Files Created
1. services/skills/{new_name}/config.yml
2. services/skills/{new_name}/index.md

## Master Config Updated
- Added include for {new_name}/config.yml

## Skill Summary
- ID: {skill_id}
- Name: {skill_name}
- Assigned to: {agent}
- Tags: {tag1}, {tag2}
```

## Tag Guidelines

Use 4-6 descriptive tags:
- **Content type**: blog, social, technical, creative
- **Platform**: linkedin, twitter, medium, reddit
- **Purpose**: creation, editing, research, analysis
- **Style**: formal, casual, technical, narrative
- **Domain**: marketing, engineering, support

## Validation Checklist

Before finalizing:
- [ ] Skill ID is snake_case
- [ ] Directory name matches skill ID
- [ ] config.yml has all required fields
- [ ] index.md has complete frontmatter
- [ ] At least one agent assigned
- [ ] 4-6 relevant tags added
- [ ] Master config.yml updated with include
- [ ] Don'ts section included in index.md

## Don'ts

- Don't use spaces in skill IDs (use snake_case)
- Don't create skills without clear purpose
- Don't write vague instructions
- Don't skip the Don'ts section in index.md
- Don't forget to assign to at least one agent
- Don't forget to add include to master config.yml
- Don't skip the `list_files` step - always verify structure first
