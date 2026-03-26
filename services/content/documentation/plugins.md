---
title: "Plugins"
description: "Manage AI plugins from the dashboard. Plugins bundle skills, agents, MCP servers, and hooks into role-scoped packages that control what each team can access."
author: "systemprompt.io"
slug: "plugins"
keywords: "plugins, packages, marketplace, distribution, skills, agents, MCP servers, hooks, roles, dashboard"
kind: "guide"
public: true
tags: ["plugins", "dashboard", "configuration"]
published_at: "2026-02-18"
updated_at: "2026-02-18"
after_reading_this:
  - "Navigate the Plugins dashboard and understand its four tabs"
  - "Enable, disable, edit, and delete plugins"
  - "Inspect a plugin's skills, agents, MCP servers, and hooks inline"
  - "Export a plugin configuration for Claude Desktop"
related_docs:
  - title: "Create Plugin"
    url: "/documentation/create-plugin"
  - title: "Skills"
    url: "/documentation/skills"
  - title: "Agents"
    url: "/documentation/agents"
  - title: "MCP Servers"
    url: "/documentation/mcp-servers"
  - title: "Hooks"
    url: "/documentation/hooks"
  - title: "Presentation"
    url: "/documentation/presentation"
---

# Plugins

**TL;DR:** The Plugins page is your central configuration dashboard. It shows every installed plugin alongside a unified view of all agents, MCP servers, and skills across your deployment. From here you can enable or disable any resource, drill into plugin details, customize skills, browse skill files, and export plugin configurations for use with Claude Desktop.

> **See this in the presentation:** [Slide 15: Skills, Plugins & Marketplace](/documentation/presentation#slide-15)

## What You'll See

When you open **Plugins** from the sidebar you land on a tabbed dashboard with four panels:

| Tab | Shows |
|-----|-------|
| **Plugins** | Every installed plugin with toggle, resource badges, and an actions menu |
| **Agents** | All agents across all plugins with their enable/disable state and plugin association |
| **MCP Servers** | All MCP servers with binary, port, OAuth status, and plugin association |
| **Skills** | Every skill (system and custom) with command, source badge, and plugin association |

A stats ribbon at the top of each tab shows counts at a glance: how many resources are enabled out of the total, how many are custom versus system, and so on.

### Toolbar

Above the tabs you will find:

- **Search** -- filters the active tab's table by name.
- **Status filter** -- dropdown to show only enabled or disabled resources.
- **Export** -- exports the current configuration for Claude Desktop (generates a `claude_desktop_config.json` snippet).
- **+ Create Plugin** -- navigates to the [Create Plugin](/documentation/create-plugin) wizard.

## Plugins Tab

The plugins table has five columns:

| Column | Description |
|--------|-------------|
| **On** | Toggle switch to enable or disable the plugin. The "Custom Skills" virtual plugin cannot be toggled. |
| **Name** | Plugin name and truncated description. |
| **Resources** | Clickable badges showing skill count, agent count, MCP server count, and hook count. Clicking a badge expands the detail row to that section. |
| **Status** | "Active" (green) or "Disabled" (gray). |
| **Actions** | Three-dot menu with Edit, Generate (macOS/Linux), Generate (Windows), and Delete. |

### Expanding Plugin Details

Click a resource badge or the plugin row itself to expand an inline detail panel that shows four sections:

- **Skills** -- lists each skill with an enable/disable toggle, source badge (system or custom), and buttons to **Customize** (fork a system skill into a custom copy) or browse **Files** (view attached file metadata).
- **Agents** -- lists each agent with an enable/disable toggle.
- **MCP Servers** -- lists each server by ID.
- **Hooks** -- lists each hook showing its event type, matcher pattern, and whether it runs asynchronously.

### Enable / Disable a Plugin

Flip the toggle in the "On" column. The change takes effect immediately through the API. Disabling a plugin does not delete it; re-enable at any time.

### Edit a Plugin

Click **Edit** from the actions menu (or click the row) to open the edit form. The edit page lets you change:

- **Name** and **Description**
- **Version** and **Category**
- **Enabled** checkbox
- **Keywords** (comma-separated)
- **Author Name**
- **Roles** -- checkboxes for admin, developer, analyst, viewer
- **Skills** -- checklist of all available skill IDs
- **Agents** -- checklist of all available agents
- **MCP Servers** -- checklist of all available MCP servers

Click **Save Changes** to persist, **Cancel** to discard, or **Delete** to remove the plugin entirely. The Plugin ID is read-only after creation.

### Delete a Plugin

Deleting a plugin requires admin access. Use either the **Delete** button on the edit page or the **Delete** option in the actions dropdown. A confirmation prompt will appear. Deletion removes the plugin's `config.yaml` from disk and is not reversible.

### Generate for Claude Desktop

The actions menu offers **Generate (macOS/Linux)** and **Generate (Windows)**. These produce a JSON configuration snippet compatible with Claude Desktop's `claude_desktop_config.json`, so you can use the plugin's MCP servers outside the systemprompt.io dashboard.

## Agents Tab

Shows all agents across every plugin. Each row displays:

- Enable/disable toggle
- Agent name (with a "Primary" badge if applicable)
- Which plugin the agent belongs to
- Active/Disabled status

Click a row to navigate to the agent edit page.

## MCP Servers Tab

Shows every MCP server with:

- Enable/disable toggle
- Server ID and description
- Binary path (the executable)
- Port number
- Plugin association
- Status (Active/Disabled) and an "OAuth" badge if the server requires authentication

## Skills Tab

Shows all skills across all plugins, including both system skills (defined in plugin configs) and custom skills (user-created). Each row shows:

- Enable/disable toggle
- Skill name (with a "customized" badge if it was forked from a system skill)
- Command (the slash command used to invoke the skill)
- Source (system or custom)
- Plugin association
- **Customize** button (for system skills) to fork into an editable custom copy
- **Files** button to browse attached file metadata (path, category, language, size)

Stats show total skills, enabled count, custom count, and system count.

## The "Custom Skills" Virtual Plugin

User-created custom skills are grouped into a virtual plugin called **Custom Skills** that appears at the bottom of the plugins list. This plugin:

- Cannot be toggled, edited, or deleted from the plugins table
- Has no agents, MCP servers, or hooks
- Contains skills with commands in the format `/custom:<skill_id>`
- Is visible only when at least one custom skill exists

## Role-Based Access

Plugins use the `roles` field to control visibility. The four built-in roles are:

| Role | Typical access |
|------|---------------|
| `admin` | Full access to all plugins |
| `developer` | Development and engineering plugins |
| `analyst` | Data and analytics plugins |
| `viewer` | Read-only plugins |

When a non-admin user views the plugins page, they only see plugins whose roles match their own. Admins see everything.

## Plugin Configuration Fields

These are the fields stored in each plugin's `config.yaml`:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique identifier (kebab-case). Immutable after creation. |
| `name` | string | yes | Display name |
| `description` | string | no | What the plugin provides |
| `version` | string | no | Semantic version (defaults to `0.1.0`) |
| `enabled` | boolean | no | Whether the plugin is active (defaults to `true`) |
| `category` | string | no | Plugin category for organization |
| `keywords` | array | no | Comma-separated search keywords |
| `author_name` | string | no | Author or team name |
| `roles` | array | no | Which organizational roles receive this plugin |
| `skills` | array | no | Skill IDs included in this plugin |
| `agents` | array | no | Agent IDs included in this plugin |
| `mcp_servers` | array | no | MCP server IDs included in this plugin |
| `hooks` | array | no | Event hooks (see [Hooks](/documentation/hooks)) |

## Troubleshooting

**Plugin not appearing in the list** -- Verify that the plugin's `config.yaml` exists under `services/plugins/<id>/` and contains valid YAML. Check `systemprompt infra logs view --level error` for parsing errors.

**Toggle not persisting** -- The toggle calls the API to update the plugin's enabled state. Check your browser's network tab for a failed `PATCH` request. Ensure the user has admin access.

**Skills missing from a plugin** -- Open the plugin edit page and verify the skill IDs are checked in the Skills checklist. Skill IDs must exactly match those defined in `services/skills/`.

**Export button not working** -- The Export button generates configuration for Claude Desktop. Ensure at least one MCP server is configured and enabled across your plugins.

**"Admin access required" error on delete** -- Only users with the `admin` role can delete plugins. Non-admin users will see this error if they attempt deletion via the API.

**Role filtering hiding plugins** -- Non-admin users only see plugins whose `roles` array includes at least one of their assigned roles. To make a plugin visible to everyone, include all roles or ensure the `admin` role is assigned to the user.
