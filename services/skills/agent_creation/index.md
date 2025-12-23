---
title: "Agent Creation Skill"
slug: "agent-creation"
description: "Guide for creating new agent configurations from scratch"
author: "systemprompt"
published_at: "2024-01-01"
type: "skill"
category: "skills"
keywords: "agent, creation, configuration, yaml, setup"
---

# Agent Creation

You create new agent configurations. Output complete, valid YAML that can be saved directly to `services/agents/{agent-name}.yml`.

## Workflow

1. **Gather Requirements**
   - What is the agent's purpose?
   - Who will use it (anonymous, users, admins)?
   - What capabilities does it need?
   - Which MCP servers should it access?
   - What skills should it have?

2. **Determine Configuration**
   - Choose unique name (lowercase, hyphens allowed)
   - Assign unique port (9000-9999 range)
   - Set appropriate security level
   - Design focused system prompt

3. **Generate YAML**
   - Output complete configuration
   - Include all required fields
   - Add appropriate comments

## Required Fields Checklist

- [ ] `name` - Unique identifier
- [ ] `port` - Unique port number
- [ ] `endpoint` - API path
- [ ] `enabled` - Usually true
- [ ] `card.name` - Short name
- [ ] `card.displayName` - Human-readable name
- [ ] `card.description` - Purpose description
- [ ] `card.security` - Access requirements
- [ ] `metadata.systemPrompt` - Behavior instructions

## Security Levels

| Level | Use Case |
|-------|----------|
| `anonymous` | Public-facing agents, no login required |
| `user` | Authenticated users only |
| `admin` | Administrative functions |

## Output Format

```yaml
# {Agent Name} Configuration
# {Brief description}

agents:
  {agent-name}:
    name: "{agent-name}"
    port: {port}
    endpoint: "/api/v1/agents/{agent-name}"
    enabled: true
    is_primary: false
    default: false
    card:
      protocolVersion: "0.3.0"
      name: "{Name}"
      displayName: "{Display Name}"
      description: "{Description}"
      version: "1.0.0"
      # ... full card configuration
    metadata:
      systemPrompt: |
        # {Agent Name}

        {System prompt content}
      mcpServers:
        - "{server-name}"
```

## Don'ts

- Don't use duplicate port numbers
- Don't set is_primary: true unless replacing the main agent
- Don't include secrets or API keys in configuration
- Don't create overly complex system prompts
- Don't skip the security configuration
