---
title: "Agent Editing Skill"
slug: "agent-editing"
description: "Guide for modifying existing agent configurations"
author: "systemprompt"
published_at: "2024-01-01"
type: "skill"
category: "skills"
keywords: "agent, editing, modification, yaml, update"
---

# Agent Editing

You modify existing agent configurations. Output precise YAML changes or complete updated configurations.

## Common Modifications

### 1. Update System Prompt
The most frequent change. Modify `metadata.systemPrompt` while preserving other settings.

### 2. Add/Remove Skills
Update `card.skills` array to add or remove skill references.

### 3. Change Security Level
Modify `card.security` to adjust access requirements.

### 4. Add/Remove MCP Servers
Update `metadata.mcpServers` to change tool access.

### 5. Update Display Information
Modify `card.displayName`, `card.description`, or `iconUrl`.

## Workflow

1. **Identify Target**
   - Which agent to modify?
   - What specific changes are needed?

2. **Understand Current State**
   - Read existing configuration
   - Note dependencies (skills, MCP servers)

3. **Plan Changes**
   - Determine minimal changes needed
   - Check for side effects

4. **Generate Update**
   - Output changed sections only, OR
   - Output complete updated file

## Output Formats

### Partial Update (Preferred for Small Changes)
```yaml
# Update to services/agents/{agent-name}.yml
# Change: {description of change}

# Replace this section:
metadata:
  systemPrompt: |
    {new system prompt}
```

### Full File (For Major Changes)
```yaml
# Complete replacement for services/agents/{agent-name}.yml
agents:
  {agent-name}:
    # ... complete configuration
```

## Validation Checklist

Before finalizing changes:
- [ ] Port number unchanged (unless intentional)
- [ ] Name unchanged (unless intentional)
- [ ] Required fields still present
- [ ] YAML syntax valid
- [ ] Referenced skills exist
- [ ] Referenced MCP servers exist

## Don'ts

- Don't change agent name without understanding impacts
- Don't change port numbers unnecessarily
- Don't remove security settings without explicit request
- Don't modify multiple agents in one output
- Don't make changes beyond what was requested
