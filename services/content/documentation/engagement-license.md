---
title: "Phase 2: Platform License"
description: "12-month production use license, beginning after Phase 1 sign-off. List price €5,000/month, reference partner rate €3,000/month."
author: "systemprompt.io"
slug: "engagement-license"
keywords: "license, platform, production use, annual, pricing, partner discount"
kind: "guide"
public: true
tags: ["enterprise", "sales", "proposal", "license"]
published_at: "2026-03-25"
updated_at: "2026-03-25"
after_reading_this:
  - "Understand the platform license pricing and the reference partner discount"
  - "Know exactly what the license covers and what it does not"
  - "Understand the difference between the licensed platform and the client-owned extension code"
related_docs:
  - title: "Engagement Summary"
    url: "/documentation/engagement-summary"
  - title: "Licensing Terms"
    url: "/documentation/proposal-licensing"
  - title: "Implementation Services"
    url: "/documentation/engagement-implementation"
---

# Phase 2: Platform License

**12-Month Production Use License — Begins After Phase 1 Sign-Off**

---

## Pricing

| | List Price | Partner Price |
|---|----------:|-------------:|
| **Monthly rate** | €5,000/month | **€3,000/month** |
| **Annual total** | €60,000 | **€36,000** |
| **Saving** | | **€24,000 (40%)** |

> **Reference implementation partner rate.** The 40% discount reflects Foodles's role as a flagship deployment partner.

**Payment:** €36,000 paid 12 months upfront after Phase 1 sign-off. The production license is granted upon receipt of payment.

---

## What the License Covers

The platform license grants Foodles 12 months of production use of the core systemprompt platform — the AI governance engine. This includes:

| Feature | Included |
|---------|----------|
| **Agent governance** | RBAC, department-scoped permissions, bulk assignment |
| **MCP integration** | Native protocol support, per-server OAuth, hooks on every tool invocation |
| **Skills and plugins** | Governed bundles of agents, skills, and MCP servers |
| **Audit and compliance** | Every tool call, session, and prompt logged and searchable |
| **Analytics** | Per-department, per-model, per-agent cost tracking with CSV export |
| **Horizontal scaling** | Stateless JWT architecture, N replicas behind a load balancer |
| **CLI access** | Full `systemprompt` CLI across all 8 domains |
| **Updates** | All platform updates released during the license term |

### No per-seat or per-instance limits

The license covers unlimited users, unlimited agents, unlimited skills, unlimited MCP servers, and unlimited deployment instances. There are no usage-based surcharges from the platform — AI provider API costs are separate and billed directly by the provider.

---

## What the License Does Not Cover

| Item | Notes |
|------|-------|
| **AI provider API costs** | Billed directly by Anthropic, OpenAI, or other providers |
| **Infrastructure costs** | Hosting on Foodles infrastructure is Foodles's responsibility |
| **Extension codebase** | The Control Center is owned by Foodles — it is not part of the platform license |

---

## License Enforcement

The platform license is enforced by the CLI. The systemprompt binary validates the license against the systemprompt API at startup and operates within the licensed term.

The core platform is licensed under the **Business Source License 1.1 (BSL-1.1)** — the source code is fully available for auditing. Foodles can inspect every line of the licence enforcement logic. Source-available means full transparency with no obfuscation.

---

## Platform IP vs. Extension Code

The platform license covers the **core systemprompt intellectual property** — the governance engine, the CLI, the MCP integration layer, the analytics system, and all core platform code.

It does **not** cover the Control Center extension codebase. That code is [owned by Foodles](/documentation/engagement-implementation) and is not subject to the platform license. Foodles retains the Control Center regardless of whether the platform license is renewed.

---

## Renewal

At the end of the 12-month term:

- The platform license can be renewed at the then-current rate
- Reference partner pricing may be extended based on the relationship
- The Control Center extension code remains Foodles's property regardless of renewal
- All skills, agents, configurations, and data created during the term remain with Foodles

---

**Next:** [Maintenance & Support](/documentation/engagement-maintenance)

**Back to:** [Engagement Summary](/documentation/engagement-summary)
