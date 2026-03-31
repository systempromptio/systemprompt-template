---
title: "Licensing Terms"
description: "12-month production use licence for the enterprise AI governance platform. Two-tier IP model: licensed platform, client-owned extension code."
author: "systemprompt.io"
slug: "proposal-licensing"
keywords: "licensing, production use, IP, intellectual property, source code, restrictions, 12-month"
kind: "guide"
public: true
tags: ["enterprise", "sales", "proposal", "licensing"]
published_at: "2026-03-04"
updated_at: "2026-03-25"
after_reading_this:
  - "Understand the 12-month production use licence and exactly what it grants"
  - "Know the two-tier IP model: licensed platform vs. client-owned extension code"
  - "See how IP ownership works for the core platform vs. the Control Center"
  - "Understand what happens at the end of the licence term"
related_docs:
  - title: "Engagement Summary"
    url: "/documentation/engagement-summary"
  - title: "Platform License"
    url: "/documentation/engagement-license"
  - title: "Questions & Answers"
    url: "/documentation/objections"
---

# Licensing Terms

**What Enterprise Demo Licences and What Enterprise Demo Owns**

---

## 12-Month Production Use Licence

A **licence fee** grants Enterprise Demo a 12-month production use licence for the core systemprompt platform. This licence covers the AI governance engine — the binary, the CLI, the MCP integration layer, the analytics system, and all core platform code.

The licence is signed as part of **Phase 2** of the engagement — after the [PRD and development phase](/documentation/engagement-implementation) has been signed off by both sides. The licence term is 12 months from production deployment. It is renewable at the end of the term.

---

## Two-Tier IP Model

This engagement involves two distinct codebases with different ownership structures:

| Codebase | Ownership | Description |
|----------|-----------|-------------|
| **Core platform** | **Licensed** — systemprompt IP | The AI governance engine, CLI, and core infrastructure. Licensed for production use during the 12-month term. |
| **Control Center** | **Owned by Enterprise Demo** | The branded dashboard, all skills, agents, configurations, and extensions. Enterprise Demo's intellectual property outright. |

This distinction is fundamental to the engagement. The platform is licensed; the Control Center is owned.

---

## What the Licence Grants

The production use licence grants Enterprise Demo full operational use of the core platform:

- **Full source code** — read, audit, modify, and extend every line for internal use
- **Unlimited deployment** — no per-server, per-instance, or per-seat limits
- **Modification rights** — modify the platform codebase for internal use without restriction
- **Updates** — all platform updates released during the licence term
- **Self-hosted** — runs on Enterprise Demo infrastructure with no external dependencies

### Licence Enforcement

The licence is enforced by the CLI. The systemprompt binary validates the licence against the systemprompt API at startup.

The core platform is licensed under the **Business Source License 1.1 (BSL-1.1)** — the source code is fully available for auditing. Enterprise Demo can read, review, and audit every line of the licence enforcement logic. There is no obfuscation and no hidden behaviour.

---

## What Enterprise Demo Owns Outright

Everything built on top of the platform is **Enterprise Demo's intellectual property**:

- The Control Center dashboard and all branding
- All custom skills, agents, and plugin configurations
- All MCP server configurations and hook definitions
- All user data, conversation logs, and analytics data
- All YAML configurations and documentation
- All extensions and customisations

There is no requirement to share, open-source, or licence back any code, configuration, or content that Enterprise Demo creates or that is created on Enterprise Demo's behalf during the implementation.

---

## Licence Restrictions

The production use licence is subject to the following restrictions:

- The licence covers **Enterprise Demo's internal use only**
- Enterprise Demo cannot resell the core platform software to third parties
- Enterprise Demo cannot launch a hosted service using the core platform that competes with the licensor
- The core platform intellectual property remains with the licensor

These restrictions apply only to the core platform. They do not apply to the Control Center, skills, agents, or any other extension code — that is Enterprise Demo's property without restriction.

---

## End of Term

At the end of the 12-month licence term:

| Item | What Happens |
|------|-------------|
| **Platform licence** | Renewable at the then-current rate. Reference partner pricing may be extended. |
| **Control Center** | Remains Enterprise Demo's property. No dependency on licence renewal. |
| **Skills and agents** | Remain with Enterprise Demo. All configurations and data are retained. |
| **Source code** | Enterprise Demo retains the last version received during the licence term. |
| **Platform updates** | Cease unless the licence is renewed or a maintenance retainer is active. |

If the licence is not renewed, the platform binary will no longer start. The Control Center code, all skills, all agents, all configurations, and all data remain with Enterprise Demo.

---

## Source Code Access

The implementation is delivered as a **separate codebase in a dedicated repository**:

- **Visible and auditable at all stages** — Enterprise Demo can review and audit the codebase at any point
- **Continuous delivery** — code is committed as it is developed
- **Full history** — the Git history provides a complete record of every change
- **Unobfuscated** — full, readable source code

---

## Relationship to the General BSL-1.1 Licence

The general platform codebase is licenced under the Business Source License 1.1 (BSL-1.1). Under the BSL-1.1, production use requires a negotiated licence agreement.

**Enterprise Demo's 12-month production use licence is that negotiated agreement.** It supersedes the general BSL-1.1 terms for Enterprise Demo's deployment and grants the explicit right to use the software in production during the licence term — subject only to the restrictions described above.

---

**Back to:** [Engagement Summary](/documentation/engagement-summary)

**Back to:** [Capability Brief](/documentation/proposal)

**Related:** [Questions & Answers](/documentation/objections)
