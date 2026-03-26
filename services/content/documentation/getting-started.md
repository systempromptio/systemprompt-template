---
title: "Getting Started"
description: "Create your first plugin for the systemprompt.io agentic governance platform. Learn the basics of skills, agents, and plugin configuration."
author: "systemprompt.io"
slug: "getting-started"
keywords: "getting started, quick start, first plugin, tutorial, agentic governance"
kind: "guide"
public: true
tags: ["getting-started", "tutorial"]
published_at: "2026-02-18"
updated_at: "2026-02-18"
after_reading_this:
  - "Create a working plugin with skills and an agent"
  - "Understand the plugin directory structure"
  - "Configure an agent with a system prompt"
  - "Test your plugin locally"
related_docs:
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "Skills"
    url: "/documentation/skills"
  - title: "Agents"
    url: "/documentation/agents"
  - title: "Installation"
    url: "/documentation/installation"
---

# Getting Started

**TL;DR:** A plugin is a directory with a `config.yaml` file that bundles skills, agents, and tools together. Create the config, define some skills, assign them to an agent, and your plugin is ready.

## What You Will Build

In this guide, you will create a simple plugin called "research-assistant" that gives Claude the ability to help with research tasks. The plugin will include:

- A skill for summarizing documents
- A skill for comparing sources
- An agent configured to use those skills

## Prerequisites

- A running systemprompt.io instance (see [Installation](/documentation/installation))
- Access to the admin dashboard

## Step 1: Create the Plugin Directory

Plugins live in `services/plugins/`. Create a directory for your plugin:

```
services/plugins/research-assistant/
└── config.yaml
```

## Step 2: Define Your Plugin

Create `services/plugins/research-assistant/config.yaml`:

```yaml
plugin:
  id: research-assistant
  name: "Research Assistant"
  description: "Helps with research tasks including summarization and source comparison"
  version: "1.0.0"
  enabled: true

  skills:
    source: explicit
    include:
      - document_summary
      - source_comparison

  agents:
    source: explicit
    include:
      - research

  mcp_servers:
    - systemprompt

  roles:
    - admin
    - user

  keywords:
    - research
    - summary
    - comparison
  category: productivity

  author:
    name: "Your Name"
```

This configuration tells the system that your plugin uses two skills and one agent.

## Step 3: Create the Skills

Skills are defined in `services/skills/`. Each skill gets its own directory:

Create `services/skills/document_summary/config.yaml`:

```yaml
skill:
  id: document_summary
  name: "Document Summary"
  description: "Summarizes documents, articles, and long-form content into concise overviews"
  version: "1.0.0"
  enabled: true

  tags:
    - research
    - summary
    - documents

  examples:
    - "Summarize this article for me"
    - "Give me the key points from this document"
    - "What are the main takeaways?"
```

Create `services/skills/source_comparison/config.yaml`:

```yaml
skill:
  id: source_comparison
  name: "Source Comparison"
  description: "Compares multiple sources to identify agreements, contradictions, and gaps"
  version: "1.0.0"
  enabled: true

  tags:
    - research
    - comparison
    - analysis

  examples:
    - "Compare these two articles"
    - "What do these sources agree on?"
    - "Find contradictions between these papers"
```

## Step 4: Configure the Agent

Agents are defined in `services/agents/`. Create `services/agents/research.yaml`:

```yaml
agents:
  research:
    name: research
    port: 9010
    endpoint: http://localhost:8080/api/v1/agents/research
    enabled: true
    is_primary: false
    default: false
    card:
      protocolVersion: "0.3.0"
      name: "Research Assistant"
      displayName: "Research Assistant"
      description: "An AI assistant specialized in research tasks"
      version: "1.0.0"
      preferredTransport: JSONRPC
      capabilities:
        streaming: true
        pushNotifications: false
        stateTransitionHistory: false
      defaultInputModes:
        - text/plain
      defaultOutputModes:
        - text/plain
        - application/json
      skills:
        - id: document_summary
          name: "Document Summary"
          description: "Summarizes documents and articles"
        - id: source_comparison
          name: "Source Comparison"
          description: "Compares multiple sources"
    metadata:
      systemPrompt: |
        You are a Research Assistant. Your role is to help users with research tasks.

        ## Capabilities
        - Summarize documents, articles, and long-form content
        - Compare multiple sources to find agreements and contradictions
        - Extract key points and takeaways

        ## Guidelines
        - Always cite specific parts of the source material
        - Present findings in a structured format
        - Acknowledge when sources are insufficient
      mcpServers:
        - systemprompt
      skills:
        - document_summary
        - source_comparison
    oauth:
      required: true
      scopes:
        - user
      audience: a2a
```

## Step 5: Verify Your Plugin

Use the admin dashboard to verify everything is configured correctly:

1. Navigate to **Plugins** to see your plugin listed
2. Check **Skills** to verify both skills appear
3. Check **Agents** to confirm the research agent is configured
4. Check **Hooks** if you added any event triggers

## Plugin Structure Summary

```
services/
├── plugins/research-assistant/
│   └── config.yaml              # Plugin definition
├── skills/
│   ├── document_summary/
│   │   └── config.yaml          # Skill: summarization
│   └── source_comparison/
│       └── config.yaml          # Skill: comparison
└── agents/
    └── research.yaml            # Agent configuration
```

## Next Steps

- Learn more about [Skills](/documentation/skills) to create advanced capabilities
- Read the [Agents](/documentation/agents) guide for detailed agent configuration
- Add [Hooks](/documentation/hooks) to automate workflows
- Connect [MCP Servers](/documentation/mcp-servers) for tool access
- Explore the [Plugins](/documentation/plugins) guide for packaging and distribution
