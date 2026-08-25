---
title: "Cost Management, Budgets & FinOps"
description: "Attribute every AI request to a user, org, and model; set soft alerts and hard monthly caps; forecast overruns; export CSV reports and schedule Slack digests."
author: "Astound Digital"
slug: "enterprise-cost-management"
keywords: "cost, budget, finops, spend, caps, alerts, forecasting, csv, digests, chargeback"
kind: "guide"
public: true
tags: ["enterprise", "finops", "admin"]
published_at: "2026-08-25"
updated_at: "2026-08-25"
after_reading_this:
  - "Read the Spend tab: meters, burn-up chart, projections, and crossing history"
  - "Configure soft thresholds and hard monthly caps on an organization plan"
  - "Attribute any request's cost to its user, org, department, model, and provider"
  - "Export cost reports as CSV from the web UI or CLI"
  - "Schedule weekly or monthly cost digests to Slack"
related_docs:
  - title: "Usage, Adoption & Productivity Analytics"
    url: "/documentation/enterprise-analytics"
  - title: "Model Gateway, Routing & Data Residency"
    url: "/documentation/enterprise-model-routing"
  - title: "Enterprise Roadmap & Known Limitations"
    url: "/documentation/enterprise-roadmap"
---

# Cost Management, Budgets & FinOps

**TL;DR:** Every AI request lands with its cost attributed to a user, organization, department, model, and provider. Organization plans carry a soft threshold that fires a Slack alert and a hard monthly cap the gateway enforces with HTTP 429. The Spend tab shows meters, a burn-up chart with a month-end projection, and projected-overrun alerts; reports export as CSV from the web and CLI, and scheduled digests land in Slack.

## Per-request attribution

Cost tracking is not a rollup bolted on afterwards: **each request row carries user, organization, department, model, provider, tokens, and cost** (stored as microdollar integers end to end). Dashboards and reports slice on all of these dimensions in near real time, so chargeback to a cost center is a filter, not a reconciliation project.

```bash
systemprompt analytics costs summary
systemprompt infra logs request list --limit 20
```

## The Spend tab

`/admin/analytics` includes a Spend tab showing:

- **Total spend and daily cost trend** for the selected period, with cost per request and token counters.
- **Model distribution and cost-by-model series** — where the money actually goes.
- **Spend meters** — each organization's month-to-date spend against its soft threshold and hard cap.
- **Burn-up chart with month-end projection** — a linear projection of where the month will land, drawn against the cap.
- **12-month crossing history** — every soft-threshold and cap crossing over the past year, per organization.

## Budgets: soft threshold and hard cap

Each organization plan defines two budget lines:

- **Soft threshold** — non-blocking. On the first crossing in a month, a Slack alert fires (once per transition, not per request) and the crossing is recorded.
- **Hard monthly cap** — enforced at the gateway. Once an organization's month-to-date spend reaches its cap, further requests are refused with **HTTP 429 and a `retry-after`** pointing at the month boundary. Traffic resumes automatically when the month rolls over.

Budget scope is the organization. Per-team and per-project budgets await the open hierarchy decisions, and per-key budgets are a known limitation — see the [roadmap](/documentation/enterprise-roadmap).

## Early warning: projected overruns

Waiting for the cap to hit is too late, so the projection runs on the request path: the burn-up projection is compared against the cap continuously, and the **first projected-overrun crossing per organization per month** is recorded and alerted to Slack. Finance hears about a trajectory problem mid-month, while there is still time to act.

## Self-service reporting

- **CLI** — cost reports accept `--since` / `--until` and export CSV. Start from `systemprompt analytics costs summary` and `systemprompt analytics --help`.
- **Web** — the admin UI exposes CSV export endpoints for cost and request reports, scoped to the organizations you can see.

## Scheduled digests

A scheduled digest job delivers **weekly or monthly summaries to Slack** over the existing alerting transport: spend by organization, budget utilization against thresholds and caps, and notable movements. Stakeholders get the numbers without logging in.

## Provider and model cost comparison

An internal report (also reachable from the CLI) breaks cost and margin down **by provider and by model**, so you can see what a switch of route would save. Quality-normalized comparison ("is the cheaper model good enough?") requires an agreed quality baseline, which is an open decision — noted on the [roadmap](/documentation/enterprise-roadmap).

## Verified evidence

Every capability on this page is proven by tagged end-to-end tests run against a seeded instance. To replicate: `just start`, then `just e2e-seed --reset`, then the command in the table. Screenshots regenerate with `just e2e-screens`.

| REQ | What the test proves | Replicate with |
|---|---|---|
| REQ-004 | The Spend tab shows total spend, daily trend, cost/request, token counters, and cost-by-model distribution | `just e2e-req REQ-004` |
| REQ-009 | The hard monthly cap returns 429 at the gateway; the soft threshold records and Slack-alerts on first crossing | `just e2e-req REQ-009` |
| REQ-010 | Every request row carries user, org, department, model, provider, and cost, and reports slice on all of them | `just e2e-req REQ-010` |
| REQ-011 | The month-end projection fires a projected-overrun alert on its first crossing per org per month | `just e2e-req REQ-011` |
| REQ-012 | The soft threshold alerts once per transition and never blocks a request | `just e2e-req REQ-012` |
| REQ-013 | Requests over the org's monthly cap are refused with 429 + retry-after and resume at the month boundary | `just e2e-req REQ-013` |
| REQ-014 | The burn-up chart pairs actual spend with the linear month-end projection against the cap | `just e2e-req REQ-014` |
| REQ-015 | Cost reports export as CSV from both the CLI (`--since`/`--until`) and the admin web UI, org-scoped | `just e2e-req REQ-015` |
| REQ-016 | The scheduled digest job delivers weekly/monthly cost and budget-utilization summaries to Slack | `just e2e-req REQ-016` |
| REQ-017 | The comparison report breaks cost down by provider and by model | `just e2e-req REQ-017` |

![The Spend tab with total spend, trend, and cost per request](/files/images/evidence/req-004-spend.png)
*REQ-004 — the Spend tab: total spend, daily cost trend, and cost per request.*

![Cost distribution across models](/files/images/evidence/req-004-model-mix.png)
*REQ-004 — the model mix: where spend actually lands, by model.*

![Per-organization spend meters against soft threshold and hard cap](/files/images/evidence/req-009-spend-meters.png)
*REQ-009 — spend meters: month-to-date spend against each organization's threshold and cap.*

REQ-010 through REQ-017 additionally have gateway-level proofs in the `req_010_*`–`req_017_*` Rust modules under `tests/integration/` (run with `just test-integration`); the screenshot pack grows with `just e2e-screens`.
