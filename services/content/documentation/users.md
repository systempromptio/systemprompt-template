---
title: "Users"
description: "Manage organization users, assign roles and departments, view individual usage statistics, and control access to AI capabilities through the admin dashboard."
author: "systemprompt.io"
slug: "users"
keywords: "users, roles, departments, access control, RBAC, user management"
kind: "guide"
public: true
tags: ["users", "admin", "roles"]
published_at: "2026-02-18"
updated_at: "2026-02-18"
after_reading_this:
  - "View and search the user list"
  - "Create, edit, and delete users"
  - "Assign roles and departments to control plugin access"
  - "View individual user activity and gamification data"
related_docs:
  - title: "Dashboard"
    url: "/documentation/dashboard"
  - title: "Events"
    url: "/documentation/events"
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "Profile"
    url: "/documentation/profile"
---

# Users

**TL;DR:** The Users page lets admins manage all organization members. You can search, create, edit, and delete users, assign roles and departments, and view individual user details including activity history, custom skills, and gamification progress. The user list page is admin-only. Individual user detail pages are accessible by the user themselves or by admins.

## Access Control

- **User list** (`/admin/users/`) — Admin-only. Non-admin users receive a 403 Forbidden response.
- **User detail** (`/admin/user?id={user_id}`) — Admins can view any user. Non-admin users can only view their own profile.

## What You'll See

### User List

The user list page displays a table of all organization members with the following columns:

| Column | Description |
|--------|-------------|
| **User** | Display name (or user ID as fallback) with avatar initials, linked to the detail page |
| **Email** | User's email address |
| **Department** | Department badge, if assigned |
| **Roles** | Comma-separated list of assigned roles |
| **Status** | Green "Active" or gray "Inactive" badge |
| **Last Active** | Relative timestamp of last activity |

An actions menu (three-dot button) on each row provides quick access to edit and delete operations.

### Search

A search bar at the top filters the user table in real-time. It matches against both display name and email address. The search is client-side with a 200ms debounce for performance.

### Creating a User

Click the **+ Add User** button in the toolbar to open the create user panel. The following fields are available:

| Field | Required | Description |
|-------|----------|-------------|
| **User ID** | Yes | Unique identifier for the user |
| **Display Name** | Yes | Name shown in the UI |
| **Email** | Yes | User's email address |
| **Department** | No | Organizational department |
| **Roles** | No | Array of roles (e.g., `admin`, `sales`, `engineering`) |

After creation, the page reloads to show the new user in the list.

### Editing a User

Through the API, the following user fields can be updated:

- **Display Name** — Change how the user appears in the UI
- **Email** — Update the user's email
- **Department** — Reassign to a different department
- **Roles** — Add or remove roles to control plugin access
- **Active Status** — Enable or disable the user account

### Deleting a User

Users can be deleted through the actions menu. This is a destructive operation — the user and their associated data will be removed.

## User Detail Page

Navigate to a user's detail page by clicking their name in the user list, or go directly to `/admin/user?id={user_id}`.

### Overview Cards

Three stat cards at the top show:

- **User** — Display name and user ID
- **Total Events** — Lifetime event count
- **Last Active** — When the user was last active

### Profile Section

A table displays the user's core information:

- User ID
- Display Name
- Email
- Department (as a badge)
- Roles (as badges)
- Status (Active/Inactive)
- Member Since (account creation date)

### Gamification

When gamification data exists, four stat cards show:

- **Total XP** — Experience points accumulated
- **Rank** — Current rank name (e.g., "Spark", "Flame", "Blaze")
- **Current Streak** — Consecutive days of activity
- **Leaderboard** — Current position on the leaderboard

#### Achievements

Below the gamification stats, an achievement grid displays all unlocked achievements. Each achievement card shows:

- An icon based on the achievement category (First Steps, Milestones, Exploration, Creation, Streaks, Ranks, Tokens, Special)
- Achievement name
- Description of how it was earned
- Category badge

### Activity Breakdown

A grid of stat cards shows event counts grouped by category (e.g., logins, tool usage, sessions, marketplace activity).

### Custom Skills

A table lists the user's custom skills with columns for:

- Name
- Description
- Status (Enabled/Disabled)
- Tags (as badges)
- Last updated timestamp

### Recent Activity

A timeline feed shows the user's most recent events, with category-based color coding and relative timestamps.

## Roles and Access Control

Roles determine which plugins a user receives. When a user's roles match a plugin's `roles` configuration, that user gets access to all skills, agents, MCP servers, and hooks in that plugin.

Common roles include:

- `admin` — Full access to all admin pages and all plugins
- `user` — Default role assigned to all users
- Department-specific roles (e.g., `sales`, `engineering`, `hr`) — Grant access to role-specific plugins

## Troubleshooting

**User not appearing in the list** — Verify the user was created successfully. Check `systemprompt infra logs view --level error` for creation errors.

**User not receiving expected plugins** — Check that the user's roles match the plugin's `roles` array. Navigate to the user detail page to confirm their assigned roles.

**"Access Denied" when viewing a user** — Non-admin users can only view their own profile. Ensure you are either an admin or viewing your own user ID.

**User shows as "Never" for last active** — The user has not logged in or generated any events since their account was created.
