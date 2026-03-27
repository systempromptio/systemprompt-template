---
title: "Leaderboard"
description: "View individual and department rankings based on XP earned through AI usage. Track streaks, achievements, and competitive standings across your organization."
author: "systemprompt.io"
slug: "leaderboard"
keywords: "leaderboard, rankings, XP, gamification, streaks, departments, ranks"
kind: "guide"
public: true
tags: ["gamification", "leaderboard", "admin"]
published_at: "2026-02-18"
updated_at: "2026-02-18"
after_reading_this:
  - "Read individual and department leaderboards"
  - "Understand how XP is calculated from AI usage"
  - "Know the 10 ranking tiers from Spark to Superintelligence"
  - "Track usage streaks and achievement counts"
related_docs:
  - title: "Achievements"
    url: "/documentation/achievements"
  - title: "Marketplace"
    url: "/documentation/marketplace"
  - title: "Plugins"
    url: "/documentation/plugins"
---

# Leaderboard

**TL;DR:** The Leaderboard ranks all users in your organization by XP earned through AI usage. Users earn XP from sessions, tool usage, prompts, subagent spawns, custom skill creation, and token consumption. The page has two tabs — All Users (individual rankings) and By Department (aggregated team scores). XP determines your rank tier, from Spark (beginner) all the way up to Superintelligence.

## What You'll See

When you navigate to **Leaderboard** in the admin sidebar, the page displays:

- **Summary stats ribbon** — Four stat cards showing Ranked Users, Average XP, total Achievements Unlocked, and Active Streaks across the organization.
- **Tab bar** — Switch between "All Users" (individual leaderboard) and "By Department" (department-level aggregates).
- **Leaderboard table or department cards** — Depending on the selected tab.

The leaderboard recalculates automatically each time the page loads, ensuring scores, streaks, and achievements are up to date.

## All Users Tab

The individual leaderboard is a table with the following columns:

| Column | Description |
|--------|-------------|
| **#** | Rank position. The top 3 positions receive special visual styling |
| **User** | Avatar with initials and display name |
| **Department** | Department badge, or a dash if unassigned |
| **XP** | Total experience points earned |
| **Streak** | Current consecutive-day usage streak (in days), or a dash if no active streak |
| **Achievements** | Count of achievements unlocked |
| **Last Active** | Relative time since the user's last activity (e.g. "2 hours ago") |

Click any row to navigate to that user's detail page.

The leaderboard displays up to 100 users, sorted by total XP in descending order.

## By Department Tab

The department view shows a card grid where each card represents one department. Each department card displays:

- **Department name** — The department heading.
- **Total XP** — Sum of all XP earned by users in this department.
- **Avg XP** — Average XP per user in the department.
- **Users** — Count of ranked users in the department.
- **Top user** — The highest-XP user in the department, with their individual XP shown.

Departments are sorted by total XP (highest first). Only users with a non-empty department field are included.

## How XP Is Calculated

XP is computed from recorded usage events. Each event type awards a specific amount:

| Activity | XP per event |
|----------|-------------|
| AI session started | 5 XP |
| Tool/skill used | 10 XP |
| Prompt submitted | 3 XP |
| Subagent spawned | 15 XP |
| Error encountered | 2 XP |
| Custom skill created | 50 XP (first-time unique skill bonus: 25 XP) |
| Token usage | 1 XP per 1,000 tokens |
| Streak bonus | 15 XP per streak day |

Additionally, administrators can award bonus XP through the XP ledger, which is added on top of event-based and token-based XP.

Total XP = base event XP + token XP + bonus ledger XP.

## Ranking Tiers

Your rank tier is determined by total XP:

| Level | Rank Name | XP Required |
|-------|-----------|-------------|
| 1 | Spark | 0 |
| 2 | Prompt Apprentice | 50 |
| 3 | Token Tinkerer | 150 |
| 4 | Context Crafter | 400 |
| 5 | Neural Navigator | 800 |
| 6 | Model Whisperer | 1,500 |
| 7 | Pipeline Architect | 3,000 |
| 8 | Singularity Sage | 5,000 |
| 9 | Emergent Mind | 8,000 |
| 10 | Superintelligence | 12,000 |

Once you reach a tier's threshold, you hold that rank until you earn enough XP for the next tier. There is no rank decay.

## Streaks

A usage streak counts consecutive calendar days with at least one recorded usage event. The system tracks two streak values:

- **Current streak** — How many consecutive days (up to and including today or yesterday) the user has been active. Resets to 0 if the user misses a day.
- **Longest streak** — The user's all-time best streak. This value never decreases.

Streaks are calculated from the `employee_daily_usage` table, which aggregates daily event counts per user.

## Summary Stats

The four stat cards at the top of the page are computed client-side from the leaderboard table data:

- **Ranked Users** — Count of users in the leaderboard table.
- **Avg XP** — Mean XP across all ranked users.
- **Achievements Unlocked** — Sum of all achievement counts across all users.
- **Active Streaks** — Count of users with a current streak greater than 0.
