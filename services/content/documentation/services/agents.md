---
title: "Agent Services"
description: "Configure AI agents with A2A protocol, skills, capabilities, and OAuth security schemes. Agents are the AI workers that perform tasks in SystemPrompt."
author: "SystemPrompt Team"
slug: "services/agents"
keywords: "agents, a2a, skills, oauth, configuration"
image: "/files/images/docs/services-agents.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Agent Services

**TL;DR:** Agents are AI workers that perform tasks in SystemPrompt. Each agent is defined in a YAML file with an A2A protocol card, skills, a system prompt, and OAuth security configuration. Agents connect to AI providers for reasoning and MCP servers for tool access.

## The Problem

AI applications need a consistent way to define how AI workers behave. Without structure, each deployment would configure agents differently, making it hard to share agents, discover capabilities, or ensure security.

The agent service solves this by providing a standardized configuration format based on the A2A (Agent-to-Agent) protocol. Each agent has a card that describes its capabilities, making it discoverable by other agents and systems. Skills define what the agent can do, and OAuth scopes control who can use it.

## How Agents Work

Agents are defined as YAML files in `services/agents/`. When the application starts, it loads these files and makes each agent available at its configured endpoint. The agent's behavior comes from three sources: the system prompt that shapes its personality, the skills that define its capabilities, and the AI provider that powers its reasoning.

When a user sends a message to an agent, the agent:
1. Receives the message through its API endpoint
2. Loads the conversation context
3. Sends the message to the AI provider with the system prompt
4. If the AI needs tools, invokes them through MCP servers
5. Returns the response to the user

## Agent Configuration

Agents are configured in YAML files under `services/agents/`. The filename identifies the agent.

<details>
<summary>Full agent configuration structure</summary>

```yaml
# services/agents/welcome.yaml
agents:
  welcome:
    name: "welcome"
    port: 9000
    endpoint: "http://localhost:8080/api/v1/agents/welcome"
    enabled: true
    is_primary: true
    default: true
    card:
      # A2A protocol configuration
    metadata:
      systemPrompt: |
        Your agent's system prompt...
```

</details>

The key configuration sections are:

- **Basic settings** - name, port, endpoint, enabled status
- **A2A card** - protocol version, capabilities, transport modes
- **Security schemes** - OAuth configuration for access control
- **Skills** - capabilities the agent offers
- **System prompt** - personality and behavior instructions

## A2A Protocol Card

The card section defines the A2A protocol card that describes the agent to other systems:

<details>
<summary>A2A card configuration</summary>

```yaml
card:
  protocolVersion: "0.3.0"
  name: "Welcome"
  displayName: "Welcome"
  description: "A helpful AI assistant"
  version: "1.0.0"
  preferredTransport: "JSONRPC"

  provider:
    organization: "Your Organization"
    url: "https://yourproject.com"

  iconUrl: "https://ui-avatars.com/api/?name=W&..."
  documentationUrl: "https://yourproject.com/docs"

  capabilities:
    streaming: true
    pushNotifications: false
    stateTransitionHistory: false

  defaultInputModes:
    - "text/plain"

  defaultOutputModes:
    - "text/plain"
    - "application/json"
```

</details>

The card makes agents discoverable and interoperable. Other systems can query the card to understand what the agent does and how to communicate with it.

## Skills Configuration

Agents reference skills by id. Skills are defined separately in the skills service and can be shared across multiple agents.

```yaml
skills:
  - id: "general_assistance"
  - id: "content_writing"
```

The skill ids must match skills defined in `services/skills/`. Each skill brings its tags and examples, which help the agent understand what kinds of requests it can handle.

## Security Configuration

OAuth protects agent endpoints from unauthorized access:

<details>
<summary>Security configuration</summary>

```yaml
securitySchemes:
  oauth2:
    type: oauth2
    flows:
      authorizationCode:
        authorizationUrl: "/api/v1/core/oauth/authorize"
        tokenUrl: "/api/v1/core/oauth/token"
        scopes:
          anonymous: "Public access"
          user: "Authenticated user access"
          admin: "Administrative access"
    description: "OAuth 2.0 authentication"

security:
  - oauth2: ["anonymous"]  # Required scopes
```

</details>

The security section defines which scopes are required to access the agent. Use `anonymous` for public agents, `user` for authenticated access, or `admin` for restricted agents.

## System Prompt

The system prompt defines the agent's personality and behavior:

```yaml
metadata:
  systemPrompt: |
    You are a helpful AI assistant.

    ## Core Principles
    1. Be Helpful: Provide accurate information
    2. Be Clear: Use plain language
    3. Be Honest: Acknowledge limitations
```

Write system prompts that clearly define the agent's role, capabilities, and boundaries. Include guidance for common scenarios and how to handle edge cases.

## Service Relationships

Agents connect to other services:

- **Skills service** - Agents reference skills by id for capability definition
- **AI service** - Provides the LLM that powers agent reasoning
- **MCP servers** - Provide tools agents can use during conversations
- **Config service** - Includes agent configurations through the aggregation pattern

## Managing Agents

Use the CLI to manage agents:

```bash
# List all agents
systemprompt admin agents list

# Show agent details
systemprompt admin agents show welcome

# Sync agent configuration to database
systemprompt cloud sync local agents --direction to-db -y
```

## CLI Reference

| Command | Description |
|---------|-------------|
| `systemprompt admin agents list` | List configured agents |
| `systemprompt admin agents show <name>` | Display agent configuration |
| `systemprompt admin agents validate` | Check agent configs for errors |
| `systemprompt admin agents create` | Create new agent |
| `systemprompt admin agents edit <name>` | Edit agent configuration |
| `systemprompt admin agents delete <name>` | Delete an agent |
| `systemprompt admin agents status` | Show agent process status |
| `systemprompt admin agents logs` | View agent logs |
| `systemprompt admin agents registry` | Get running agents from gateway (A2A discovery) |
| `systemprompt admin agents message <name>` | Send message to agent via A2A protocol |
| `systemprompt admin agents task <id>` | Get task details and response from agent |
| `systemprompt admin agents tools <name>` | List MCP tools available to agent |
| `systemprompt admin agents run <name>` | Run agent server directly (bypasses orchestration) |

See `systemprompt admin agents <command> --help` for detailed options.

## Troubleshooting

**Agent not responding** -- Check that the agent is enabled and the endpoint is correct. Verify the AI provider is configured and has valid credentials.

**Unauthorized errors** -- The user's token does not have the required scopes. Check the security configuration and user permissions.

**Skills not working** -- Verify skill ids in the agent match skills defined in the skills service. Sync both agents and skills to the database.