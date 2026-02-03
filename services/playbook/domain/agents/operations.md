---
title: "Agent Operations"
description: "Create, configure, and manage AI agents with A2A protocol, skills, and OAuth security."
keywords:
  - agents
  - a2a
  - skills
  - oauth
  - configuration
category: domain
---

# Agent Operations

Agent lifecycle management. Config: `services/agents/*.yaml`

> **Help**: `{ "command": "core playbooks show domain_agents-operations" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session](../cli/session.md)

---

## Create Agent

Step 1: Create `services/agents/<name>.yaml`:

```yaml
agents:
  my-assistant:
    name: "my-assistant"
    port: 9001
    endpoint: "http://localhost:8080/api/v1/agents/my-assistant"
    enabled: true
    is_primary: false
    default: false
    card:
      protocolVersion: "0.3.0"
      name: "My Assistant"
      displayName: "My Assistant"
      description: "A specialized AI assistant"
      version: "1.0.0"
      preferredTransport: "JSONRPC"
      provider:
        organization: "Your Organization"
        url: "https://yourproject.com"
      iconUrl: "https://ui-avatars.com/api/?name=MA&background=6366f1&color=fff&bold=true&size=256"
      capabilities:
        streaming: true
        pushNotifications: false
        stateTransitionHistory: false
      defaultInputModes:
        - "text/plain"
      defaultOutputModes:
        - "text/plain"
        - "application/json"
      securitySchemes:
        oauth2:
          type: oauth2
          flows:
            authorizationCode:
              authorizationUrl: "http://localhost:8080/api/v1/core/oauth/authorize"
              tokenUrl: "http://localhost:8080/api/v1/core/oauth/token"
              scopes:
                anonymous: "Public access"
                user: "Authenticated user access"
                admin: "Administrative access"
      security:
        - oauth2: ["anonymous"]
      skills:
        - id: "general_assistance"
          name: "General Assistance"
          description: "Help with questions and general tasks"
          tags: ["assistance", "general"]
          examples:
            - "Help me understand this"
      supportsAuthenticatedExtendedCard: false
    metadata:
      systemPrompt: |
        You are My Assistant, a helpful AI agent.

        ## Your Role
        You help users with their questions and tasks.

        ## Guidelines
        1. Provide accurate information
        2. Ask clarifying questions when needed
        3. Format responses with markdown
```

Step 2: Validate

{ "command": "admin agents validate" }

Step 3: Sync to database

{ "command": "cloud sync local agents --direction to-db -y" }

Step 4: Verify

{ "command": "admin agents list" }
{ "command": "admin agents show my-assistant" }
{ "command": "admin agents registry" }

Step 5: Test

{ "command": "admin agents message my-assistant -m \"Hello\" --blocking" }

---

## Add Skills

Step 1: Create skill at `services/skills/<id>/config.yaml`:

```yaml
skill:
  id: code_review
  name: "Code Review"
  description: "Review code for quality, bugs, and best practices"
  version: "1.0.0"
  enabled: true
  tags:
    - code
    - review
    - development
  examples:
    - "Review this code for bugs"
    - "Check this function for best practices"
```

Step 2: Reference in agent `services/agents/<name>.yaml`:

```yaml
skills:
  - id: "code_review"
    name: "Code Review"
    description: "Review code for quality and best practices"
    tags: ["code", "review"]
    examples:
      - "Review this code for bugs"
```

Step 3: Sync both

{ "command": "core skills sync --direction to-db -y" }
{ "command": "cloud sync local agents --direction to-db -y" }

---

## Configure OAuth

Public agent (no auth):

```yaml
security:
  - oauth2: ["anonymous"]
```

Authenticated agent:

```yaml
security:
  - oauth2: ["user"]
```

Admin-only agent:

```yaml
security:
  - oauth2: ["admin"]
```

OAuth endpoints configuration:

```yaml
securitySchemes:
  oauth2:
    type: oauth2
    flows:
      authorizationCode:
        authorizationUrl: "http://localhost:8080/api/v1/core/oauth/authorize"
        tokenUrl: "http://localhost:8080/api/v1/core/oauth/token"
        scopes:
          anonymous: "Public access"
          user: "Authenticated user access"
          admin: "Administrative access"
```

---

## Configure A2A Protocol

Capabilities:

```yaml
capabilities:
  streaming: true
  pushNotifications: false
  stateTransitionHistory: false
```

Transport:

```yaml
preferredTransport: "JSONRPC"
```

Input/Output modes:

```yaml
defaultInputModes:
  - "text/plain"
  - "application/json"
defaultOutputModes:
  - "text/plain"
  - "application/json"
  - "text/markdown"
```

---

## Monitor Agent

{ "command": "admin agents status" }
{ "command": "admin agents logs my-assistant" }
{ "command": "admin agents logs my-assistant --follow" }
{ "command": "admin agents registry" }
{ "command": "admin agents tools my-assistant" }

---

## Update System Prompt

{ "command": "admin agents edit my-assistant" }

Or edit `services/agents/<name>.yaml` directly:

```yaml
metadata:
  systemPrompt: |
    You are My Assistant, an AI agent specialized in code review.

    ## Your Role
    Expert code reviewer helping developers improve their code.

    ## Guidelines
    1. Be Thorough: Check for bugs, security, best practices
    2. Be Constructive: Provide actionable feedback
    3. Be Educational: Explain why changes are recommended
```

Sync:

{ "command": "cloud sync local agents --direction to-db -y" }

---

## Troubleshooting

- Agent not in list: `{ "command": "admin agents validate" }` then `{ "command": "cloud sync local agents --direction to-db -y" }`
- 401 Unauthorized: Check security scope, use `security: [oauth2: ["anonymous"]]` for public
- Skills not recognized: `{ "command": "core skills sync --direction to-db -y" }`, verify skill ID matches
- Agent not running: `{ "command": "admin agents status" }`, check logs

---

## Quick Reference

| Task | Command |
|------|---------|
| List | `admin agents list` |
| Show | `admin agents show <name>` |
| Validate | `admin agents validate` |
| Status | `admin agents status` |
| Logs | `admin agents logs <name>` |
| Message | `admin agents message <name> -m "text" --blocking` |
| Task | `admin agents task <id>` |
| Tools | `admin agents tools <name>` |
| Registry | `admin agents registry` |
| Edit | `admin agents edit <name>` |
| Sync | `cloud sync local agents --direction to-db -y` |

---

## Related

-> See [Agent Troubleshooting](agents-troubleshooting.md)
-> See [CLI Agents](../cli/agents.md)
-> See [Skills Development](skills-development.md)
-> See [Agent Service](/documentation/services/agents)
