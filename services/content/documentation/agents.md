---
title: "Agents"
description: "Create and manage AI agents from the admin dashboard. Define system prompts, enable or disable agents, and control which agents are available to your organization."
author: "systemprompt.io"
slug: "agents"
keywords: "agents, ai, system prompt, configuration, admin, dashboard, create agent, manage agents"
kind: "guide"
public: true
tags: ["agents", "admin", "configuration"]
published_at: "2026-02-18"
updated_at: "2026-02-18"
after_reading_this:
  - "Navigate the Agents list page and understand what each column means"
  - "Create a new agent with an ID, name, and system prompt"
  - "Edit an existing agent's configuration"
  - "Enable, disable, or delete agents from the dashboard"
related_docs:
  - title: "MCP Servers"
    url: "/documentation/mcp-servers"
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "Hooks"
    url: "/documentation/hooks"
  - title: "Skills"
    url: "/documentation/skills"
---

# Agents

**TL;DR:** Agents are the AI workers in your systemprompt.io deployment. Each agent has an ID, a name, a system prompt that defines its behavior, and an enabled/disabled status. You manage agents from the admin dashboard -- creating new ones, editing their prompts, toggling them on or off, and deleting ones you no longer need.

## What You'll See

When you navigate to **Agents** in the admin sidebar, you land on the Agents list page. The page has two main elements:

1. **Toolbar** -- a search box to filter agents by name and a **+ New Agent** button to create one.
2. **Agents table** -- a data table with the following columns:

| Column | What it shows |
|--------|---------------|
| **Name** | The agent's display name |
| **Agent ID** | The unique identifier (shown as inline code) |
| **Description** | A truncated summary of the agent's purpose |
| **Primary** | A "Primary" badge if the agent is marked as the primary agent |
| **Status** | A toggle switch to enable or disable the agent |
| **Actions** | An action menu with Edit and Delete options |

If no agents exist, you see an empty state message: "No agents found."

### RBAC Visibility

Non-admin users only see agents that belong to plugins assigned to their roles. Admins see all agents.

## Creating an Agent

1. Click **+ New Agent** in the toolbar (or navigate to `/admin/agents/edit/`).
2. Fill in the form fields:

| Field | Required | Description |
|-------|----------|-------------|
| **ID** | Yes | A unique kebab-case identifier (e.g., `sales-assistant`). Cannot be changed after creation. |
| **Name** | Yes | A human-readable display name (e.g., "Sales Assistant"). |
| **Description** | No | A brief summary of what the agent does. |
| **System Prompt** | No | The instructions that shape the agent's personality and behavior. This is a large text area (15 rows) where you write the prompt the AI model receives. |
| **Enabled** | -- | Checkbox, enabled by default for new agents. |

3. Click **Save** to create the agent. You are redirected back to the Agents list.

### Writing a System Prompt

The system prompt is the most important part of agent configuration. It defines:

- The agent's role and personality
- What the agent should and should not do
- How it should respond to different types of requests
- Any domain-specific knowledge or constraints

Example:

```
You are a Customer Support Agent for Enterprise Demo.

## Your Role
Help customers resolve issues with their accounts, orders, and products.

## Guidelines
- Always greet the customer by name when available
- Escalate billing disputes to a human agent
- Never share internal system details or employee information
- If unsure, say so honestly rather than guessing
```

## Editing an Agent

1. Click the action menu (three dots) on any agent row and select **Edit**, or navigate to `/admin/agents/edit/?id=<agent-id>`.
2. Modify any field except the ID (which is read-only after creation).
3. Click **Save** to apply changes.

## Enabling and Disabling Agents

Use the toggle switch in the **Status** column to enable or disable an agent directly from the list page. Disabled agents are not available for conversations.

## Deleting an Agent

1. Click the action menu on the agent row.
2. Select **Delete**.
3. Confirm the deletion.

Deletion requires admin access. Non-admin users cannot delete agents.

## Searching Agents

Type in the search box at the top of the page to filter the agent list. The search matches against agent names.

## How Agents Connect to Other Concepts

- **Plugins** bundle agents together with skills, MCP servers, and hooks. An agent must belong to a plugin to be accessible to non-admin users.
- **MCP Servers** provide the tools an agent can use during conversations. Agents reference MCP servers in their underlying YAML configuration.
- **Skills** define the capabilities an agent advertises. Skills are listed on the agent's A2A protocol card.
- **Hooks** can fire in response to events generated during agent conversations (e.g., session start, tool usage).
