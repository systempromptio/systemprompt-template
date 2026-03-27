---
title: "Organization Management"
description: "Manage organization-level resources including marketplaces, plugins, skills, agents, MCP servers, and hooks. Admin-only pages for controlling what your entire organization can access."
author: "systemprompt.io"
slug: "organization"
keywords: "organization, org, admin, marketplaces, plugins, skills, agents, MCP servers, hooks, access control, departments, roles"
kind: "guide"
public: true
tags: ["organization", "admin", "marketplaces", "access-control"]
published_at: "2026-03-02"
updated_at: "2026-03-02"
after_reading_this:
  - "Understand the difference between organization-level and user-level resource management"
  - "Create and configure org marketplaces with role and department access rules"
  - "Manage org-level plugins, skills, agents, MCP servers, and hooks"
  - "Control which plugins are available to which teams through marketplace assignments"
related_docs:
  - title: "Marketplace"
    url: "/documentation/marketplace"
  - title: "Browse Plugins"
    url: "/documentation/browse-plugins"
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "Access Control"
    url: "/documentation/access-control"
  - title: "My Workspace"
    url: "/documentation/my-workspace"
---

# Organization Management

**TL;DR:** The Organization section is the admin-only area where you manage shared resources for your entire organization. You create marketplaces to group plugins, assign them to roles and departments, and manage the canonical set of plugins, skills, agents, MCP servers, and hooks that users can then fork into their personal workspaces.

## What You'll See

The Organization section lives under the `/admin/org/` path in the sidebar. It contains the following pages:

| Page | Path | Purpose |
|------|------|---------|
| **Org Marketplaces** | `/admin/org/marketplaces` | Create and manage plugin marketplaces with access control |
| **Org Plugins** | `/admin/org/plugins` | Manage the master set of plugins available to the organization |
| **Org Skills** | `/admin/org/skills` | Manage organization-level skill definitions |
| **Org Agents** | `/admin/org/agents` | Manage organization-level agent configurations |
| **Org MCP Servers** | `/admin/org/mcp-servers` | Manage organization-level MCP server connections |
| **Org Hooks** | `/admin/org/hooks` | Manage organization-level event hooks |

All organization pages are restricted to administrators. Non-admin users will see a "403 Forbidden" response.

## How Org Resources Work

Organization resources form the canonical, shared layer of your deployment. The resource flow is:

1. **Admins configure org resources** -- plugins, skills, agents, MCP servers, and hooks are defined at the organization level.
2. **Marketplaces group plugins** -- admins create marketplaces and assign plugins to them. Each marketplace has role and department access rules.
3. **Users browse and fork** -- regular users see only the marketplaces (and their plugins) that match their role and department. They can fork org resources into their personal workspace.

This separation means admins control the master configuration while users get a personal copy they can customize without affecting others.

## Org Marketplaces

The Org Marketplaces page (`/admin/org/marketplaces`) is the central hub for organizing how plugins are distributed to your team.

### Toolbar

The toolbar at the top provides:

- **Search** -- filter marketplaces by name.
- **Department filter** -- dropdown to show only marketplaces assigned to a specific department.
- **+ Create Marketplace** -- opens the create form.

### Stats Bar

A stats ribbon shows three counts at a glance:

| Stat | Description |
|------|-------------|
| **Marketplaces** | Total number of org marketplaces |
| **Total plugins** | Sum of all plugins across all marketplaces |
| **Total skills** | Sum of all skills across all marketplace plugins |

### Marketplaces Table

The main table displays each marketplace with the following columns:

| Column | Description |
|--------|-------------|
| **Name** | Marketplace name and description |
| **Plugins** | Badge showing the number of plugins assigned |
| **Resources** | Badges for skill, agent, MCP server, and hook counts across all assigned plugins |
| **Roles** | Badges showing which roles have access. "All" if no role restrictions are set |
| **Departments** | Badges showing which departments have access. Yellow badges indicate "default included" departments. "None" if no department restrictions |
| **Actions** | Three-dot menu with Edit, Manage Plugins, Copy Install Link, and Delete options |

Click any row to expand a detail panel showing the full plugin list with a sub-table of each plugin's name, category, and resource counts (skills, agents, MCP servers, hooks). A "Show JSON" button reveals the raw marketplace data.

### Creating a Marketplace

Click **+ Create Marketplace** to open the creation form with these fields:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| **Marketplace ID** | Text | Yes | Unique identifier (e.g., `engineering-tools`). Cannot be changed after creation |
| **Name** | Text | Yes | Display name shown in the table and to users |
| **Description** | Textarea | No | Brief description of the marketplace's purpose |
| **Roles** | Checklist | No | Select which roles can access this marketplace. If none are selected, all roles have access |
| **Departments** | Checklist | No | Select which departments can access this marketplace. Each department shows its user count. A "Default" toggle marks departments where the marketplace is included by default for new users |
| **Plugins** | Checklist | No | Select which plugins to include in this marketplace |

### Editing a Marketplace

Click **Edit** from the actions menu or the edit page at `/admin/org/marketplaces/edit?id=<marketplace-id>`. The edit form is identical to creation except:

- The Marketplace ID field is read-only.
- A **Delete** button appears at the bottom to remove the marketplace entirely.
- A **Save Changes** button replaces the "Create" button.

### Managing Plugins

Use "Manage Plugins" from the actions menu to open a side panel where you can quickly toggle which plugins belong to the marketplace without leaving the list page.

### Access Control Integration

Each marketplace integrates with the access control system through two mechanisms:

- **Role-based access** -- assign roles (e.g., `admin`, `editor`, `viewer`) to control which user roles can see the marketplace.
- **Department-based access** -- assign departments with an optional "default included" flag. When a department is marked as default, new users in that department automatically get access to the marketplace.

## Org Plugins, Skills, Agents, MCP Servers, and Hooks

The remaining org pages (`/admin/org/plugins`, `/admin/org/skills`, `/admin/org/agents`, `/admin/org/mcp-servers`, `/admin/org/hooks`) reuse the same interface as the standard admin pages for these resources. They display the same tables, forms, and functionality described in the respective documentation pages:

- [Plugins](/documentation/plugins) -- view, enable/disable, edit, and create plugins
- [Skills](/documentation/skills) -- view, edit, and manage skill definitions
- [Agents](/documentation/agents) -- view, edit, and configure agent behavior
- [MCP Servers](/documentation/mcp-servers) -- view, edit, and manage MCP server connections
- [Hooks](/documentation/hooks) -- view, edit, and manage event hooks

The key difference is scope: these org pages represent the **organization-level master copy** of each resource. Changes made here affect what is available for users to fork into their personal workspaces.

### Org vs. Admin Pages

| Aspect | Admin Pages (`/admin/plugins`, etc.) | Org Pages (`/admin/org/plugins`, etc.) |
|--------|--------------------------------------|----------------------------------------|
| **Access** | Admin only | Admin only |
| **Scope** | System-wide configuration | Organization-level resources |
| **UI** | Same handlers and templates | Same handlers and templates |
| **Purpose** | Direct system configuration | Canonical resources users can fork |

Both sets of pages use the same underlying SSR handlers, so the UI and functionality are identical. The org routes provide a conceptual grouping in the sidebar to distinguish organization-level resource management from system administration.
