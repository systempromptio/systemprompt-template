---
title: "Platform Overview — What the Software Does"
description: "The three pillars of enterprise AI governance: insight into AI usage, governance to control it, and integration into the surfaces where it is used. Built on Rust, deployed on your infrastructure."
author: "systemprompt.io"
slug: "platform-overview"
keywords: "AI governance, narrow waist, Rust, on-premise, insight, governance, integration, enterprise"
kind: "guide"
public: true
tags: ["introduction", "overview", "governance", "architecture"]
published_at: "2026-03-27"
updated_at: "2026-03-27"
after_reading_this:
  - "Explain the three pillars: insight, governance, and integration"
  - "Understand the narrow-waist architecture at a high level"
  - "Know why on-premise Rust matters for enterprise AI governance"
related_docs:
  - title: "Introduction to the Platform"
    url: "/documentation/introduction"
  - title: "Architecture Overview"
    url: "/documentation/architecture"
  - title: "Tool Governance"
    url: "/documentation/tool-governance"
  - title: "Dashboard"
    url: "/documentation/dashboard"
  - title: "Authentication"
    url: "/documentation/authentication"
---

# Platform Overview

**This is enterprise AI governance software. A single Rust binary backed by PostgreSQL that sits between every AI client and every backend service in your organisation — the narrow waist through which all AI usage flows. It provides three things: insight into what people are doing with AI, governance to control what they can do, and integration into the Claude surfaces where AI is actually used. It runs on your infrastructure, on your network, under your compliance controls.**

---

## The Problem

Enterprise AI adoption is fragmented. Different teams adopt different tools, different models, different workflows. Without a governance layer:

- **No visibility** — You do not know who is using AI, what they are using it for, or what it costs.
- **No control** — You cannot enforce policies on tool usage, data access, or model selection.
- **No audit trail** — When something goes wrong, there is no record of what happened.
- **No consistency** — Each team manages its own AI stack independently, creating compliance gaps and duplicated effort.

The platform solves this by providing a single governance layer that all AI usage passes through — regardless of which Claude surface, which team, or which tools are involved.

## The Narrow Waist

The platform sits between all AI clients (Claude Code, Claude Cowork, claude.ai, custom agents, internal tools) and all backend services (LLM providers, MCP servers, databases, APIs, data pipelines).

```
┌─────────────────────────────────────────────────────────────┐
│                      Client Stacks                           │
│  Claude Code, Claude Cowork, claude.ai, Custom Agents,       │
│  MCP Clients, REST Consumers, Internal Tools                 │
├─────────────────────────────────────────────────────────────┤
│            ▼  Governance Platform (narrow waist)  ▼          │
│  Auth, access control, rate limiting, audit,                 │
│  cost tracking, compliance, tool governance                  │
├─────────────────────────────────────────────────────────────┤
│                      Backend Stacks                          │
│  LLM Providers, MCP Servers, Databases, APIs,                │
│  Internal Services, Data Pipelines, Vector Stores            │
└─────────────────────────────────────────────────────────────┘
```

The platform does not replace anything above or below it. It governs the boundary. Every AI interaction passes through this layer — authenticated, authorised, rate-limited, logged, and costed. For the full technical architecture, see [Architecture Overview](/documentation/architecture).

---

## Three Pillars

### Insight — Know What Is Happening

You cannot govern what you cannot see. The platform provides complete visibility into AI usage across your organisation.

| Capability | What It Delivers |
|------------|-----------------|
| **Activity Tracking** | 13 activity categories covering every AI interaction — tool calls, sessions, conversations, agent invocations |
| **Cost Tracking** | Per-department, per-model, per-agent token consumption and cost. CSV export for chargeback |
| **Analytics Dashboard** | Real-time metrics with charts, activity feed, and health indicators |
| **Engagement Metrics** | Usage-driven engagement analytics, achievement systems, leaderboards, department scores |

Insight is not just reporting — it drives governance decisions. When you can see that a department is overspending on a specific model, or that a tool is being used in unexpected ways, you can act.

- [Dashboard](/documentation/dashboard) — Real-time metrics and health indicators
- [Activity Tracking](/documentation/activity-tracking) — All 13 activity categories
- [Cost Tracking](/documentation/cost-tracking) — Token consumption and chargeback
- [Gamification & Leaderboard](/documentation/gamification) — Engagement tracking

### Governance — Control What Can Happen

The platform enforces business logic across all AI interactions. This is not a bolt-on compliance layer — governance is the core function.

| Capability | What It Delivers |
|------------|-----------------|
| **Access Control** | Role-based and department-scoped permissions governing all resources |
| **Tool Governance** | Per-tool access control, execution logging, and event hooks on every MCP tool call |
| **Hooks & Automation** | Event-driven triggers that fire when things happen — tool calls, logins, session starts, policy violations |
| **Rate Limiting** | Per-role rate limits to control resource consumption and prevent abuse |
| **Audit Trails** | Every tool call, session, and prompt logged and searchable. Full conversation-level audit for compliance |
| **Secrets Management** | Environment variables and secret management with encryption at rest |

Governance policies are defined centrally and enforced consistently. A developer using Claude Code, a manager using Claude Cowork, and an analyst using claude.ai all operate under the same governance rules.

- [Access Control & RBAC](/documentation/access-control) — Role and department-based permissions
- [Tool Governance](/documentation/tool-governance) — MCP tool access control and execution logging
- [Hooks & Automation](/documentation/hooks) — Event-driven triggers
- [Rate Limiting](/documentation/rate-limiting) — Per-role resource controls
- [Audit Trails & Events](/documentation/events) — System event log and compliance reporting

### Integration — Meet Users Where They Work

Governance is only effective if it integrates into the tools people actually use. The platform provides native integration with the Claude ecosystem.

| Surface | Integration |
|---------|-------------|
| **Claude Code** | Plugin system distributes governed skills, agents, and tools to developers via personalised marketplaces |
| **Claude Cowork** | Collaborative AI sessions connect to the same governance pipeline — same policies, same audit trail |
| **claude.ai** | MCP servers bridge claude.ai to governed tools with OAuth-based authentication |

The key principle: **one governance layer, every surface**. Whether a user is in a CLI, a collaborative workspace, or a web browser, the same access control, audit trail, and cost tracking applies.

- [Integration: Claude Code](/documentation/integration-claude-code)
- [Integration: Claude Cowork](/documentation/integration-claude-cowork)
- [Integration: Claude AI](/documentation/integration-claude-ai)

---

## Technical Fundamentals

The platform is built for minimal operational overhead and maximum performance.

- **Single Rust binary** — One binary, one PostgreSQL database. No Redis, no Kafka, no microservice mesh. Deployment is trivial.
- **On your infrastructure** — Runs in your VPC, your compliance boundary, your network. No data leaves your infrastructure unless you configure external providers.
- **Lightning fast** — Rust with zero-cost abstractions, no garbage collector, and async I/O via Tokio. The governance layer adds minimal latency to AI interactions.
- **CLI-first** — Every operation available via `systemprompt <domain> <subcommand>` across 8 domains. The CLI operates on the same code paths as the HTTP API.
- **Profile-based configuration** — The same binary runs in development and production with different YAML configs. No code changes between environments.
- **Stateless and scalable** — JWT authentication, no session affinity. Horizontal scaling is adding instances behind a load balancer.

For the full technical deep-dive, see [Architecture Overview](/documentation/architecture). For deployment options, see [Deployment Models](/documentation/deployment-models). For setup instructions, see [Installation](/documentation/installation).
