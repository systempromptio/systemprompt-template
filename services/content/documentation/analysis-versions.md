---
title: "Versions: Marketplace Hashes, History and Compare"
description: "Every marketplace by the content hash of what it serves: how a version is computed and recorded, how to read its history and inline diff, how to compare two versions skill by skill, and where delivery evidence lives."
author: "systemprompt.io"
slug: "analysis-versions"
keywords: "versions, marketplace hash, content hash, history, compare, diff, distribution, receipts, coverage, bundle, base, kit"
kind: "guide"
public: true
tags: ["enterprise", "admin", "operations"]
published_at: "2026-09-16"
updated_at: "2026-09-23"
after_reading_this:
  - "Say what a marketplace version is, and why a base and a bundled marketplace carry the same kind of identity"
  - "Read a marketplace's history and tell a changed skill from an added or removed one"
  - "Compare two versions and read the figures under each, skill by skill"
  - "Find whether devices received the version being served"
related_docs:
  - title: "Analysis: Measure, Compare and Publish a Skill"
    url: "/documentation/analysis"
  - title: "Code ↔ Instance: Sources, Planes and Sync"
    url: "/documentation/services-sync"
---

# Versions: marketplace hashes, history and compare

**TL;DR:** `/admin/analysis/versions` lists every marketplace by the content hash of what it serves. A version *is* its hash. Open a marketplace for **History** (every version, what changed, how each performed), **Compare** (two versions side by side, skill by skill) and **Distribution** (what devices received). Analysis observes and compares; experiments happen off the platform.

## What a version is

A marketplace version is the sha256 over everything the marketplace delivers to a consumer: its own `marketplaces/<id>/` directory, every plugin it includes under `plugins/<id>/`, and every skill those plugins ship under `skills/<id>/`. It is computed with the same file walk and digest a kit's `content_hash` and this tree's hash use, so all three read the same way.

Because the hash covers bytes and nothing else:

- A base marketplace (declared in this repository's `services/`) and a bundled one (shipped by a kit the profile pins) have the same kind of identity. The kit's content hash or the base tree hash is recorded beside it as **source** provenance, never as identity.
- Two marketplaces sharing a plugin hash differently only through their own config.
- A plugin that names a skill whose directory is missing still moves the hash: the broken reference is part of the version.

Versions are recorded at boot and on every **Sync inventory now**. A hash seen before reopens its row; a hash that moved closes the previous version at that moment; a marketplace the composition no longer declares is closed and reads as *retired*. History is never rewritten.

A conversation is credited to the version being served when it first invoked one of the marketplace's skills, and stays there whatever is deployed later. The `conversation_rollup` job records that per skill and per conversation in `conversation_skill_facts`, beside the conversation's own record, and both are kept forever — so a version's figures outlive the 90-day raw-event window. Versions recorded before manifests were kept carry the coarser source hash and are marked *source hash*.

## The landing page

One row per marketplace: name, the current version chip, source, plugin and skill counts, how many versions have been recorded, when the current one was first observed, and the window's invocations, users, requests and cost across every version. The window control in the header applies to every figure. A retired marketplace keeps its history and shows a *retired* badge in place of a chip.

## History

One row per version, newest first, the current one tinted.

| Column | Meaning |
|---|---|
| **Version** | The short hash; hover for the full value. `current` marks the one being served; `source hash` marks a version seeded from facts that predate manifests. |
| **Served** | When the version was first observed and, for a closed one, when it stopped. |
| **Plugins · Skills** | What the version's manifest carried. |
| **Change** | Skills added, removed and changed against the version before it, with an inline list behind *what changed*: each moved plugin or skill, its change, and the before → after digest. The oldest row reads *first version*. |
| **Invocations · Users · Requests · Cost · p50 · p95** | The window's figures for the conversations that ran under this version. Invocations are the skills' own; users are every person who ran one of those conversations; requests are their turns; cost is their turn spend with side calls excluded; p50 is the median of the conversations' median latency and p95 the 95th percentile of their p95. |
| **Compare** | Opens Compare with this version as *after* and the one before it as *before*. |

## Compare

Pick a *before* and an *after* version; the two newest are chosen by default. The page shows each version's tiles side by side, then one table with every skill either version carried: its change (added, removed, changed, unchanged — from the manifests' per-skill digests) and its invocations, users, requests, failures, cost and p95 under each version, with the invocation delta.

A version seeded from a source hash has no manifest, so no change can be derived; the page says so and lists skills from their recorded invocations alone.

## Distribution

The delivery pipeline for the marketplace's skills, scoped to the skills its current manifest names:

- **Device coverage** — entitled devices holding the served publication of each skill, acknowledged and verified.
- **Publications** — every reviewed publication, its revision, bundle digest, provenance hash, distribution state and receipt count. A revision link opens its file evidence at `/admin/analysis/revisions/<id>`.
- **Withdrawal proposals** — raised when the upstream source no longer carries a skill; approve or reject here.
- **Deliveries** and **Installation receipts** — the bridge's claims on the outbox and what devices reported back.

The manual review form at the foot records a publication decision by hand.

## Reading requests, cost and latency

Users, requests, spend and latency reach a version through the conversations its skills ran in, each conversation counted once per version. They are *related* figures: they overlap across versions and across skills in the same conversation and must never be summed. Spend is the conversation's turns; side calls such as titles and summaries are excluded. Latency is conversation-level: the median of each conversation's p50 and the 95th percentile of each conversation's p95.

## How pinning or importing a kit shows up here

1. You pin the kit by digest on [Code ↔ Instance](/documentation/services-sync), or change the digest it pins.
2. On the next boot or **Sync inventory now**, each marketplace the kit declares is hashed and recorded with source `bundle:<name>`.
3. A marketplace that already existed with different bytes gets a new version; History shows what changed.
4. Each new skill becomes a managed resource with a captured revision and a normal row on [Skills](/documentation/analysis-measure-skills), whose marketplace caption carries the new hash.

A kit owns its content. This repository owns access, so entitlement for a kit's marketplace is still declared in `rules.yaml`.

## Troubleshooting

### No marketplace versions recorded

**Symptom:** The landing page is empty.
**Cause:** Versions are recorded at boot and on inventory sync, and neither has run since the table was created.
**Solution:** Press **Sync inventory now** on Skills, or restart the server.

### A version shows *no manifest*

**Symptom:** The Change cell reads *no manifest* and Compare cannot derive a diff.
**Cause:** The version was seeded before manifests were kept; only its source hash is known.
**Solution:** Nothing to fix. Every version recorded since carries a manifest.

### Distribution lists every skill

**Symptom:** Skills from other marketplaces appear under Distribution.
**Cause:** The current version has no manifest to scope by.
**Solution:** Sync the inventory so a manifest-bearing version is current.

## Related pages

- [Analysis overview](/documentation/analysis)
- [Code ↔ Instance: Sources, Planes and Sync](/documentation/services-sync)
