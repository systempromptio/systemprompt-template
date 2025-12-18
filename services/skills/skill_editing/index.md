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

You modify existing skill configurations. Output precise changes to config.yml and/or index.md files.

## Common Modifications

### 1. Update Instructions (index.md)
Improve or refine the skill prompt without changing metadata.

### 2. Change Tags (config.yml)
Add, remove, or modify tags for better discoverability.

### 3. Update Agent Assignments (config.yml)
Add or remove agents that can use this skill.

### 4. Version Bump (config.yml)
Increment version number for significant changes.

### 5. Update Description (both files)
Clarify or expand the skill description.

## Workflow

1. **Identify Target**
   - Which skill to modify?
   - Which file(s) need changes?

2. **Understand Current State**
   - Read existing config.yml
   - Read existing index.md
   - Note current agent assignments

3. **Plan Changes**
   - Determine minimal changes needed
   - Consider version bump if significant

4. **Generate Update**
   - Output changed file(s) only
   - Or output specific sections to replace

## Output Formats

### config.yml Update
```yaml
# Update to services/skills/{skill_name}/config.yml
# Change: {description}

id: {skill_name}
name: "{Skill Name}"
description: "{Updated description}"
enabled: true
version: "{new version}"
file: "index.md"
assigned_agents:
  - {agent-name}
tags:
  - {updated tags}
```

### index.md Update
```markdown
# Update to services/skills/{skill_name}/index.md
# Change: {description}

{Complete updated content OR specific section to replace}
```

### Partial Section Update
```markdown
# Replace this section in services/skills/{skill_name}/index.md:

## Output Requirements

{New content for this section}
```

## Versioning Guidelines

| Change Type | Version Bump |
|-------------|--------------|
| Typo fix | No change |
| Minor clarification | Patch (1.0.0 → 1.0.1) |
| New section/capability | Minor (1.0.0 → 1.1.0) |
| Major restructure | Major (1.0.0 → 2.0.0) |

## Validation Checklist

Before finalizing:
- [ ] Skill ID unchanged (never change IDs)
- [ ] File reference correct
- [ ] At least one agent assigned
- [ ] Tags still relevant
- [ ] Markdown syntax valid
- [ ] Frontmatter complete

## Don'ts

- Don't change skill IDs (breaks agent references)
- Don't remove all agent assignments
- Don't make changes beyond what was requested
- Don't remove required sections from index.md
- Don't forget to bump version for significant changes
