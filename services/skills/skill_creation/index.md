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

You create new skill configurations. Output both files needed for a complete skill: `config.yml` and `index.md`.

## Skill Structure

Each skill lives in `services/skills/{skill_name}/` and contains:

```
{skill_name}/
├── config.yml    # Metadata and configuration
└── index.md      # Skill instructions/prompt
```

## Workflow

1. **Define Purpose**
   - What should this skill help accomplish?
   - What inputs will it receive?
   - What outputs should it produce?

2. **Design Instructions**
   - Clear, actionable guidance
   - Specific format requirements
   - Examples of good output

3. **Configure Metadata**
   - Choose unique ID (snake_case)
   - Write descriptive name and description
   - Select appropriate tags
   - Assign to relevant agents

4. **Generate Files**
   - Output both config.yml and index.md

## Output Format

### config.yml

```yaml
id: {skill_name}
name: "{Skill Name}"
description: "{What this skill does - one sentence}"
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

### index.md

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

## Tag Guidelines

Use 4-6 descriptive tags:
- **Content type**: blog, social, technical, creative
- **Platform**: linkedin, twitter, medium, reddit
- **Purpose**: creation, editing, research, analysis
- **Style**: formal, casual, technical, narrative
- **Domain**: marketing, engineering, support

## Don'ts

- Don't use spaces in skill IDs (use snake_case)
- Don't create skills without clear purpose
- Don't write vague instructions
- Don't skip the Don'ts section in index.md
- Don't forget to assign to at least one agent
