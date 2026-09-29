---
title: "Usage, Adoption & Productivity Analytics"
description: "Track AI usage, adoption, and productivity on the admin analytics dashboard: request volume, weekly active users, and honestly labelled code metrics, scoped by group, project, or user."
author: "systemprompt.io"
slug: "enterprise-analytics"
keywords: "analytics, usage, adoption, productivity, wau, projects, code metrics, dashboards"
kind: "guide"
public: true
tags: ["enterprise", "analytics", "admin"]
published_at: "2026-08-25"
updated_at: "2026-09-28"
after_reading_this:
  - "Navigate the Overview, Models, Skills, Tools, Sessions, and Cost analytics tabs"
  - "Switch between period presets from 15 minutes to 90 days, or set a custom range"
  - "Read WAU and requests per user per day, scoped to a group, project, or user"
  - "Interpret the code-impact metrics as the proxies they are"
  - "Understand what stands between usage data and an outcome metric"
related_docs:
  - title: "Cost Management, Budgets & FinOps"
    url: "/documentation/enterprise-cost-management"
  - title: "Dashboard"
    url: "/documentation/dashboard"
---

# Usage, Adoption & Productivity Analytics

**TL;DR:** `/admin/analytics` is a server-rendered dashboard with six tabs — Overview, Models, Skills, Tools, Sessions, and Cost (the last covered in [Cost Management](/documentation/enterprise-cost-management)). It answers who is using the platform, how much, and what they produce — with period presets from 15 minutes to 90 days, and every view scoped by group, project, or user.

## Periods and filters

Every tab shares the same controls: period presets of **15 minutes, 1 hour, 24 hours, 7 days, 30 days and 90 days**, plus a **custom range**, a day/week bucket toggle, and **group** and **project** selectors. Scope and window live in the query string, so a view can be bookmarked or shared. Groups and projects are rows an operator manages (and, when SSO is configured, AD groups can map into them) — see [User & Access Management](/documentation/enterprise-user-access). The template ships with none declared, so until you create some the selectors only offer the whole instance. The dashboard is for console roles; a non-admin sees only their own activity, through their profile and history pages.

## Overview tab

The Overview puts the whole instance on one screen:

- **Request volume** — total requests over the period, with trend series.
- **Error rate** — failed requests as a share of total.
- **Active users** — distinct users per bucket, charted over time.
- **Model mix, latency split, anomalies and code impact** — each covered below or on its own page.

The other tabs go deeper: **Models** is per-model gateway behaviour, **Skills** links to skill analysis, **Tools** is MCP execution health, and **Sessions** is client-reported session cost and rating.

For CLI cross-checks and scripted reporting:

```bash
systemprompt analytics overview
systemprompt analytics requests stats
systemprompt analytics costs breakdown --by user --since 7d
systemprompt analytics conversations list --since 7d --source gateway
```

## Adoption

The Overview also measures adoption rather than raw traffic:

- **Weekly active users (WAU)** with period-over-period deltas.
- **Requests per user per day** — the intensity metric: is usage broad and shallow, or concentrated?
- **Top users** — a leaderboard of the heaviest users in the period, sortable by requests, cost, tokens, or last activity.

## Code impact: productivity proxies

The code-impact section reports what Claude Code sessions produce. These metrics are **proxies, and are labelled as such in the UI** — they indicate direction, not ground truth:

- **AI lines added and removed** — lines applied through Edit/Write tool calls, as observed by the session hooks.
- **AI edit operations** — the count of those applied edits.
- **Permission-grant rate** — how often users approve the tool actions the model requests; a rough trust signal.
- **Commits and committed lines** — commits observed through Claude Code sessions, charted beside AI lines (the UI notes these are different measurement frames).

Two limits are worth stating plainly:

- **Tab-acceptance rate is not measurable.** Claude Code emits no accept/reject signal for completions, and no manual-LOC baseline exists, so a true acceptance metric would require an IDE-level integration that does not exist today.
- **Commits made outside Claude Code are invisible.** Full commit analytics require nominating an authoritative SCM and an identity mapping into it.

Both limitations have their canonical home on the [Enterprise Roadmap](/documentation/enterprise-roadmap).

## Verification

Use the commands and UI checks above against the configured instance. Repository maintainers can run `just test-unit` and `just test-integration`; a passing fixture test does not establish that a deployed integration is configured.

## From usage to outcomes

Everything above measures **consumption**. Mapping consumption to **outcomes** is a separate question, and it needs a decision before it needs engineering.

### What is measured today

Every request through the gateway records input tokens, output tokens, cache-read and cache-creation tokens, cost, latency, model, provider, and the user, session and trace it belongs to. Those rows roll up per user per day, which is what the dashboards and the **Usage** tab on each user's page (`/admin/users/{user_id}?tab=usage`) read.

Two honest caveats. **Cost is computed, not billed** — it comes from a model price catalogue applied to observed token counts, so it is an accurate estimate rather than an invoice. And the Cost tab's customer view deliberately carries no supplier cost figure; spend lives in the internal view, so that a report shared with a team shows usage without exposing budget.

### The limit worth stating

The only outcome-shaped data captured today is **self-reported by the model**: at the end of a Claude Code session a summariser records whether the stated goal was achieved, a quality score, and a goal-to-outcome mapping. It is a useful coaching signal and a reasonable way to spot sessions that went badly. It is not a business metric, and it should not be presented as one — it is a model's opinion of its own work, not an observed result in a system of record.

Nothing today joins AI usage to a ticket being closed, a deal advancing, an incident being resolved, or a release shipping.

### The decision required

To map usage to outcomes you first have to agree what an outcome *is*. The candidates, in rough order of how directly they are observable:

- **Issue-tracker transitions** — cycle time from in-progress to done, reopen rate, throughput per person per sprint. The most concrete option for engineering work, and the easiest to game if it becomes a target.
- **Source control** — pull requests merged, review turnaround, change failure rate. Closest to delivery, but only meaningful once commits made outside Claude Code are visible.
- **CRM records** — opportunities advanced, cases resolved. The right measure for commercial rather than engineering usage.
- **Ratified self-report** — keep the session goal signal, but have a human confirm or reject it, converting an opinion into a label.

Whichever is chosen, two dependencies follow and neither is optional:

1. **Credentials to sync that system into systemprompt.io.** Each external system needs an API credential before its data can be correlated with anything. Without it there is no outcome side of the join.
2. **An identity mapping rule** from the external account to a platform user — a tracker account id, a CRM username, a commit-author email. This is the harder half, and it is a decision for the deploying organisation.

Until an outcome is defined and its source system connected, the platform can report what AI cost and what it produced in code, and it should not claim more than that.
