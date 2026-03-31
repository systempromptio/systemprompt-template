---
title: "Enterprise AI Governance Platform — Capability Brief"
description: "What the platform does, the two-phase engagement model, and how to get started. A concise overview for enterprise evaluation."
author: "systemprompt.io"
slug: "proposal"
keywords: "enterprise, AI governance, platform, licensing, MCP, agents, capability"
kind: "guide"
public: true
tags: ["enterprise", "sales", "proposal"]
published_at: "2026-03-04"
updated_at: "2026-03-25"
after_reading_this:
  - "Understand what the platform delivers and why it matters for enterprise AI governance"
  - "Know the two-phase engagement model: PRD then production"
  - "Know how to engage and next steps"
related_docs:
  - title: "Engagement Summary"
    url: "/documentation/engagement-summary"
  - title: "Questions & Answers"
    url: "/documentation/objections"
  - title: "Licensing"
    url: "/documentation/proposal-licensing"
---

# Enterprise AI Governance Platform

**Prepared for Enterprise Demo** | March 2026

---

## How the Engagement Works

This engagement is structured in two phases. Phase 1 is a collaborative development phase — no production commitment required. Phase 2 only begins once both sides are satisfied with the result.

### Phase 1: PRD & Delivery

Agree a detailed PRD between both sides. The Control Center is then built and delivered on systemprompt cloud infrastructure according to the PRD. Fixed price. Both sides evaluate hands-on throughout development. Phase 1 ends with formal sign-off.

### Phase 2: Production License & Support

After Phase 1 sign-off, the platform is deployed on Enterprise Demo infrastructure and goes live. 12-month production use license with monthly maintenance and support.

- **Platform license** — 12-month production use, paid in advance
- **Maintenance & support** — monthly retainer for updates, security patches, and priority support

> **Phase 2 is conditional on Phase 1 sign-off.** Enterprise Demo is not committed to the production license until both sides agree the PRD has been delivered successfully.

Pricing is available on request and will be discussed as part of the engagement process. Reference implementation partners receive preferential terms.

[Read the full engagement summary](/documentation/engagement-summary)

---

## What the Platform Does

This is an enterprise AI governance platform — a single Rust binary backed by PostgreSQL that provides centralized control over AI agents, tools, and usage across an organization.

### Core Capabilities

| Capability | What It Delivers |
|------------|-----------------|
| **Agent Governance** | Role-based access control (RBAC), department-scoped permissions, and bulk assignment across all agents, plugins, and MCP servers |
| **Audit & Compliance** | Every tool call, session, and prompt logged and searchable. Full conversation-level audit trails for compliance reporting |
| **MCP Integration** | Native MCP protocol support with per-server OAuth, hooks on every tool invocation, and central server registry |
| **Skills & Plugins** | Governed bundles of agents, skills, and MCP servers — assigned by role and department. Custom skills created in seconds |
| **Analytics** | Per-department, per-model, per-agent cost tracking. Usage dashboards with CSV export for chargeback |
| **Gamification & Engagement** | Achievement system, leaderboards, usage-driven engagement analytics. Department-scoped engagement tracking |
| **Horizontal Scaling** | Stateless JWT-based architecture. N replicas behind a load balancer. Connection pooling via SQLx |

### Architecture

- **Single binary deployment** — one Rust binary, one PostgreSQL database. No Redis, no Kafka, no microservice mesh
- **Self-hosted** — runs in your infrastructure, your compliance boundary, your network
- **CLI-first** — every operation available via `systemprompt <domain> <subcommand>` across 8 domains
- **Stateless** — JWT authentication, no session affinity. Horizontal scaling is trivial

---

## Why This Platform

- **Built for agent governance** — not a general-purpose tool with governance bolted on
- **MCP-native** — aligned with industry-standard MCP and A2A protocols
- **Production-grade** — single binary, no microservice sprawl, battle-tested scaling path
- **Full ownership** — client-owned extension code, full source, no SaaS dependency

For detailed answers to questions about data sovereignty, model lock-in, cost justification, and enterprise scale, see [Questions & Answers](/documentation/objections).

---

## Next Steps

1. **Review the engagement** — see the [Engagement Summary](/documentation/engagement-summary) for the full two-phase breakdown
2. **Schedule a meeting** — agree on a joint PRD document covering the initial Phase 1 engagement: a customised and personalised Control Center for the Enterprise Demo organisation. The PRD covers the complete feature set — skills, marketplace, governance tools (whitelisting, blacklisting), analytics, user engagement, and internal data integration — scoped collaboratively to Enterprise Demo's priorities
3. **Begin Phase 1** — sign the PRD, begin development on systemprompt cloud infrastructure
