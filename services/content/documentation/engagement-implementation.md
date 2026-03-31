---
title: "Phase 1: PRD & Development"
description: "Phase 1 of the engagement — collaborative PRD, Control Center build-out on systemprompt cloud, hands-on evaluation."
author: "systemprompt.io"
slug: "engagement-implementation"
keywords: "PRD, development, Control Center, dashboard, branding, skill migration, evaluation, phase 1"
kind: "guide"
public: true
tags: ["enterprise", "sales", "proposal", "implementation"]
published_at: "2026-03-25"
updated_at: "2026-03-25"
after_reading_this:
  - "Understand the PRD process and what Phase 1 delivers"
  - "Know that Phase 1 runs on systemprompt cloud, not Enterprise Demo infrastructure"
  - "Know that the extension codebase is owned by Enterprise Demo from delivery"
related_docs:
  - title: "Engagement Summary"
    url: "/documentation/engagement-summary"
  - title: "Phase 2: Platform License"
    url: "/documentation/engagement-license"
  - title: "Phase 2: Maintenance & Support"
    url: "/documentation/engagement-maintenance"
---

# Phase 1: PRD & Development

**Collaborative Build on systemprompt Cloud**

---

## Pricing

Phase 1 pricing reflects preferential reference partner terms and will be discussed as part of the engagement process. Payment is required in advance before work begins.

---

## How Phase 1 Works

Phase 1 is a collaborative development phase. Both sides work together to define the requirements, build the Control Center, and evaluate the result. No production commitment is required — Phase 2 only begins after both sides sign off on Phase 1 delivery.

### Process

| Step | What Happens |
|------|-------------|
| **1. PRD Agreement** | A detailed Product Requirements Document is agreed between both sides. This defines the scope, deliverables, acceptance criteria, and timeline. |
| **2. Development** | The Control Center is built on systemprompt cloud infrastructure. Enterprise Demo has hands-on access throughout development. |
| **3. Evaluation** | Both sides work with the running system. Iterative feedback, refinements, and adjustments based on real usage. |
| **4. Sign-Off** | Formal sign-off by both parties confirms the PRD has been delivered. This is the gate to Phase 2. |

### Where It Runs

Phase 1 runs entirely on **systemprompt cloud infrastructure**. Enterprise Demo does not need to provision any servers, databases, or infrastructure during this phase. The platform is fully operational on our cloud — Enterprise Demo evaluates a working system, not a mockup.

---

## What We Build

The Control Center is a branded dashboard built on the platform's extension architecture. It is a standalone codebase — separate from the core platform — configured for Enterprise Demo's operations.

### Deliverables

| Deliverable | Description |
|-------------|-------------|
| **Branded dashboard** | White-label Control Center with Enterprise Demo branding, colours, and assets |
| **Skill migration** | All existing skills mapped, migrated, and configured for Enterprise Demo's operational domains |
| **Agent configuration** | Agents configured with appropriate RBAC, skills, and MCP server access |
| **Branding assets** | Logo integration, colour scheme, typography, and favicon configuration |
| **Deployment package** | Ready to deploy into Enterprise Demo's enterprise infrastructure (used in Phase 2) |
| **Documentation** | Architecture overview, configuration guide, and operational runbook |

### What Control Center Includes

The Control Center provides Enterprise Demo with a governed administration interface for managing AI agents across the organisation:

- **Agent management** — configure, monitor, and govern all agents from a single dashboard
- **Skill management** — create, edit, and assign skills to agents by department
- **User and role management** — RBAC configuration, department assignment, access control
- **Analytics dashboards** — usage, cost, and adoption metrics by department and agent
- **MCP server management** — register, configure, and monitor MCP server connections
- **Audit and compliance** — searchable audit trails, conversation logs, and compliance reporting

---

## Ownership

> **Enterprise Demo owns the Control Center codebase from the moment of delivery.**

The extension codebase — the dashboard, all configurations, all skills, all agents, all customisations — is Enterprise Demo's intellectual property from the moment of delivery. This is true regardless of whether Phase 2 proceeds.

This means:

- **Full source code** — Enterprise Demo receives the complete, unobfuscated source code
- **Modification rights** — Enterprise Demo can modify, extend, or fork the codebase without restriction
- **No dependency on Phase 2** — the Control Center code belongs to Enterprise Demo even if the production license is never signed
- **Internal development** — Enterprise Demo's engineering team can develop the codebase independently
- **Additional consulting** — Enterprise Demo can engage us for additional development on the Control Center at any time

The Control Center is built on the platform's extension architecture, but it is a separate codebase delivered in a dedicated repository with full Git history.

---

## PRD Scope

The PRD defines the complete feature set for the Enterprise Demo Control Center. The platform's full capability set is available for scoping based on Enterprise Demo's priorities. Areas that will be collaboratively defined include:

- **Skills and skill governance** — which skills are available, how they are created, approved, and distributed across departments
- **Plugin and marketplace management** — how plugins are curated, approved, and published within the organisation
- **Governance tools** — whitelisting and blacklisting rules for tools, skills, MCP servers, and content patterns
- **Achievement and gamification analysis** — engagement tracking, achievement definitions aligned to organisational goals, and leaderboard configuration
- **User analytics with internal data mapping** — how platform analytics integrate with Enterprise Demo's existing data sources and internal reporting structures
- **Full dashboard capabilities** — which metrics, charts, drill-downs, and export options are included in the Enterprise Demo Control Center
- **MCP server integration and controls** — which MCP servers are deployed, how they are governed, and how OAuth scopes are configured per role and department

The PRD is a collaborative document. Both sides contribute to scoping decisions, ensuring the delivered Control Center matches Enterprise Demo's operational requirements.

---

## What Happens After Phase 1

| Outcome | What Happens Next |
|---------|------------------|
| **Both sides sign off** | Phase 2 begins — platform license signed, deployed to Enterprise Demo infrastructure, goes live |
| **Enterprise Demo needs changes** | PRD is revised, additional work is scoped and quoted |
| **Enterprise Demo does not proceed** | Enterprise Demo keeps the Control Center codebase. No further obligation. |

---

**Next:** [Phase 2: Platform License](/documentation/engagement-license)

**Back to:** [Engagement Summary](/documentation/engagement-summary)
