---
title: "Access Control"
description: "Configure role-based and department-based access rules for plugins, agents, and MCP servers from the Access Control dashboard."
author: "systemprompt.io"
slug: "access-control"
keywords: "access control, RBAC, roles, departments, permissions, security, plugins, agents, MCP servers"
kind: "guide"
public: true
tags: ["access-control", "security", "dashboard", "administration"]
published_at: "2026-03-02"
updated_at: "2026-03-02"
after_reading_this:
  - "Navigate the Access Control dashboard and its three entity tabs"
  - "Assign role-based and department-based access rules to individual entities"
  - "Use bulk assignment to apply access rules across multiple entities at once"
  - "Understand how access rules filter what users see across the platform"
related_docs:
  - title: "Authentication"
    url: "/documentation/authentication"
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "Agents"
    url: "/documentation/agents"
  - title: "MCP Servers"
    url: "/documentation/mcp-servers"
  - title: "Users"
    url: "/documentation/users"
  - title: "Presentation"
    url: "/documentation/presentation"
---

# Access Control

**TL;DR:** The Access Control page lets administrators assign role-based and department-based permissions to every plugin, agent, and MCP server in the system. Rules determine which users can see and use each resource, and changes can be applied individually or in bulk across multiple entities at once.

> **See this in the presentation:** [Slide 10: RBAC](/documentation/presentation#slide-10)

## What You'll See

When you open **Access Control** from the sidebar you land on a tabbed dashboard with three panels -- Plugins, Agents, and MCP Servers. Each panel shows a table of entities with their current role and department assignments. A toolbar at the top provides search, filtering, and bulk assignment controls.

This page is admin-only. Non-admin users receive an "Access Denied" message.

### Toolbar

The toolbar at the top of the page contains:

| Control | Description |
|---------|-------------|
| **Search** | Text input that filters the active tab's table by entity name |
| **Role filter** | Dropdown to show only entities assigned to a specific role (`admin`, `developer`, `analyst`, `viewer`) |
| **Department filter** | Dropdown to show only entities assigned to a specific department, with user counts shown in parentheses |
| **Bulk Assign** | Button that opens the bulk assignment panel. Shows the count of selected entities. Disabled when no entities are selected. |

### Tab Bar

Three tabs switch between entity types, each showing a count badge:

| Tab | Shows |
|-----|-------|
| **Plugins** | All installed plugins with role and department assignments |
| **Agents** | All agents with role and department assignments |
| **MCP Servers** | All MCP servers with role and department assignments |

### Coverage Summary

Below the tabs, a coverage indicator summarizes how many entities have access rules configured.

## Entity Tables

Each tab displays a table with identical columns:

| Column | Description |
|--------|-------------|
| **Checkbox** | Select the entity for bulk assignment. A "select all" checkbox in the header selects all visible entities. |
| **Name** | Entity name and truncated description |
| **Roles** | Blue badges for each assigned role. Shows "All" (gray badge) if no role restrictions are set. |
| **Departments** | Badges for each assigned department. Green badges indicate "default included" (enforced) departments. Yellow badges indicate explicitly assigned departments. Shows "None" if no department rules exist. |
| **Coverage** | Fraction showing how many departments out of the total have access (e.g., `2/5`) |
| **Status** | Green "Active" badge or gray "Disabled" badge reflecting whether the entity is enabled |

## Editing Access Rules

Click any entity row to open a side panel where you can configure its access rules.

### Side Panel

The side panel shows the entity name and provides controls to:

1. **Assign roles** -- Toggle which of the four built-in roles can access this entity.
2. **Assign departments** -- Toggle which departments can see and use this entity. Each department shows its user count.
3. **Save or cancel** -- Click "Save Changes" to persist the rules or "Cancel" to discard.

### Rule Types

Access rules have two dimensions:

| Rule Type | Description |
|-----------|-------------|
| **Role** | Controls access based on the user's assigned role. The four built-in roles are `admin`, `developer`, `analyst`, and `viewer`. |
| **Department** | Controls access based on the user's department. Departments are derived from user records in the database. |

Each rule specifies:

| Field | Description |
|-------|-------------|
| `rule_type` | Either `role` or `department` |
| `rule_value` | The specific role name or department name |
| `access` | `allow` to grant access |
| `default_included` | For department rules, whether this is a default/enforced assignment |

### How Rules Are Evaluated

- **Roles** -- For plugins, role assignments merge from two sources: the plugin's `config.yaml` `roles` field and database access control rules. A user sees the entity if any of their roles match any assigned role. If no role rules exist, the entity is visible to all roles.
- **Departments** -- Department rules are stored in the database only. A user sees the entity if their department matches an allowed department rule. If no department rules exist, department filtering does not apply.
- **Combined** -- Both role and department rules apply simultaneously. A user must satisfy both dimensions (if both are configured) to access the entity.

### YAML Synchronization

When you save role assignments for a plugin, the changes can optionally be synchronized back to the plugin's `config.yaml` file on disk. This keeps the YAML source of truth in sync with database rules. The sync writes the `roles` array to the plugin's configuration file.

## Bulk Assignment

Select multiple entities using the checkboxes and click the **Bulk Assign** button to open the bulk assignment panel.

### Bulk Assignment Panel

The bulk panel lets you apply the same set of access rules to all selected entities at once:

1. Select the entities you want to update across one or more tabs.
2. Click **Bulk Assign** (the button shows the count of selected entities).
3. In the bulk panel, configure the role and department rules you want to apply.
4. Click **Apply to Selected** to save the rules for all selected entities.

Bulk assignment replaces existing rules for each entity -- it does not merge with previous assignments.

## Entity Types

Access control rules can be applied to the following entity types:

| Entity Type | API Value | Description |
|-------------|-----------|-------------|
| Plugin | `plugin` | Controls who can see and use the plugin and its bundled resources |
| Agent | `agent` | Controls who can interact with the agent |
| MCP Server | `mcp_server` | Controls who can use the MCP server's tools |
| Marketplace | `marketplace` | Controls marketplace visibility (used in organization marketplace management) |

## How Access Rules Affect the Platform

Access control rules filter what users see across every page of the dashboard:

- **Plugins page** -- Users only see plugins whose roles include at least one of their assigned roles.
- **Skills, Agents, MCP Servers pages** -- Resources bundled inside restricted plugins are hidden from users who lack the required roles.
- **My Workspace** -- Users only see resources they have permission to fork or access.
- **Browse Plugins** -- Marketplace listings respect access control rules.

Admins always see everything regardless of access rules.

## API Endpoints

The Access Control page interacts with these API endpoints:

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/admin/api/access-control/rules` | List all rules, optionally filtered by `entity_type` and `entity_id` |
| PUT | `/admin/api/access-control/rules/{entity_type}/{entity_id}` | Set access rules for a specific entity (replaces existing rules) |
| POST | `/admin/api/access-control/bulk-assign` | Apply rules to multiple entities at once |
| GET | `/admin/api/access-control/departments` | List all departments with user counts |

## Troubleshooting

| Issue | Solution |
|-------|----------|
| **"Access Denied" when opening the page** | Only users with the `admin` role can access the Access Control page. Ask an administrator to add the admin role to your account. |
| **Entity missing from the table** | The entity may not be loaded from disk. Check that the plugin/agent/MCP server configuration exists under `services/` and contains valid YAML. Run `systemprompt infra logs view --level error` to check for parsing errors. |
| **Role changes not taking effect** | If you updated roles via the Access Control page but users still see the wrong resources, the change may not have synced to the plugin YAML. Re-save the rules and ensure YAML sync is enabled. Users may also need to refresh their session. |
| **Department filter shows no departments** | Departments are pulled from the `department` field on user records. If no users have departments assigned, the filter will be empty. Assign departments to users from the Users page. |
| **Bulk assign button stays disabled** | You must select at least one entity checkbox before the Bulk Assign button becomes active. Use the select-all checkbox in the table header to select all visible entities. |
| **Coverage shows 0 for all entities** | This means no department-level rules have been configured. Open individual entities and assign department access, or use bulk assignment to set department rules across multiple entities. |
