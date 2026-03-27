---
title: "My Workspace"
description: "Your personal workspace in systemprompt.io lets you create, fork, and manage your own plugins, skills, agents, MCP servers, hooks, and marketplace view independently from organization-level resources."
author: "systemprompt.io"
slug: "my-workspace"
keywords: "workspace, my plugins, my skills, my agents, my mcp servers, my hooks, my marketplace, fork, customize, personal"
kind: "guide"
public: true
tags: ["workspace", "dashboard", "plugins", "skills", "agents", "mcp-servers", "hooks"]
published_at: "2026-03-02"
updated_at: "2026-03-02"
after_reading_this:
  - "Understand the difference between personal workspace resources and organization resources"
  - "Create, edit, and delete your own plugins, skills, agents, MCP servers, and hooks"
  - "Fork organization resources into your personal workspace using copy-on-write"
  - "Navigate the My Marketplace view to see inherited, customized, and custom plugins"
related_docs:
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "Skills"
    url: "/documentation/skills"
  - title: "Agents"
    url: "/documentation/agents"
  - title: "MCP Servers"
    url: "/documentation/mcp-servers"
  - title: "Secrets"
    url: "/documentation/secrets"
---

# My Workspace

**Your personal workspace is where you create custom resources and fork organization resources to tailor your systemprompt.io experience without affecting other users.**

## Personal vs. Organization Resources

systemprompt.io separates resources into two layers:

| Layer | Managed by | Stored in | Visible to |
|-------|-----------|-----------|------------|
| **Organization** | Admins | Filesystem (`services/`) | All authorized users (read-only) |
| **Personal (My)** | Each user | Database (per-user rows) | Only you |

Organization resources are the shared baseline configured by administrators. Your personal workspace lets you build on top of that baseline. You can create entirely new resources from scratch, or **fork** an organization resource to get a personal copy you can modify freely. Forked resources track their origin via a `base_*_id` field, displayed as a yellow "forked" badge in the UI.

## Forking Workflow (Copy-on-Write)

Forking creates a personal copy of an organization resource. The original remains untouched.

1. Navigate to the relevant "My" page (e.g., My Plugins).
2. Click **Fork from Org** to open the fork panel.
3. Select the organization resource you want to fork.
4. The system reads the resource configuration from the filesystem and creates a database copy owned by you.
5. Your forked copy appears in your workspace with a yellow **forked** badge showing the original ID.

**Deep fork for plugins:** When you fork a plugin, the system performs a deep fork -- it copies the plugin itself along with all of its associated skills, agents, MCP servers, and hooks into your workspace, then wires up the associations automatically.

You can edit or delete your forked copy at any time without affecting the organization original.

## My Plugins

**URL:** `/admin/my/plugins/`

The My Plugins page lists all plugins you own -- both custom-created and forked from the organization.

### What You'll See

A toolbar at the top provides:

- **Search** -- filters the table by plugin name.
- **Category filter** -- dropdown populated from your plugins' categories.
- **Bulk Actions** -- select multiple plugins via checkboxes for batch operations.
- **Fork from Org** -- opens a side panel listing forkable organization plugins.
- **+ Create Plugin** -- navigates to the plugin creation form.

A stats bar shows the total plugin count and how many are enabled.

### Plugin Table Columns

| Column | Description |
|--------|-------------|
| **Checkbox** | Select for bulk actions |
| **Name** | Plugin name and truncated description |
| **Category** | Category badge (blue) |
| **Version** | Version badge (gray) |
| **Resources** | Clickable badges for skill count, agent count, MCP server count, and hook count |
| **Source** | "forked" (yellow) if the plugin has a base_plugin_id, otherwise "custom" (green) |
| **Actions** | Three-dot menu with Edit and Delete options |

### Expanding Plugin Details

Click a row or resource badge to expand an inline detail panel showing:

- **Summary bar** -- author name, fork origin (if forked), keyword badges.
- **Skills sub-table** -- lists associated skills with remove buttons and an "Add Skill" button.
- **Agents sub-table** -- lists associated agents with remove/add controls.
- **MCP Servers sub-table** -- lists associated MCP servers with remove/add controls.
- **Hooks sub-table** -- lists hooks with event type, matcher pattern, async badge, and remove/add controls.
- **Show JSON** -- toggles a raw JSON view of the plugin data.

### Creating a Plugin

Click **+ Create Plugin** to open the edit form. Fill in:

| Field | Description |
|-------|-------------|
| **Plugin ID** | Unique kebab-case identifier (read-only after creation) |
| **Name** | Display name (required) |
| **Description** | Brief description of the plugin |
| **Version** | Semantic version string |
| **Category** | Plugin category for filtering |
| **Keywords** | Comma-separated keywords |
| **Author Name** | Your name or team name |

After saving, associate skills, agents, and MCP servers using the detail panel's add buttons.

## My Skills

**URL:** `/admin/my/skills/`

Lists all skills you own. Each skill shows its name, description, tags, enabled status, usage count, content preview, and whether it was forked.

### Stats Bar

- Total skill count
- Active (enabled) skill count

### Skill Table Features

- **Search** -- filter by skill name.
- **Tag filter** -- dropdown populated from all tags across your skills.
- **Fork from Org** -- fork an organization skill into your workspace.
- **+ Create Skill** -- open the skill creation form.

### Skill Edit Form Fields

| Field | Description |
|-------|-------------|
| **Skill ID** | Unique kebab-case identifier (read-only after creation) |
| **Name** | Display name (required) |
| **Description** | Brief description |
| **Content** | Skill content in markdown (large textarea) |
| **Tags** | Comma-separated tags for categorization |

Forked skills display a yellow banner showing the base skill ID they were forked from.

## My Agents

**URL:** `/admin/my/agents/`

Lists all agents you own. Each agent shows its name, description, system prompt preview, enabled status, usage count, plugin assignments, and fork origin.

### Stats Bar

- Total agent count
- Active (enabled) agent count

### Agent Edit Form Fields

| Field | Description |
|-------|-------------|
| **Agent ID** | Unique kebab-case identifier (read-only after creation) |
| **Name** | Display name (required) |
| **Description** | Brief description |
| **System Prompt** | Custom instructions for the agent (large textarea) |

Forked agents display a yellow banner showing the base agent ID.

## My MCP Servers

**URL:** `/admin/my/mcp-servers/`

Lists all MCP (Model Context Protocol) servers you own.

### Stats Bar

- Total server count
- Enabled server count

### MCP Server Table Data

Each row shows: name, description, binary, package name, port, endpoint, enabled status, OAuth requirement, and fork origin.

### MCP Server Edit Form Fields

| Field | Description |
|-------|-------------|
| **MCP Server ID** | Unique kebab-case identifier (read-only after creation) |
| **Name** | Display name (required) |
| **Description** | What this MCP server does |
| **Binary** | Command to run (e.g., `npx`) |
| **Package Name** | NPM package (e.g., `@scope/package`) |
| **Port** | Port number (0 for stdio) |
| **Endpoint** | URL endpoint for remote servers |
| **OAuth Required** | Checkbox to enable OAuth |
| **OAuth Scopes** | Comma-separated OAuth scopes |
| **OAuth Audience** | OAuth audience URL |

The edit page includes a **Delete** button for existing servers.

## My Hooks

**URL:** `/admin/my/hooks/`

Lists all hooks you own. Hooks execute commands in response to specific events during agent sessions.

### Stats Bar

- Total hook count
- Enabled hook count

### Supported Hook Events

Hooks can trigger on any of these events:

- `PostToolUse` -- after a tool call completes successfully
- `PostToolUseFailure` -- after a tool call fails
- `PreToolUse` -- before a tool call executes
- `UserPromptSubmit` -- when a user submits a prompt
- `SessionStart` -- when an agent session begins
- `SessionEnd` -- when an agent session ends
- `Stop` -- when an agent stops
- `SubagentStart` -- when a sub-agent starts
- `SubagentStop` -- when a sub-agent stops
- `Notification` -- on notification events

### Hook Edit Form Fields

| Field | Description |
|-------|-------------|
| **Hook ID** | Unique kebab-case identifier (read-only after creation) |
| **Name** | Display name (required) |
| **Description** | What this hook does |
| **Event** | Event type to trigger on (required, select from dropdown) |
| **Matcher** | Regex pattern to filter events (defaults to `.*`) |
| **Command** | Shell command to execute (required, textarea) |
| **Async Execution** | Checkbox -- when enabled, the hook runs without blocking the agent |

The edit page includes a **Delete** button for existing hooks.

## My Marketplace

**URL:** `/admin/my/marketplace/`

The My Marketplace page provides a unified view of all plugins available to you, combining organization-inherited plugins with your personal customizations and custom creations.

### What You'll See

A toolbar provides:

- **Search** -- filter plugins by name.
- **Source filter** -- show only Inherited, Customized, or Custom plugins.
- **Category filter** -- filter by plugin category.

### Stats Bar

- **Total plugins** -- combined count across all sources
- **Inherited** -- organization plugins you have access to (based on your roles and department)
- **Customized** -- organization plugins you have forked and modified
- **Custom** -- plugins you created from scratch

### Plugin Sources

| Source | Badge | Meaning |
|--------|-------|---------|
| **Inherited** | Purple | An organization plugin assigned to you via marketplace access control. Read-only. |
| **Customized** | Yellow | An organization plugin you have forked into your workspace. Editable. |
| **Custom** | Green | A plugin you created from scratch. Editable. |

### Actions by Source

- **Inherited plugins** show a **Customize** button that forks the plugin into your workspace (deep fork including all skills, agents, MCP servers, and hooks).
- **Customized and Custom plugins** show a three-dot menu with Edit and Delete options.

### Detail Expansion

Click any row to expand a detail panel showing:

- Fork origin (if customized)
- Skills list with names
- Agents list with names
- MCP Servers list with names
- Hooks list with event type, matcher, and async badge
- Show JSON toggle for raw data
