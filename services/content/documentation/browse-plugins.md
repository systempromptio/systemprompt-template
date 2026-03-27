---
title: "Browse Plugins"
description: "Discover and add plugins from your organization's marketplaces. Browse available plugins, see what each one includes, and add them to your personal workspace."
author: "systemprompt.io"
slug: "browse-plugins"
keywords: "browse, plugins, discover, marketplace, install, add, fork, skills, agents, MCP servers, categories"
kind: "guide"
public: true
tags: ["plugins", "marketplace", "workspace", "discovery"]
published_at: "2026-03-02"
updated_at: "2026-03-02"
after_reading_this:
  - "Browse all plugins available to you from your organization's marketplaces"
  - "Understand the difference between Browse Plugins and the Marketplace page"
  - "Add plugins to your personal workspace"
  - "Filter plugins by category to find what you need"
related_docs:
  - title: "My Workspace"
    url: "/documentation/my-workspace"
  - title: "Marketplace"
    url: "/documentation/marketplace"
  - title: "Organization Management"
    url: "/documentation/organization"
  - title: "Plugins"
    url: "/documentation/plugins"
---

# Browse Plugins

**TL;DR:** The Browse Plugins page is your plugin discovery hub. It shows all plugins available to you from your organization's marketplaces, filtered by your role and department. From here you can see what each plugin includes and add it to your personal workspace with one click.

## What You'll See

When you open **Browse Plugins** from the sidebar, you see a catalog of all plugins you are authorized to access. The page pulls plugins from every org marketplace that matches your role and department, merges them into a single list, and sorts them alphabetically.

### Stats Bar

At the top, a stats ribbon shows two key numbers:

| Stat | Description |
|------|-------------|
| **Total available** | Number of plugins you can browse from all your authorized marketplaces |
| **Already added** | Number of plugins you have already added to your personal workspace |

### Category Filter

If plugins have categories assigned, a category filter appears. Categories are collected from all available plugins and sorted alphabetically. Select a category to narrow the list.

### Plugin Cards

Each plugin displays:

| Field | Description |
|-------|-------------|
| **Name** | The plugin's display name |
| **Description** | Brief summary of what the plugin does |
| **Category** | The plugin's category badge (if assigned) |
| **Version** | Current version number |
| **Skills** | Number of skills included |
| **Agents** | Number of agents included |
| **MCP Servers** | Number of MCP server connections included |
| **Already added** | Indicator showing whether this plugin is already in your workspace |

## How Browse Plugins Differs from Marketplace

These two pages serve different audiences and purposes:

| Aspect | Browse Plugins | Marketplace |
|--------|---------------|-------------|
| **Audience** | All users | Admins |
| **Purpose** | Discover and add plugins to your workspace | View analytics, ratings, and manage visibility |
| **Data shown** | Plugin contents (skills, agents, MCP counts) | Usage stats, star ratings, rank scores, user lists |
| **Access filtering** | Filtered by your role and department | Shows all plugins regardless of access rules |
| **Actions** | Add to workspace | Edit visibility rules, view detailed analytics |

In short: Browse Plugins is the **user-facing catalog** for finding and adding plugins. The Marketplace page is the **admin-facing analytics dashboard** for monitoring plugin adoption and managing visibility.

## Adding Plugins to Your Workspace

When you find a plugin you want to use:

1. Locate the plugin in the list (use search or category filter to narrow results).
2. Click the add button on the plugin card.
3. The plugin is forked into your personal workspace under **My Plugins**.

Once added, the plugin appears in your [My Workspace](/documentation/my-workspace) where you can customize its skills, agents, and other resources without affecting the original organization-level configuration.

Plugins that are already in your workspace show an "already added" indicator so you know which ones you have.

## Access and Authorization

Browse Plugins respects the access control rules configured on each org marketplace:

- **Role-based filtering** -- you only see plugins from marketplaces assigned to your role.
- **Department-based filtering** -- you only see plugins from marketplaces assigned to your department.
- **Admin override** -- administrators see plugins from all marketplaces regardless of role or department restrictions.
- **System plugin exclusion** -- the internal `systemprompt` system plugin is automatically hidden from the browse list.

If you cannot see a plugin you expect to find, ask your administrator to check the marketplace's role and department assignments in [Organization Management](/documentation/organization).
