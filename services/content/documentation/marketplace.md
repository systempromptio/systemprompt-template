---
title: "Marketplace"
description: "Browse, search, and manage all plugins available in your organization's marketplace. View ratings, usage stats, visibility rules, and rank scores for every plugin."
author: "systemprompt.io"
slug: "marketplace"
keywords: "marketplace, plugins, ratings, usage, visibility, rank score, browse, search"
kind: "guide"
public: true
tags: ["marketplace", "plugins", "admin"]
published_at: "2026-02-18"
updated_at: "2026-02-18"
after_reading_this:
  - "Browse and search all marketplace plugins"
  - "Understand how plugins are ranked and scored"
  - "View plugin usage statistics and star ratings"
  - "Manage plugin visibility rules and role access"
related_docs:
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "Marketplace Versions"
    url: "/documentation/marketplace-versions"
  - title: "Leaderboard"
    url: "/documentation/leaderboard"
  - title: "Achievements"
    url: "/documentation/achievements"
---

# Marketplace

**TL;DR:** The Marketplace page is your organization's plugin catalog. It lists every plugin with its star rating, usage statistics, component counts (skills, agents, MCP servers, hooks), and a computed rank score. You can search, sort, expand plugin details, edit visibility rules, and view per-plugin user lists — all from one page.

## What You'll See

When you navigate to **Marketplace** in the admin sidebar, you land on the "All Plugins" page. The page displays:

- **Search bar** — A text input to filter plugins by name, description, or category.
- **Sort dropdown** — Sort the list by Rank (default), Highest Rated, Most Used, or Alphabetical.
- **Plugin cards** — One card per plugin, arranged vertically and ranked by score. Each card shows the plugin's name, version badge, category badge, description, star rating, user count, and component counts (skills, agents, MCP servers, hooks).

If no plugins exist, an empty state message reads "No marketplace plugins available."

## Plugin Cards

Each plugin card displays the following at a glance:

| Element | Description |
|---------|-------------|
| **Rank number** | Position in the ranked list (e.g. #1, #2, #3) |
| **Name** | Plugin display name |
| **Version badge** | Current semantic version (e.g. `1.2.0`) |
| **Category badge** | Plugin category (e.g. `sales`, `engineering`) |
| **Description** | Short text describing the plugin's purpose |
| **Star rating** | Average rating displayed as stars, with the numeric average and rating count |
| **User count** | Number of unique users who have used this plugin |
| **Component badges** | Counts of skills, agents, MCP servers, and hooks bundled in the plugin |
| **Enabled/Disabled badge** | Whether the plugin is currently active |

Click the expand arrow on any card to reveal the detail panel.

## Expanded Plugin Details

When you expand a plugin card, three sections appear:

### Visibility

Shows the current role assignments and any custom visibility rules applied to the plugin. Click **Edit Visibility** to modify which roles and rules control access. Visibility rules let you restrict or grant plugin access based on rule type and value (such as department or user group).

### Users

Click **Load Users** to fetch the list of users who have interacted with this plugin. The user list loads on demand to keep the page fast.

### Stats

Detailed usage metrics for the plugin:

- **Total Events** — Total number of usage events recorded for this plugin.
- **Active (7d)** — Users active in the last 7 days.
- **Active (30d)** — Users active in the last 30 days.
- **Rank Score** — The computed score used to order plugins in the marketplace (see below).

## How Rank Score Works

Plugins are sorted by a composite rank score calculated from three weighted factors:

```
rank_score = 0.4 * usage_score + 0.3 * active_score + 0.3 * bayesian_rating
```

Where:

- **usage_score** = `ln(total_events + 1)` — Logarithmic scale of total usage events, so early growth counts more.
- **active_score** = `ln(active_users_30d + 1)` — Logarithmic scale of 30-day active users.
- **bayesian_rating** = `(avg_rating * rating_count + 3.0 * 5.0) / (rating_count + 5.0)` — A Bayesian average that pulls ratings toward 3.0 when there are few votes, preventing a plugin with one 5-star rating from outranking a plugin with many 4-star ratings.

This formula rewards plugins that have both high usage and high ratings, while preventing gaming through a small number of ratings.

## Searching and Sorting

### Search

Type in the search bar to filter plugins in real time. The search matches against:

- Plugin name
- Plugin description
- Plugin category

### Sort Options

Use the sort dropdown to reorder the list:

| Option | Behavior |
|--------|----------|
| **Rank** | Default. Sorts by computed rank score (highest first) |
| **Highest Rated** | Sorts by average star rating |
| **Most Used** | Sorts by total usage events |
| **Alphabetical** | Sorts by plugin name A-Z |

## Editing Visibility

Click **Edit Visibility** on any expanded plugin card to manage access rules. Visibility is controlled through two mechanisms:

1. **Roles** — The roles listed in the plugin's `config.yaml` determine baseline access. Users with matching roles receive the plugin automatically.
2. **Visibility rules** — Additional rules stored in the database that grant or deny access based on rule type and value. These rules supplement the role-based access defined in the plugin configuration.

## Troubleshooting

**No plugins appear** — Verify that plugins exist under `services/plugins/` with valid `config.yaml` files. Check that at least one plugin has `enabled: true`.

**Rank score shows 0** — The plugin has no usage events and no ratings. Rank score increases as users interact with the plugin and submit ratings.

**Star rating seems wrong** — Ratings use a Bayesian average that pulls toward 3.0 with few votes. As more users rate the plugin, the displayed average converges toward the true mean.

**Edit Visibility button does nothing** — You must have admin role to modify visibility rules. Non-admin users can view but not edit visibility settings.

**Load Users shows empty** — No users have recorded usage events for this plugin yet. Usage events are tracked automatically when users interact with plugin capabilities through hooks.
