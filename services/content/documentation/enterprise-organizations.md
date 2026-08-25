---
title: "Organizations, Departments & Hubs"
description: "Model your enterprise as organizations and departments, filter every analytics view by them, and see the two-organization worked example with isolated rosters."
author: "Astound Digital"
slug: "enterprise-organizations"
keywords: "organizations, departments, hubs, enterprise, hierarchy, filters, analytics, plans"
kind: "guide"
public: true
tags: ["enterprise", "admin", "organizations"]
published_at: "2026-08-25"
updated_at: "2026-08-25"
after_reading_this:
  - "Create and manage organizations at /admin/enterprises and departments at /admin/access/departments"
  - "Filter any analytics tab by organization, department, or individual user"
  - "Apply the two-organization pattern to keep separate projects fully isolated"
  - "Understand what a Hub is and where the Hub analytics dimension stands"
related_docs:
  - title: "User & Access Management"
    url: "/documentation/enterprise-user-access"
  - title: "Usage, Adoption & Productivity Analytics"
    url: "/documentation/enterprise-analytics"
  - title: "Enterprise Roadmap & Known Limitations"
    url: "/documentation/enterprise-roadmap"
---

# Organizations, Departments & Hubs

**TL;DR:** The platform models your company as organizations containing departments containing users, managed at `/admin/enterprises` and `/admin/access/departments`. Every analytics tab filters on all three dimensions, and organizations carry their own plans, seat limits, and budget caps — so two projects can run on one instance without seeing each other.

## Organizations

`/admin/enterprises` is where organizations are created and managed. An organization is the top-level boundary: it owns a plan (seat limit, soft budget threshold, hard monthly cap), a user roster, and its own slice of every analytics view. Access-control rules and gateway entitlements can be scoped to an organization, so what one org's users can reach says nothing about another's.

## Departments

`/admin/access/departments` manages departments inside an organization. Departments are the chargeback and reporting unit: cost attribution, usage dashboards, and access rules all understand them. A user belongs to a department, and the department rolls up to its organization.

## Filtering everywhere

The organization / department / user filters are applied **identically across every analytics tab** — Overview, Usage, Spend, Seats, and Code. Pick an organization and every chart, leaderboard, and export on the page re-scopes to it; narrow further to a department or a single user without leaving the view. See [Usage, Adoption & Productivity Analytics](/documentation/enterprise-analytics) for what each tab shows.

## Worked example: two isolated projects

A real deployment runs two organizations on one instance:

- **`ai-kit`** — one delivery group, with its own plan, seat limit, and budget cap.
- **`ai-sdlc-delivery`** — a second group, likewise self-contained.

Each organization has its own roster (populated by admin invites per group), its own analytics scope, and its own ACLs. Nothing overlaps: a user in `ai-kit` cannot appear in `ai-sdlc-delivery` reports, spend in one never counts against the other's cap, and entitlements are granted per organization. This is the recommended pattern whenever two initiatives need clean cost and access separation.

## Hubs

A **Hub is a geographical grouping** — a region or location dimension that sits alongside organizations and departments rather than inside them. This is the agreed definition.

The Hub *analytics dimension* itself — filtering dashboards and reports by Hub — is not yet available; it is scheduled as the next delivery pass. Until it ships, geographic reporting is approximated by mapping locations onto organizations or departments. Track its status on the [Enterprise Roadmap](/documentation/enterprise-roadmap).

## Verified evidence

Every capability on this page is proven by tagged end-to-end tests run against a seeded instance. To replicate: `just start`, then `just e2e-seed --reset`, then the command in the table. Screenshots regenerate with `just e2e-screens`.

| REQ | What the test proves | Replicate with |
|---|---|---|
| REQ-006 | Analytics drill down by organization, department, and individual user, with the two seeded organizations fully isolated from each other | `just e2e-req REQ-006` |

![Analytics drill-down filtered by organization and department](/files/images/evidence/req-006-drilldown.png)
*REQ-006 — the same dashboard re-scoped by the organization and department filters.*

![Analytics drill-down narrowed to a single user](/files/images/evidence/req-006-user-drilldown.png)
*REQ-006 — narrowing further to one individual user without leaving the view.*
