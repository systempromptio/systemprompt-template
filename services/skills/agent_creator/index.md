---
title: "Agent Creator"
slug: "agent-creator"
description: "Socratic interview to design and create a custom AI agent with skills and MCP server access."
author: "systemprompt"
published_at: "2026-02-23"
type: "skill"
category: "marketplace"
keywords: "agents, creator, socratic, builder, custom, business, AI"
---

# Agent Creator

Guide the user through creating a custom AI agent by understanding what role it plays in their business. You ask questions and build the agent — the user never touches YAML or config files.

## What is an Agent?

An agent is an AI assistant with a specific role, skills, and tool access. It has:
- A name and purpose (what role it plays)
- Skills (reusable capabilities it can draw on)
- MCP server access (external tools and APIs it can use)
- A system prompt (detailed instructions for how it behaves)
- Guardrails (rules about what it must and must not do)

## Interview Flow

### Step 1: Role

**Ask:** "What role does this agent play? Think of it as a job title or a team responsibility. For example: 'content writer', 'customer support lead', 'data analyst', or 'operations coordinator'."

- Listen for: the agent's identity, scope of responsibility
- Follow-up: "How would you describe this agent's purpose in one sentence?"
- This derives: name, displayName, description

### Step 2: Tasks

**Ask:** "What specific tasks will this agent perform day-to-day? List the 3-5 most important things it does."

- Listen for: concrete actions, not abstract goals
- If abstract: "Can you give me an example of a specific task it would handle from start to finish?"
- This derives: the "Core Tasks" section of the system prompt

### Step 3: Skills

**Ask:** "Based on those tasks, let me show you the skills available in the system."

List existing skills using `list_skills` MCP tool or `core skills list` CLI.

**Then ask:** "Which of these existing skills should this agent use? Or do we need to create new ones?"

- If they pick existing skills: note the skill IDs
- If they need new skills: pause agent creation and switch to the `skill_creator` workflow. Return here after skills are created.
- This derives: card.skills references

### Step 4: Tools and MCP Servers

**Ask:** "Does this agent need access to any external tools or services? For example: search the web, send emails, manage files, access a database, or interact with other platforms?"

List available MCP servers using `plugins mcp list` CLI.

**Then ask:** "Any of these existing servers, or do we need to connect something new?"

- If existing: note the server IDs
- If new needed: pause and switch to `mcp_configurator` workflow
- This derives: mcp_servers list and the "Available Tools" section of system prompt

### Step 5: Guardrails

**Ask:** "What rules or guardrails should this agent follow? Think about:
- What should it ALWAYS do? (e.g., always be professional, always cite sources)
- What should it NEVER do? (e.g., never make promises about pricing, never share internal data)
- Are there any compliance or regulatory requirements?"

- Listen for: safety boundaries, brand protection, legal requirements
- This derives: the "Rules and Guardrails" section of system prompt

### Step 6: Authentication

**Ask:** "Who should be able to use this agent?
- **Anyone** (no login required — good for public-facing chatbots)
- **Logged-in users** (requires authentication — good for internal tools)
- **Admins only** (restricted to administrators)"

- This derives: oauth configuration and security scopes

### Step 7: Communication Style

**Ask:** "How should this agent communicate? For example:
- Formal and professional
- Casual and friendly
- Technical and precise
- Warm and empathetic
- Your company's specific brand voice"

- Listen for: tone, vocabulary level, personality
- Follow-up if they have brand voice: "What are the key rules? Any words to always or never use?"
- This derives: the "Communication Style" section of system prompt

## Synthesis

Present the complete agent configuration:

> "Here is the agent I have designed:
>
> **Name:** [name]
> **Role:** [one-line description]
> **Skills:** [list]
> **MCP Servers:** [list]
> **Auth:** [public/user/admin]
>
> **System prompt preview:**
> You are [name], a [role]. [2-3 sentence summary of purpose and style]
>
> Does this look right? Anything to adjust?"

Wait for confirmation.

## Agent YAML Template

Generate following this structure (matching blog_orchestrator.yaml and welcome.yaml patterns):

```yaml
agents:
  {agent_id}:
    name: "{agent_id}"
    port: {next_available_port}
    endpoint: "http://localhost:8080/api/v1/agents/{agent_id}"
    enabled: true
    is_primary: false
    default: false
    mcp_servers:
      - {mcp_server_id}
    card:
      protocolVersion: "0.3.0"
      name: "{Display Name}"
      displayName: "{Display Name}"
      description: "{one-line description}"
      version: "1.0.0"
      preferredTransport: "JSONRPC"
      provider:
        organization: "demo.systemprompt.io"
        url: "https://demo.systemprompt.io"
      iconUrl: "https://ui-avatars.com/api/?name={initials}&background={color}&color=fff&bold=true&size=256"
      capabilities:
        streaming: true
        pushNotifications: false
        stateTransitionHistory: false
      defaultInputModes:
        - "text/plain"
      defaultOutputModes:
        - "text/plain"
        - "application/json"
      skills:
        - id: "{skill_id}"
          name: "{Skill Name}"
          description: "{skill description}"
          tags: ["{tag1}", "{tag2}"]
          examples:
            - "{example usage}"
      supportsAuthenticatedExtendedCard: false
    metadata:
      systemPrompt: |
        {generated system prompt}
      mcpServers:
        - {mcp_server_id}
      skills: []
    oauth:
      required: {true/false}
      scopes: ["{scope}"]
      audience: "a2a"
```

## System Prompt Template

Generate the systemPrompt field following this structure:

```
You are {Display Name}, a {role description}.

## Your Purpose

{2-3 sentences about what this agent does and why, from Steps 1-2}

## Core Tasks

{Numbered list of specific tasks from Step 2}

1. {Task 1}
2. {Task 2}
3. {Task 3}

## Your Skills

You have access to these skills (loaded into your context):
- **{skill_name}**: {skill description}

## Available Tools ({mcp_server} MCP)

- **{tool_name}**: {tool description}
  - {parameter}: {description}

## Rules and Guardrails

**Always:**
- {Rule from Step 5}

**Never:**
- {Rule from Step 5}

## Communication Style

{Guidelines from Step 7}

## Workflow

When you receive a request:
1. Understand what is being asked
2. Determine which skill and tools apply
3. Execute one step at a time
4. Confirm results before proceeding
```

## Creation

After confirmation:

1. Write the agent YAML file to `services/agents/{agent_id}.yaml`
2. Sync: `cloud sync local agents --direction to-db -y`
3. Verify: `admin agents show {agent_id}`

## Port Assignment

Existing port allocations (check `admin agents list` for current state):
- 9000: welcome
- 9010: systemprompt_hub
- 9020-9040: blog agents
- 9050+: available for new agents

## Icon Colors

Choose a colour that reflects the agent's purpose:
- Blue (#3b82f6): information, support
- Green (#10b981): operations, productivity
- Purple (#3b82f6): creative, content
- Red (#ef4444): technical, engineering
- Orange (#f59e0b): analytics, data
- Teal (#0d9488): general, multipurpose
