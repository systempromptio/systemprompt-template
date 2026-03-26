---
title: "Gamification & Leaderboard"
description: "XP tracking, rank tiers, achievement badges, streaks, and department leaderboards. Drive AI adoption across your organisation with gamification."
author: "systemprompt.io"
slug: "gamification"
keywords: "gamification, leaderboard, XP, achievements, badges, streaks, ranks, adoption"
kind: "guide"
public: true
tags: ["analytics", "gamification", "leaderboard"]
published_at: "2026-03-25"
updated_at: "2026-03-25"
after_reading_this:
  - "Understand how XP is earned from AI usage"
  - "Know the 10 ranking tiers and 35 achievement badges"
  - "Use leaderboards to track AI adoption by user and department"
related_docs:
  - title: "Dashboard"
    url: "/documentation/dashboard"
  - title: "Activity Tracking"
    url: "/documentation/activity-tracking"
  - title: "Metrics Reference"
    url: "/documentation/metrics-reference"
---

# Gamification & Leaderboard

The platform includes a gamification system that tracks user engagement with AI through XP, ranks, achievements, and streaks. Use it to drive AI adoption, identify power users, and measure team-level engagement.

## Leaderboard

The leaderboard ranks all users by XP earned through AI usage. Two views are available:

**All Users** — Individual rankings with XP, streak, achievement count, and last active time. Top 3 positions receive special visual styling. Click any row to navigate to that user's detail page.

**By Department** — Aggregated team scores showing total XP, average XP per user, user count, and the top contributor per department. Departments are sorted by total XP.

Summary stats at the top show Ranked Users, Average XP, total Achievements Unlocked, and Active Streaks across the organisation.

## How XP Is Calculated

Each AI interaction earns XP:

| Activity | XP per Event |
|----------|-------------|
| AI session started | 5 XP |
| Tool/skill used | 10 XP |
| Prompt submitted | 3 XP |
| Subagent spawned | 15 XP |
| Error encountered | 2 XP |
| Custom skill created | 50 XP (first-time unique skill bonus: 25 XP) |
| Token usage | 1 XP per 1,000 tokens |
| Streak bonus | 15 XP per streak day |

Administrators can award bonus XP through the XP ledger. Total XP = base event XP + token XP + bonus ledger XP.

## Ranking Tiers

10 rank tiers based on total XP:

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

Ranks are permanent — there is no rank decay.

## Achievements

35 achievements across 8 categories:

### First Steps (5 badges)
Getting started milestones: first prompt, first session, first tool use, first custom skill, first subagent.

### Milestones (10 badges)
Cumulative activity: 5/25/100 sessions, 50/250/1000 tool actions, 1000 XP, 50/500 prompts, 10 subagents.

### Exploration (6 badges)
Breadth of usage: 3/10/20 different skills, 3 different plugins, 2/4 different AI models.

### Creation (2 badges)
Building custom skills: 5 and 10 custom skills created.

### Streaks (4 badges)
Consecutive daily usage: 3-day, 7-day, 14-day, and 30-day streaks.

### Ranks (4 badges)
Reaching rank milestones: Token Tinkerer (rank 3), Neural Navigator (rank 5), Pipeline Architect (rank 7), Superintelligence (rank 10).

### Tokens (3 badges)
Token consumption milestones: 10K, 100K, and 1M total tokens.

### Special (4 badges)
Unique patterns: Early Bird (before 7 AM), Night Owl (after 10 PM), Weekend Warrior, Error Handler (10 recovered errors).

Achievements are permanent — once unlocked, they cannot be lost.

## Streaks

A streak counts consecutive calendar days with at least one usage event:

- **Current streak** — Consecutive days up to today/yesterday. Resets to 0 if a day is missed.
- **Longest streak** — All-time best. Never decreases.

## Department Scores

The department leaderboard uses `DepartmentScore` to aggregate per-department:

| Field | Description |
|-------|-------------|
| `department` | Department name |
| `total_xp` | Sum of all XP in the department |
| `avg_xp` | Average XP per user |
| `user_count` | Number of ranked users |
| `top_user_name` | Highest-XP user |
| `top_user_xp` | That user's XP |

## User Profile

Each user has a `UserGamificationProfile` showing:

- Current rank level and name
- Total XP and XP to next rank
- Events count, unique skills count, unique plugins count
- Current and longest streak
- Unlocked achievements list
- Global rank position
