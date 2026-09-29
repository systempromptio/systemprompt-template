---
title: "Cost Management, Budgets & FinOps"
description: "Attribute gateway usage to users, projects, and models; inspect spend warnings and export cost reports."
author: "systemprompt.io"
slug: "enterprise-cost-management"
keywords: "cost, budget, finops, spend, caps, alerts, forecasting, csv, digests, chargeback"
kind: "guide"
public: true
tags: ["enterprise", "finops", "admin"]
published_at: "2026-08-25"
updated_at: "2026-09-28"
after_reading_this:
  - "Read the Cost tab: totals, trends, and the provider and model split"
  - "Read the spend warning thresholds and why nothing is refused on cost"
  - "Attribute any request's cost to its user, project, model, and provider"
  - "Export cost reports as CSV from the web UI or CLI"
related_docs:
  - title: "Usage, Adoption & Productivity Analytics"
    url: "/documentation/enterprise-analytics"
  - title: "Model Gateway, Routing & Data Residency"
    url: "/documentation/enterprise-model-routing"
  - title: "Enterprise Roadmap & Known Limitations"
    url: "/documentation/enterprise-roadmap"
---

# Cost Management, Budgets & FinOps

**TL;DR:** Every AI request lands with its cost attributed to a user, model, and provider, and can be sliced by group or project. The gateway's quota windows watch spend per user per hour and for the whole instance per day, and in the shipped configuration only warn: a window past its threshold lands in the governance report and the request still runs. The analytics Cost tab shows totals, trends and the provider and model split, scoped by group or project; reports export as CSV from the web and CLI.

## Per-request attribution

Cost tracking is not a rollup bolted on afterwards: **each request row carries user, model, provider, tokens, and cost** (stored as microdollar integers end to end), and users can belong to groups and projects through operator-managed membership (or directory mapping, when SSO is configured). Dashboards and reports slice on all of these dimensions in near real time, so chargeback to a project is a filter, not a reconciliation project.

```bash
systemprompt analytics costs summary
systemprompt analytics costs breakdown --by user --since 30d      # spend, requests, tokens, conversations per user
systemprompt infra logs request list --limit 20 --user <user-id>
```

## The Cost tab

`/admin/analytics?tab=cost` shows:

- **Total spend and cost trend** for the selected period, with cost per request and token counters.
- **Cost by provider and by model** — where the money actually goes.
- **Group and project selectors** — every figure on the tab narrows to the scope you pick.
- **Internal and customer views** — the default internal view carries supplier cost; the customer view (`&audience=customer`) carries no supplier figure at all, so it can be shared with a team without exposing budget.

## Spend warning thresholds

Spend thresholds are the gateway's **quota windows** (`services/gateway/policies.yaml`): a per-user hourly window (600 requests or $20) sized to flag a runaway agent, and an instance-wide daily window ($200) sized to flag an unusually expensive day. The quota plane ships in **warn mode** (`quota_mode: warn`): a window past its threshold is recorded as a `warn` decision under policy `quota`, shows up on the governance dashboard at `/admin/governance` and in `systemprompt infra logs governance report`, and the request is never refused. There is no spend clamp in the shipped configuration. Costs are attributed one request late by design (a request's cost is known after its response), so the warning lands on the request after the crossing.

Switching the plane to enforcement (`quota_mode: enforce`, HTTP 429 once a window is spent) is a review-visible change to that file.

### Subjects, periods and the sync plane

A window names a **subject** — whose usage it counts — and a **period**. Core resolves `user` and the installation-wide `organization`, the two subjects the shipped windows use. Any other subject is looked up through a registered `SubjectAttributeProvider`; this template's admin extension registers three in `extensions/web/admin/src/authz/` — `project`, `group` and `connector` — so a window can also be declared per project or per group. A person in several is counted against the first value the provider returns. `quota_fault_mode: closed` in `services/ai/gateway.yaml` means a window whose subject cannot be resolved (for example a `project` window for someone in no project) refuses the request rather than passing it uncounted, so declare such a window only once every user resolves.

Periods are any number of seconds, or a **calendar month**. Core's windows are fixed-length and aligned to the Unix epoch, so a calendar month is an interim the admin extension provides: the window is declared as `window_seconds: 2678400` (31 days), and the daily `quota_month_window` job rewrites the live row to run out at the month's end and carries yesterday's bucket into today's, so the bucket the gateway reserves against holds month-to-date usage and the first of the month starts from zero. The one gap is the first minute after the rewrite, while core's policy cache is stale.

The file is the declaration; the table the gateway enforces is `ai_gateway_policies`, which core re-reads on every request (a sixty-second cache). `/admin/sync` shows the two side by side as the `gateway_policies` plane — insert what is declared and missing, overwrite the table from code, or export the table back to `services/gateway/policies.yaml`. The template ships no console editor for the windows and no per-subject usage-against-ceiling page; edit the file and apply it through the sync plane.

## Self-service reporting

- **CLI** — cost reports accept `--since` / `--until` and export CSV. Start from `systemprompt analytics costs summary`; `analytics costs breakdown --by user|model|provider|agent` slices the same window, and `systemprompt analytics --help` lists the rest.
- **Web** — the admin UI exposes CSV exports for the Cost tab (`/admin/analytics/cost.csv`) and the request log (`/admin/requests.csv`), narrowed to the scope you select.

## Provider and model cost comparison

The Cost tab (also reachable from the CLI) breaks cost down **by provider and by model**, so you can see what a switch of route would save. Quality-normalized comparison ("is the cheaper model good enough?") requires a quality baseline for your workloads — see the [roadmap](/documentation/enterprise-roadmap).

## Verification

Use the commands and UI checks above against the configured instance. Repository maintainers can run `just test-unit` and `just test-integration`; a passing fixture test does not establish that a deployed integration is configured.
