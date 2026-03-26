---
title: "Achievements"
description: "Track unlockable achievements and badges earned through AI usage. View all available achievements, unlock criteria, rarity percentages, and category breakdowns."
author: "systemprompt.io"
slug: "achievements"
keywords: "achievements, badges, gamification, unlock, progress, milestones, streaks, exploration"
kind: "guide"
public: true
tags: ["gamification", "achievements", "admin"]
published_at: "2026-02-18"
updated_at: "2026-02-18"
after_reading_this:
  - "See all available achievements and their unlock criteria"
  - "Understand achievement categories and difficulty progression"
  - "View rarity percentages showing how many users have unlocked each achievement"
  - "Learn how achievements connect to the XP and ranking system"
related_docs:
  - title: "Leaderboard"
    url: "/documentation/leaderboard"
  - title: "Marketplace"
    url: "/documentation/marketplace"
  - title: "Plugins"
    url: "/documentation/plugins"
---

# Achievements

**TL;DR:** Achievements are badges users earn by reaching specific milestones in their AI usage. There are 35 achievements across 8 categories — from starting your first session ("First Spark") to using a million tokens ("Megabyte Mind"). The Achievements page shows every available achievement, its unlock criteria, and what percentage of users have unlocked it, giving administrators a view of organizational engagement patterns.

## What You'll See

When you navigate to **Achievements** in the admin sidebar, the page loads achievement data and renders a list of all available achievements. Each achievement displays:

- **Name** — The achievement badge name.
- **Description** — What you need to do to unlock it.
- **Category** — Which category the achievement belongs to.
- **Total unlocked** — How many users have earned this achievement.
- **Unlock percentage** — The percentage of ranked users who have unlocked it, indicating rarity.

The achievement data is rendered client-side by `AdminApp.renderAchievements()` after the page loads.

## Achievement Categories

Achievements are organized into 8 categories:

### First Steps

Introductory achievements for getting started:

| Achievement | Criteria |
|-------------|----------|
| **Hello World** | Submit your first prompt |
| **First Spark** | Start your first AI session |
| **Tool Time** | Use your first AI skill |
| **Skill Smith** | Create your first custom skill |
| **Delegation** | Spawn your first subagent |

### Milestones

Progressive achievements based on cumulative activity:

| Achievement | Criteria |
|-------------|----------|
| **Getting Warmed Up** | Complete 5 AI sessions |
| **Quarter Century** | Complete 25 AI sessions |
| **Centurion** | Complete 100 AI sessions |
| **Toolbox** | Use 50 tool actions |
| **Power User** | Use 250 tool actions |
| **Tool Titan** | Use 1,000 tool actions |
| **Kilobyte Mind** | Earn 1,000 total XP |
| **Conversationalist** | Submit 50 prompts |
| **Prompt Pro** | Submit 500 prompts |
| **Team Builder** | Spawn 10 subagents |

### Exploration

Achievements for breadth of usage across skills, plugins, and models:

| Achievement | Criteria |
|-------------|----------|
| **Skill Explorer** | Use 3 different skills |
| **Skill Collector** | Use 10 different skills |
| **Skill Master** | Use 20 different skills |
| **Plugin Pioneer** | Use 3 different plugins |
| **Model Sampler** | Use 2 different AI models |
| **Model Connoisseur** | Use 4 different AI models |

### Creation

Achievements for building custom skills:

| Achievement | Criteria |
|-------------|----------|
| **Skill Artisan** | Create 5 custom skills |
| **Skill Factory** | Create 10 custom skills |

### Streaks

Achievements for consecutive daily usage:

| Achievement | Criteria |
|-------------|----------|
| **On a Roll** | Maintain a 3-day usage streak |
| **Week Warrior** | Maintain a 7-day usage streak |
| **Fortnight Force** | Maintain a 14-day usage streak |
| **Monthly Maven** | Maintain a 30-day usage streak |

Streak achievements check both current and longest streak, so they persist even after a streak is broken.

### Ranks

Achievements tied to reaching specific rank tiers:

| Achievement | Criteria |
|-------------|----------|
| **Token Tinkerer** | Reach rank 3 (150 XP) |
| **Neural Navigator** | Reach rank 5 (800 XP) |
| **Pipeline Architect** | Reach rank 7 (3,000 XP) |
| **Superintelligence** | Reach rank 10 (12,000 XP) |

### Tokens

Achievements based on total token consumption:

| Achievement | Criteria |
|-------------|----------|
| **Token Spender** | Use 10,000 total tokens |
| **Token Whale** | Use 100,000 total tokens |
| **Megabyte Mind** | Use 1,000,000 total tokens |

### Special

Achievements for unique usage patterns:

| Achievement | Criteria |
|-------------|----------|
| **Early Bird** | Use AI before 7:00 AM |
| **Night Owl** | Use AI after 10:00 PM |
| **Weekend Warrior** | Use AI on a Saturday or Sunday |
| **Error Handler** | Encounter and recover from 10 errors |

Time-based achievements (Early Bird, Night Owl, Weekend Warrior) are checked by examining the timestamps of recorded usage events.

## How Achievements Are Checked

Achievements are evaluated automatically during the gamification recalculation process, which runs each time the Leaderboard page loads. The system:

1. Fetches each user's event counts (sessions, tool uses, custom skills, errors, prompts, subagents).
2. Checks each achievement category against the user's metrics.
3. Inserts any newly earned achievements into the `employee_achievements` table using `ON CONFLICT DO NOTHING`, so achievements are only recorded once.

Achievements are permanent — once unlocked, they cannot be lost, even if the underlying metric later decreases (e.g., skills being deleted).

## Rarity and Unlock Percentage

The unlock percentage for each achievement is calculated as:

```
unlock_percentage = (users_who_unlocked / total_ranked_users) * 100
```

Where `total_ranked_users` is the count of users in the `employee_ranks` table (users who have at least one recorded usage event). This gives administrators insight into which achievements are common versus rare, helping identify engagement gaps.

For example, if "First Spark" shows 95% but "Week Warrior" shows 8%, it indicates most users try the platform but few maintain daily usage habits.

## Troubleshooting

**No achievements appear** — The achievements data loads asynchronously. If nothing renders, check the browser console for JavaScript errors. The data endpoint may not be returning results if no users have any recorded events.

**Unlock percentages are all 0%** — The gamification data needs to be recalculated. Visit the Leaderboard page (which triggers recalculation) or use the `POST /admin/gamification/recalculate` API endpoint.

**An achievement should be unlocked but isn't** — Achievement checks run during gamification recalculation. Visit the Leaderboard page to trigger a full recalculation, then return to Achievements. Also verify the user's events are being recorded under the correct user ID.

**Time-based achievements (Early Bird, Night Owl, Weekend Warrior) not unlocking** — These achievements check the `created_at` timestamp of usage events. The time zone used is the database server's UTC time, not the user's local time. A user working at 8 AM local time might qualify for "Early Bird" if their time zone offset places them before 7 AM UTC.
