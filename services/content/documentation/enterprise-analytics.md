---
title: "Usage, Adoption & Productivity Analytics"
description: "Track AI usage, adoption, and productivity in the admin analytics tabs: request volume, weekly active users, inactive seats, and honestly labelled code metrics."
author: "Astound Digital"
slug: "enterprise-analytics"
keywords: "analytics, usage, adoption, productivity, wau, seats, code metrics, dashboards"
kind: "guide"
public: true
tags: ["enterprise", "analytics", "admin"]
published_at: "2026-08-25"
updated_at: "2026-08-25"
after_reading_this:
  - "Navigate the Overview, Usage, Seats, and Code analytics tabs"
  - "Switch between period presets from 15 minutes to 30 days, or set a custom range"
  - "Read WAU, requests per user per day, and the inactive-seat report"
  - "Interpret the Code tab's productivity proxies for what they are"
related_docs:
  - title: "Organizations, Departments & Hubs"
    url: "/documentation/enterprise-organizations"
  - title: "Cost Management, Budgets & FinOps"
    url: "/documentation/enterprise-cost-management"
  - title: "Dashboard"
    url: "/documentation/dashboard"
---

# Usage, Adoption & Productivity Analytics

**TL;DR:** `/admin/analytics` is a server-rendered dashboard with tabs for Overview, Usage, Seats, and Code (plus Spend, covered in [Cost Management](/documentation/enterprise-cost-management)). It answers who is using the platform, how much, and what they produce — with selectable periods from 15 minutes to 30 days, and every view filterable by organization, department, or user.

## Periods and filters

Every tab shares the same controls: period presets from **15 minutes up to 30 days**, plus a **custom range** picker, and the organization / department / user filters described in [Organizations, Departments & Hubs](/documentation/enterprise-organizations).

## Overview and Usage tabs

The Overview and Usage tabs cover platform utilization:

- **Request volume** — total requests over the period, with daily and weekly series and historical trend.
- **Error rate** — failed requests as a share of total.
- **Active users** — distinct users per bucket, charted over time.

For CLI cross-checks and scripted reporting:

```bash
systemprompt analytics overview
systemprompt analytics requests stats
```

## Seats tab: adoption

The Seats tab measures adoption rather than raw traffic:

- **Weekly active users (WAU)** with period-over-period deltas.
- **Requests per user per day** — the intensity metric: is usage broad and shallow, or concentrated?
- **Top users** — a leaderboard of the heaviest users in the period.
- **Inactive seats** — accounts with no activity within a configurable window of **7, 14, 30, or 90 days**. This is the list to review before a renewal: seats you pay for that nobody uses.

## Code tab: productivity proxies

The Code tab reports what Claude Code sessions produce. These metrics are **proxies, and are labelled as such in the UI** — they indicate direction, not ground truth:

- **AI-authored lines of code** — LOC written by the model in observed sessions.
- **Applied edits** — edits the model proposed that were applied.
- **Permission-grant rate** — how often users approve the tool actions the model requests; a rough trust signal.
- **Commit lines** — lines landing in commits observed through Claude Code sessions, deduplicated and rolled up daily beside AI usage.

Two limits are worth stating plainly:

- **Tab-acceptance rate is not measurable.** Claude Code emits no accept/reject signal for completions, and no manual-LOC baseline exists, so a true acceptance metric would require an IDE-level integration that does not exist today.
- **Commits made outside Claude Code are invisible.** Full commit analytics require nominating an authoritative SCM and an identity mapping into it.

Both limitations have their canonical home on the [Enterprise Roadmap](/documentation/enterprise-roadmap).

## Verified evidence

Every capability on this page is proven by tagged end-to-end tests run against a seeded instance. To replicate: `just start`, then `just e2e-seed --reset`, then the command in the table. Screenshots regenerate with `just e2e-screens`.

| Ref | Verified behaviour | Replicate with |
|---|---|---|
| REQ-003 | The dashboard shows request volume, error rate, active users, and daily/weekly trends across selectable periods | `just e2e-req REQ-003` |
| REQ-005 | WAU with deltas, requests/user/day, the top-user leaderboard, and the inactive-seat report at every window (7/14/30/90d) | `just e2e-req REQ-005` |
| REQ-007 | The Code tab reports AI-authored LOC, applied edits, permission-grant rate, and commit lines, labelled as proxies | `just e2e-req REQ-007` |
| REQ-008 | Commits observed through Claude Code sessions are deduplicated, rolled up daily, and plotted beside AI usage | `just e2e-req REQ-008` |

![The analytics Overview tab with volume, errors, and active users](/files/images/evidence/req-003-overview.png)
*The Overview tab: request volume, error rate, and active users for the selected period.*

![Daily and weekly usage trend charts](/files/images/evidence/req-003-usage-trends.png)
*Daily/weekly usage trends with the period selector.*

![The Seats tab with WAU and requests per user per day](/files/images/evidence/req-005-seats.png)
*The Seats tab: WAU with deltas and per-user intensity.*

![The inactive-seat report with its configurable window](/files/images/evidence/req-005-inactive-seats.png)
*Inactive seats at a configurable 7/14/30/90-day window.*

![The Code tab showing productivity proxy metrics](/files/images/evidence/req-007-008-code-tab.png)
*The Code tab: AI-authored LOC, applied edits, permission-grant rate, and commit lines.*
