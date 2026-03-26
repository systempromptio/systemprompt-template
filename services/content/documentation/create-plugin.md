---
title: "Create Plugin"
description: "Create new plugins from the dashboard using the step-by-step wizard, or import existing plugins from a URL. Covers every wizard step, import options, and hook configuration."
author: "systemprompt.io"
slug: "create-plugin"
keywords: "create plugin, plugin wizard, import plugin, hooks, roles, MCP servers, agents, skills"
kind: "guide"
public: true
tags: ["plugins", "dashboard", "configuration"]
published_at: "2026-02-18"
updated_at: "2026-02-18"
after_reading_this:
  - "Create a new plugin using the seven-step wizard"
  - "Import a plugin from a URL (site-wide or user-only)"
  - "Configure hooks with event types, matchers, and async execution"
  - "Assign roles and access control during plugin creation"
related_docs:
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "Skills"
    url: "/documentation/skills"
  - title: "Hooks"
    url: "/documentation/hooks"
  - title: "MCP Servers"
    url: "/documentation/mcp-servers"
---

# Create Plugin

**TL;DR:** The Create Plugin page offers two ways to add a plugin: a seven-step wizard that walks you through configuration from scratch, and an Import from URL button that fetches a plugin bundle JSON and installs it in one click. Both methods are accessible from the Plugins dashboard.

## What You'll See

Navigate to **Plugins** in the sidebar and click **+ Create Plugin** in the toolbar. The page shows:

- A **Back to Plugins** link to return to the main dashboard.
- An **Import from URL** button to import an existing plugin bundle.
- A step indicator showing your progress through the wizard.
- A card containing the current step's form fields.
- Navigation buttons (Back / Next / Create) at the bottom of the card.

## The Seven-Step Wizard

### Step 1: Basic Info

Define the plugin's identity.

| Field | Required | Description |
|-------|----------|-------------|
| **Plugin ID** | yes | Unique identifier in kebab-case (e.g., `my-plugin`). Cannot be changed after creation. |
| **Name** | yes | Human-readable display name (e.g., "My Plugin"). |
| **Description** | no | A short paragraph explaining what the plugin provides. |
| **Version** | no | Semantic version string. Defaults to `0.1.0`. |
| **Category** | no | Organizational category (e.g., `productivity`, `engineering`, `sales`). |

### Step 2: Select Skills

Choose which skills to include in the plugin.

- A searchable checklist displays every available skill ID in the system.
- Use the search field to filter by name.
- **Select All** and **Deselect All** buttons for bulk operations.
- Skills can be shared across multiple plugins. Selecting a skill here does not remove it from other plugins.

### Step 3: Select Agents

Choose which agents to bundle with the plugin.

- Same searchable checklist interface as skills.
- Each agent is listed by its display name.
- Agents define the AI workers that will be available when the plugin is active.

### Step 4: MCP Servers

Choose which MCP servers the plugin requires.

- Same searchable checklist interface.
- MCP servers provide the external tools (APIs, databases, file systems) that agents use.
- If a server requires OAuth, that configuration is managed separately on the MCP server's edit page.

### Step 5: Hooks

Configure event-driven hooks that fire during the plugin lifecycle.

Click **+ Add Hook** to add a hook entry. Each hook has:

| Field | Description |
|-------|-------------|
| **Event** | The trigger event. Options: `PostToolUse`, `SessionStart`, `PreToolUse`, `Notification`. |
| **Matcher** | A pattern to match against the event context (e.g., `*` for all, or a specific tool name). |
| **Command** | The command to execute when the hook fires. |
| **Async** | Checkbox. When checked, the hook runs in the background without blocking the event. |

You can add multiple hooks, each with different events and matchers. Use the **Remove** button to delete a hook entry.

#### Hook Events

| Event | When it fires |
|-------|---------------|
| `PostToolUse` | After an MCP tool call completes |
| `PreToolUse` | Before an MCP tool call executes |
| `SessionStart` | When a new user session begins |
| `Notification` | When a notification is dispatched |

### Step 6: Roles and Access

Control who can access this plugin.

| Field | Description |
|-------|-------------|
| **Roles** | Checkboxes for `admin`, `developer`, `analyst`, `viewer`. Select the roles that should receive this plugin. |
| **Author Name** | The author or team responsible for the plugin. |
| **Keywords** | Comma-separated keywords for search and discovery. |

### Step 7: Review and Create

A summary grid displays all configuration choices before submission. Review the plugin ID, name, description, version, category, selected skills, agents, MCP servers, hooks, roles, author, and keywords.

Click **Create** to submit. The plugin is saved as a new `config.yaml` under `services/plugins/<plugin-id>/` and immediately appears in the Plugins dashboard.

## Importing a Plugin from URL

Click **Import from URL** at the top of the Create Plugin page to open the import modal.

### Import Fields

| Field | Description |
|-------|-------------|
| **URL** | The full URL to a plugin bundle JSON file. The format follows the Anthropic marketplace bundle specification. |
| **Import target** | `Site (shared plugin)` installs the plugin for all users. `Current user only` imports the bundle's skills as personal custom skills. |

### How Site Import Works

When you select **Site (shared plugin)**, the system:

1. Fetches the JSON bundle from the provided URL.
2. Parses the plugin configuration and file contents.
3. Creates the plugin directory and `config.yaml` under `services/plugins/`.
4. Records an activity log entry for the import.

The plugin appears immediately in the Plugins dashboard.

### How User Import Works

When you select **Current user only**, the system:

1. Fetches the JSON bundle from the provided URL.
2. Extracts all Markdown files from the `skills/` directory in the bundle.
3. Creates each as a personal custom skill associated with your user account.
4. Reports how many skills were imported.

User-imported skills appear under the **Custom Skills** virtual plugin and in the **My Skills** page.

### Import Errors

| Error | Cause |
|-------|-------|
| "Failed to fetch URL" | The URL is unreachable or returned a non-success HTTP status. |
| "Failed to parse plugin bundle JSON" | The response is not valid JSON or does not match the expected bundle format. |
| "No skills found in bundle" | When importing as user-only, the bundle contained no Markdown files under `skills/`. |

## After Creating a Plugin

Once your plugin is created:

1. **Enable it** -- new plugins are enabled by default, but verify the toggle is on.
2. **Assign roles** -- if you skipped roles during creation, edit the plugin to add them.
3. **Test** -- verify the plugin's skills and agents appear for users with the assigned roles.
4. **Export** -- use the Generate button on the Plugins page to create a Claude Desktop configuration if needed.

## Troubleshooting

**Wizard won't advance past Step 1** -- Plugin ID and Name are required fields. Ensure the Plugin ID uses kebab-case (lowercase letters, numbers, hyphens only).

**No skills/agents/MCP servers in the checklists** -- These lists are populated from existing resources in `services/skills/`, `services/agents/`, and MCP server configurations. If the lists are empty, no resources have been configured yet.

**Import fails with parse error** -- The URL must point to a raw JSON file in the plugin bundle format. If the URL returns HTML (e.g., a GitHub page instead of raw content), the import will fail.

**Plugin created but not visible** -- Check that the plugin has at least one role that matches the current user's roles. Admins can see all plugins regardless of role assignment.
