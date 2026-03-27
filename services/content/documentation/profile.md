---
title: "Profile"
description: "View your personal profile page with account details, usage statistics, gamification progress, custom skills, achievements, and recent activity history."
author: "systemprompt.io"
slug: "profile"
keywords: "profile, user, account, gamification, achievements, activity, XP, rank"
kind: "guide"
public: true
tags: ["profile", "users", "gamification"]
published_at: "2026-02-18"
updated_at: "2026-02-18"
after_reading_this:
  - "View your account details and role assignments"
  - "Track your XP, rank, and streak progress"
  - "See your unlocked achievements"
  - "Review your recent activity and custom skills"
related_docs:
  - title: "Users"
    url: "/documentation/users"
  - title: "Dashboard"
    url: "/documentation/dashboard"
  - title: "Events"
    url: "/documentation/events"
  - title: "Plugins"
    url: "/documentation/plugins"
---

# Profile

**TL;DR:** The Profile page is your personal view of your account. It shows your identity, roles, department, usage statistics, gamification progress (XP, rank, streak, leaderboard position), unlocked achievements, custom skills, and recent activity. Every user can access their own profile — admins can also view any other user's profile.

## Access Control

- **Your own profile** — Available to all authenticated users via the "Profile" link in the sidebar
- **Other users' profiles** — Admin-only. Admins can view any user's profile by navigating to `/admin/user?id={user_id}`

The profile page is the same as the user detail page described in [Users](/documentation/users), but accessed through the sidebar "Profile" link which automatically fills in your own user ID.

## What You'll See

### Overview Cards

Three summary cards at the top of the page:

- **User** — Your display name and user ID
- **Total Events** — Your lifetime event count across all activity
- **Last Active** — When you were last active (relative timestamp)

### Profile Details

A table showing your account information:

| Field | Description |
|-------|-------------|
| **User ID** | Your unique identifier in the system |
| **Display Name** | Your name as shown across the platform |
| **Email** | Your email address |
| **Department** | Your organizational department (shown as a badge) |
| **Roles** | Your assigned roles (shown as badges), which determine your plugin access |
| **Status** | Whether your account is Active or Inactive |
| **Member Since** | The date your account was created |

### Gamification

When gamification is enabled, four stat cards show your progress:

| Stat | Description |
|------|-------------|
| **Total XP** | Experience points you have accumulated through platform activity |
| **Rank** | Your current rank (e.g., Spark, Flame, Blaze, Inferno) |
| **Current Streak** | Consecutive days you have been active |
| **Leaderboard** | Your current position on the organization leaderboard |

XP is earned through various activities on the platform including tool usage, session participation, and skill creation.

### Achievements

An achievement grid displays all achievements you have unlocked. Each achievement card shows:

- **Icon** — Category-specific icon (lightning bolt for First Steps, chart for Milestones, magnifier for Exploration, sparkle for Creation, fire for Streaks, trophy for Ranks, coin for Tokens, star for Special)
- **Name** — Achievement title
- **Description** — What you did to earn it
- **Category** — Badge indicating the achievement category

Achievement categories include:

| Category | Description |
|----------|-------------|
| First Steps | Introductory milestones for new users |
| Milestones | Cumulative usage milestones |
| Exploration | Discovering and using different features |
| Creation | Creating skills, plugins, or other content |
| Streaks | Maintaining consecutive days of activity |
| Ranks | Reaching new rank levels |
| Tokens | Token usage milestones |
| Special | Unique or rare accomplishments |

### Activity Breakdown

A grid of stat cards shows your events grouped by category. This gives you a quick view of how your activity is distributed across logins, tool usage, sessions, marketplace activity, and other categories.

### Custom Skills

A table lists any custom skills you have created or customized:

| Column | Description |
|--------|-------------|
| **Name** | Skill name |
| **Description** | What the skill does |
| **Status** | Enabled (green) or Disabled (gray) |
| **Tags** | Categorization tags (shown as badges) |
| **Updated** | When the skill was last modified |

### Recent Activity

A timeline feed at the bottom shows your most recent events. Each entry includes:

- A color-coded icon by category (blue for logins, orange for skill usage, green for marketplace edits, purple for marketplace connections, cyan for sessions)
- Description of the action
- Relative timestamp with full date on hover
