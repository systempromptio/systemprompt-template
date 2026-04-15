---
title: "Enterprise Demo Walkthrough"
description: "A tour of the Enterprise Demo seed data, sweep harness, and governance scenarios you can run locally."
author: "systemprompt.io"
published_at: "2026-04-05T09:00:00Z"
slug: "demo-walkthrough"
keywords: "demo, walkthrough, sweep, seed"
kind: "tutorial"
image: "/files/images/logo.png"
category: "documentation"
tags:
  - demo
  - tutorial
public: true
---

# Enterprise Demo Walkthrough

Run `./demo/00-preflight.sh` to confirm the runtime is up, then
`./demo/01-seed-data.sh` to populate skills, contexts, files, governance
decisions, page views, and ingested content. Finally, `./demo/sweep.sh`
walks every domain command and reports pass/fail.

## What the seed script populates

- **Skills** — synced from `services/skills/*.yaml` into the
  `agent_skills` table.
- **Contexts** — three sample contexts (`demo-review`,
  `incident-response`, `onboarding`).
- **Files** — fixtures from `demo/fixtures/` uploaded across the four
  storage categories (images, documents, audio, and other).
- **Governance decisions** — five sessions per agent, mixing allows,
  scope restrictions, secret injections, and blocklist hits.
- **Page views** — 100 synthetic page views spread across paths,
  referrers, countries, and devices for traffic analytics.
- **Content** — markdown under `content/blog/` and
  `content/documentation/` is indexed into `markdown_content`.
