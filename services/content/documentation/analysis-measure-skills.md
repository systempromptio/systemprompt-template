---
title: "Measure Which Skills Are Used"
description: "Step-by-step: pick a window and a marketplace, read marketplace adoption, sync the skill inventory, and read every KPI and column of the Analysis Skills table."
author: "systemprompt.io"
slug: "analysis-measure-skills"
keywords: "skill usage, inventory sync, adoption, entitled, installed, active, install rate, invocations, version hash, p95, marketplace, judge"
kind: "guide"
public: true
tags: ["enterprise", "admin", "analytics"]
published_at: "2026-09-16"
updated_at: "2026-09-28"
after_reading_this:
  - "Read a marketplace's adoption as entitled → installed → active"
  - "Sync the skill inventory and confirm it landed"
  - "Read every KPI and every column on the Skills table without guessing"
  - "Open the conversations behind one skill"
related_docs:
  - title: "Analysis: The Record of Every Conversation and Skill"
    url: "/documentation/analysis"
  - title: "Create a Conversation and See It Land"
    url: "/documentation/analysis-test-conversation"
  - title: "Versions: Marketplace Hashes, History and Compare"
    url: "/documentation/analysis-versions"
---

# Measure which skills are used

**TL;DR:** Open `/admin/analysis/skills`, pick a window and a marketplace in the scope strip, then read the three views: **Overview** (KPI tiles, then one row per marketplace: entitled → installed → active), **Activity** (invocations over the window) and **Skills** (the table). Invocations come from the harness hooks; everything else on a row is read from the conversations those invocations ran in. Figures are per marketplace and per time window.

## Prerequisites

- An administrative account on the admin console.
- At least one marketplace declared on the instance — this one ships `enterprise-demo`, which carries the `systemprompt` plugin. Kits pinned on [Code ↔ Instance](/documentation/services-sync) add theirs.
- Claude Code (or another client with the plugin's hooks) connected through the gateway. Without the hooks there are no invocations to count.

## Step 1: Open Skills and pick a scope

Go to `/admin/analysis/skills`. The page opens on the **Overview** tab with **All marketplaces** in scope.

The scope strip above the tabs holds two choices — the **Window** (7 days, 30 days, 90 days or 1 year) and the **Marketplace**. Both are links, so a choice is bookmarkable, and both survive a change of tab: pick a marketplace on Overview and the Activity chart and the Skills table show only its skills.

## Step 2: Read the KPI tiles

| Tile | What it counts |
|---|---|
| **Skills used** | Distinct `plugin:skill` keys with at least one hook-reported invocation, with the invocation count beneath. |
| **People** | The most people any one skill reached, of those entitled and installed. Entitlement is resolved from the access-control rules. |
| **Install rate** | Entitled people holding a verified installation receipt, over everyone entitled. Consumers the rules do not reach are counted beside it. |
| **Conversations** | Gateway conversations whose harness session invoked a skill. |
| **Cost** | Priced spend of those skill conversations. |
| **Errors** | Failed requests, with denied tool calls beside them, inside skill conversations. |
| **AI score** | The mean of the judge's one completion score over judged skill conversations, with how many were judged. |

## Step 3: Read marketplace adoption

The **Overview** table has one row per marketplace:

| Column | Meaning |
|---|---|
| **Marketplace** | Name, plugin and skill counts, and the version chip it serves; the chip opens its history on Versions. |
| **Entitled** | People the access-control rules reach for this marketplace. |
| **Installed** | Consumers holding a verified installation receipt for one of its skills, with the install rate over the entitled; installs by people the rules do not reach show as *+N outside*. |
| **Active** | Distinct people who invoked one of its skills in the window, with the rate over the installed. |
| **Reach** | Installed and active as shares of the entitled, on one track. |
| **Skills** | Distinct skills invoked, over those declared. |
| **Invoked** | Hook-reported invocations; the number opens the Activity chart for that marketplace. |
| **Conv.**, **Cost**, **AI** | Conversations, their spend, and the judge's mean completion. |
| **Last install** | The most recent verified installation receipt. |

**Activity** draws invocations over the window as one chart (per day, per week for a year), with the skills it draws listed beneath it.

## Step 4: Sync the inventory

On the **Skills** tab, an administrator sees **Sync inventory** in the toolbar. It publishes every configured skill into the managed inventory now, instead of waiting for the scheduled pass, and records a new marketplace version if the bytes it serves changed. A repeat sync with nothing changed on disk moves no version chip, which is the correct outcome.

The same toolbar shows **N unused** when configured skills had no invocation in the window; it lists each with its marketplace and the number of people entitled to it.

## Step 5: Read the Skills table

The table is grouped by marketplace (its heading links to Versions) and then by plugin. A skill invoked on the instance is credited to the plugin its key names, whichever plugin carries the hooks. The filter ribbon narrows by client, plugin and search, and sorts; **Export** downloads the `analysis-skills` dataset for the same scope, in CSV, JSON, JSON Lines or Markdown.

| Column | Definition |
|---|---|
| **Skill** | The skill's name, linking to its own page, with the attributed share and the date it was first used. |
| **Invoked** | Hook-reported invocations (slash command and `Skill` tool). |
| **14 days** | Invocations per day over the last fourteen days. |
| **People** | Distinct people who invoked it, over those entitled. |
| **Installs** | Consumers holding a verified installation receipt. |
| **Conv.** | Gateway conversations whose session invoked it; hover for turns. |
| **Tokens**, **Cost** | Tokens and spend of those conversations. |
| **/inv** | Cost per invocation. |
| **Tool calls** (wrench) | Tool calls in those conversations, failed in brackets; opens them. |
| **Artifacts** (file) | Tool calls that produced something viewable — files, cards, UI, retained bodies; opens them. |
| **Err** | Failed requests, with denied tool calls beside them. |
| **p95** | 95th percentile turn latency. |
| **Models**, **Agents** | Models and clients seen. |
| **Last** | When it was last invoked. |
| **AI** | The judge's mean completion over judged conversations, with judged over total. |

Tick rows to enable the bulk bar and export only those skills.

## Step 6: Open one skill

Click a skill's name. Its page opens at `/admin/analysis/skills/<plugin:skill>`: the same figures over time, a breakdown by model, client, group, project, person, version or outcome, and the conversations behind them — each with its person, tool calls, artifacts, denials, tokens, cost and judge score. **Catalog** opens the skill in the Platform catalog, where its contents and entitlement live.

**Verification:** the conversation count on that page matches the skill's **Conv.** cell for the same window. If the skill has invocations but no conversations, see [Create a Conversation and See It Land](/documentation/analysis-test-conversation#when-it-does-not-appear).

## Troubleshooting

### No skill was invoked in this window

**Symptom:** The Skills tab shows "No skill was invoked in this window".
**Cause:** No hook-reported invocation falls in the window for the chosen marketplace.
**Solution:** Widen the window or pick **All marketplaces**, and check that the plugin carrying the governance hooks is installed in the client.

### A skill has invocations but no cost

**Symptom:** **Invoked** and **People** are filled, but **Conv.**, tokens and cost are zero.
**Cause:** The sessions that invoked it produced no gateway conversation with the same session id — the inference went elsewhere, or the client did not send its session id.
**Solution:** See [Create a Conversation and See It Land](/documentation/analysis-test-conversation#when-it-does-not-appear).

### Spend looks too high across the page

**Symptom:** Adding the Cost column gives a larger number than the Cost tile.
**Cause:** Cost is related spend: one conversation that invokes several skills is counted under each of them.
**Solution:** Read costs per skill and never sum the column.

## Related pages

- [Analysis overview](/documentation/analysis)
- [Create a Conversation and See It Land](/documentation/analysis-test-conversation)
- [Versions: Marketplace Hashes, History and Compare](/documentation/analysis-versions)
