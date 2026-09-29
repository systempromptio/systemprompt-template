---
title: "Measure Which Skills Are Used"
description: "Step-by-step: sync the skill inventory, resolve name collisions, wait for the first snapshot, and read every column of the Analysis Skills table."
author: "systemprompt.io"
slug: "analysis-measure-skills"
keywords: "skill usage, inventory sync, name collision, binding, attributed use, version hash, p95, marketplace, retained snapshot"
kind: "guide"
public: true
tags: ["enterprise", "admin", "analytics"]
published_at: "2026-09-16"
updated_at: "2026-09-16"
after_reading_this:
  - "Sync the skill inventory and confirm it landed"
  - "Resolve a name collision by binding an entry to the right managed resource"
  - "Read every KPI and every column on the Skills table without guessing"
  - "Open the conversations behind one skill"
related_docs:
  - title: "Analysis: Measure, Improve and Publish a Skill"
    url: "/documentation/analysis"
  - title: "Versions: Sources, Generations and Compare"
    url: "/documentation/analysis-versions"
---

# Measure which skills are used

**TL;DR:** Open `/admin/analysis/skills`, pick a window and a marketplace in the scope strip, then read the three views: **Overview** (one row per marketplace: entitled → installed → active), **Activity** (invocations over the window) and **Skills** (the table). Press **Sync inventory** on the Skills tab when the inventory is behind. Figures are per marketplace and per time window.

## Prerequisites

- An administrative account on the admin console.
- At least one marketplace imported or pinned. If the Skills page shows an onboarding strip with **Import or pin a marketplace** unticked, start at [Code ↔ Instance](/documentation/services-sync).

## Step 1: Open Skills and pick a marketplace

Go to `/admin/analysis/skills`. The page opens on the **Overview** tab with **All marketplaces** in scope.

The scope strip above the tabs holds two choices — the **window** (7, 30, 90 days or a year) and the **marketplace**. Both are links, so a choice is bookmarkable, and both survive a change of tab: pick a marketplace on Overview and the Activity chart and the Skills table show only its skills. The tabs are:

- **Overview** — one row per marketplace: people entitled, consumers holding a verified receipt (with the install rate over the entitled), people active in the window, skills used of those declared, invocations, conversations, cost and the judge's mean. The marketplace name opens its Skills table; the invocation count opens its Activity chart.
- **Activity** — invocations over the window as one chart (per day, per week for a year), with the skills it draws beneath it.
- **Skills** — the table this guide reads, grouped by marketplace and plugin, with the **Client**, **Sort** and search filters. A skill invoked on the instance is credited to the plugin its key names, whichever plugin carries the hooks.

## Step 2: Sync the inventory

Press **Sync inventory** in the toolbar of the Skills tab. It captures every configured skill and publishes the latest revision immediately, instead of waiting for the scheduled pass.

**Expected result:** the meta line reads `… · synced a few seconds ago`. If no marketplace caption's version chip changed, nothing on disk changed, which is the correct outcome for a repeat sync.

## Step 3: Resolve name collisions

If a red **Name collisions** notice appears, some skills exist both as files in `services/` and as an imported managed resource with the same id. Metrics cannot attach until you choose which one counts.

1. Expand **Resolve**.
2. For each row, pick the managed resource in the dropdown.
3. Press **Bind**.

A row with `no imported resource to bind to` has nothing to choose from yet; sync again after the resource is imported.

## Step 4: Wait for the first snapshot

Figures appear once a snapshot exists for the bound resource. The inventory refresh runs every minute; the feedback fact and snapshot passes run every five seconds. Give it a minute or two, then reload.

Until then the Invocations column shows one of three words:

| Word | Meaning |
|---|---|
| `unbound` | The entry is not bound to a managed resource, so nothing can attach to it. |
| `pending` | Bound, waiting for its first snapshot. |
| `suppressed` | The entry is excluded from measurement. |

## Step 5: Read the KPI tiles

| Tile | What it counts |
|---|---|
| **Skills** | Skills in this marketplace, with how many are measured. |
| **Attributed use** | Invocations carrying verified evidence. |
| **Failing** | Skills with a name collision or more than 10% failed requests. |
| **Requests** | Attributed requests, with the failed count beside it. |
| **Spend** | Related spend. It overlaps across skills and must never be summed. |

The tiles are organisation-wide for the window and include traffic not yet attributed to any skill, so they will not add up to the sum of the rows. That is intended.

## Step 6: Read the columns

The page opens on every marketplace, each under its own caption with the version chip it serves. Use the **Marketplace** field to narrow to one, **Window** for 7, 30, 90 or 365 days, and **Skill** to search. **All columns** reveals three extra columns; **Fewer columns** hides them again. **Export** opens the export dialog: pick the window, the format (CSV, JSON, JSON Lines or Markdown) and the columns, and the dialog counts the rows and cells before you download. Every table page in the console carries the same button.

| Column | Definition |
|---|---|
| **Skill** | The skill's resource key, and the link to its conversations. Hovering it reads "<name> — conversations that used this skill". |
| **Health** | Red for a name collision, an unavailable or withdrawn entry, or more than 10% failed requests. Amber for unbound or audience-less skills, and for measured skills with no attributed use. Grey while a bound resource waits for its first snapshot. |
| **Invocations** | Invocations attributed to this skill, with the share carrying verified evidence as a percentage beside it. |
| **Users** | Distinct people who invoked the skill in the window, from the retained facts. An em-dash means identity was never available. The people *entitled* to a marketplace are on its caption, not on the row. |
| **Requests** | Attributed requests, with failures in red. |
| **Tokens** | Input and output tokens on attributed requests. Behind **All columns**. |
| **Cost** | Related spend on this skill's attributed requests. Overlaps across skills. |
| **p95** | 95th percentile latency, measured over the requests related to the skill in this window. A value marked `≤` is the snapshot histogram's upper bound, shown only when no request latency exists. |
| **Assessed** | Scored conversations over assessed conversations. Behind **All columns**. |
| **Revision** | The revision the inventory last captured for this skill. Behind **All columns**. |
| **Version** | The content hash of the marketplace version serving this skill. Opens that marketplace's history on Versions. |
| **Actions** | **Catalog** opens the skill in the marketplace catalog, where its contents and entitlement live. The conversations link is the skill name itself, in the first column, not an action here. |

A skill carried by two plugins appears under each one. Group rollups count it once.

## Step 7: Open one skill's conversations

Click the skill name in the Skills table. The skill's page opens at `/admin/analysis/skills/<plugin:skill>` and lists every conversation with an attributed invocation: listing every session with an attributed invocation: session, consumer, first and last seen, invocations, requests, tokens, cost and the latest retained assessment.

**Verification:** the conversation count on that page should be consistent with the Users column you just read for the same window. If the skill link lands on an empty table, no conversation in the window invoked the skill yet: widen the window, or run one through the gateway and wait for the minute-by-minute rollup.

## Troubleshooting

### The table is empty for this marketplace

**Symptom:** "No skills here".
**Cause:** No skill matches the current marketplace and search.
**Solution:** Press **Clear** to drop the search, change the **Marketplace** field, or sync the inventory.

### Every p95 reads the same value with a ≤

**Symptom:** The p95 column shows `≤ 16.78 s` on many rows.
**Cause:** No request latency exists for those skills in the window, so the snapshot histogram's power-of-two bucket bound is shown.
**Solution:** Nothing to fix; once requests related to the skill carry latency, a measured percentile replaces the bound.

### Spend looks too high across the page

**Symptom:** Adding the Cost column gives a larger number than the Spend tile.
**Cause:** Related spend overlaps when several skills run in one conversation.
**Solution:** Read costs per skill and never sum the column.

## Related pages

- [Analysis overview](/documentation/analysis)
