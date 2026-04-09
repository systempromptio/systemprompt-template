---
title: "Deployment"
description: "How the AI governance control plane deploys into production — as a pluggable governance layer alongside existing AI infrastructure, or as a centralised platform for distributed AI systems."
author: "systemprompt.io"
slug: "deployment"
keywords: "deployment, production, binary, white label, control plane, infrastructure, enterprise, scalable"
kind: "guide"
public: true
tags: ["introduction", "deployment", "enterprise"]
published_at: "2026-04-01"
updated_at: "2026-04-01"
after_reading_this:
  - "Understand the two primary production deployment patterns: pluggable governance and centralised platform"
  - "Know the binary footprint and infrastructure requirements"
  - "Choose the right deployment pattern for your organisation"
related_docs:
  - title: "Introduction"
    url: "/documentation/introduction"
  - title: "Platform Overview"
    url: "/documentation/platform-overview"
  - title: "Architecture Overview"
    url: "/documentation/architecture"
  - title: "Deployment Models"
    url: "/documentation/deployment-models"
  - title: "Scaling Architecture"
    url: "/documentation/scaling"
  - title: "Configuration & Profiles"
    url: "/documentation/configuration"
---

# Deployment

The control plane is a single compiled Rust binary — approximately 50 MB — with PostgreSQL as its only dependency. No runtime, no interpreter, no container orchestration required. It runs on any Linux server and can be deployed in minutes.

There are two primary ways to deploy into production.

---

## 1. Pluggable Governance Layer

**You already have AI infrastructure. You need governance.**

In this model, the binary slots into your existing AI stack as a governance control plane. Your agents, your LLM providers, your MCP servers, your internal tools — all stay where they are. The control plane sits between them, governing the boundary.

```
┌─────────────────────────────────────────────────────────────┐
│           Your Existing AI Infrastructure                    │
│  Agents, LLM providers, MCP servers, internal tools          │
├─────────────────────────────────────────────────────────────┤
│         ▼  Control Plane (~50 MB binary)  ▼                  │
│  Auth · Access control · Rate limiting · Audit trails        │
│  Cost tracking · Tool governance · Secret detection          │
├─────────────────────────────────────────────────────────────┤
│              Your Existing Backend Services                   │
│  Databases, APIs, data pipelines, vector stores              │
└─────────────────────────────────────────────────────────────┘
```

### How it works

- Deploy the binary alongside your existing services — as a sidecar process, a Docker container, or a standalone service on the same network
- Point your AI clients at the control plane's MCP endpoint
- Configure OAuth to authenticate against your existing identity provider
- Governance policies are applied transparently — your agents and tools continue to work as before, with access control, audit logging, and cost tracking added

### When to use

- Your organisation already has AI agents and tools in production
- You need to add governance without replacing or disrupting what you have
- You want a white-label solution branded to your organisation
- Compliance or security requires audit trails and access control on AI tool usage
- You want to start governing immediately with minimal integration effort

### What you get

- Full governance on every MCP tool call passing through the control plane
- Unified audit trail across all AI clients and tools
- Cost tracking and attribution by user, department, and agent
- Role-based access control integrated with your existing identity provider
- Zero changes to your existing agents or backend services

---

## 2. Centralised AI Platform

**You are building enterprise AI infrastructure. You need the platform.**

In this model, the binary is the centre of your AI operations — the platform through which all AI usage is provisioned, governed, and observed. It manages the full lifecycle: user onboarding, agent configuration, skill distribution, marketplace management, analytics, and compliance.

```
┌─────────────────────────────────────────────────────────────┐
│                    Distributed AI Systems                     │
│  Department A agents · Department B agents · Partner tools   │
│  Remote offices · Cloud workloads · On-premise systems       │
│                          ▼                                    │
├─────────────────────────────────────────────────────────────┤
│          Centralised Control Plane Platform                   │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │ Governance│  │Marketplace│  │ Analytics│  │  Admin   │    │
│  │ Pipeline  │  │  & Skills │  │Dashboard │  │Dashboard │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │   MCP    │  │   User   │  │   Audit  │  │   Cost   │    │
│  │ Registry │  │Management│  │  Trails  │  │ Tracking │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
├─────────────────────────────────────────────────────────────┤
│                    PostgreSQL Database                        │
└─────────────────────────────────────────────────────────────┘
```

### How it works

- Deploy the binary as a centralised service — behind a load balancer for high availability
- Onboard users and configure roles, departments, and permissions through the admin dashboard
- Create and distribute governed plugins, skills, and agents through the built-in marketplace
- AI clients across the organisation connect to the platform for tool access, skill distribution, and governance
- Analytics dashboards provide real-time visibility into usage, costs, and compliance across all departments

### When to use

- You are deploying AI across a large enterprise with multiple departments and teams
- You need centralised visibility and control over distributed AI systems
- You want a managed marketplace for distributing governed AI capabilities
- You need department-level cost attribution and chargeback
- Compliance requires a single audit trail across all AI usage in the organisation

### What you get

- Everything in the pluggable model, plus:
- Built-in admin dashboard with real-time analytics
- Marketplace for distributing plugins, skills, and agents to users
- User management with role-based and department-scoped access control
- Gamification, leaderboards, and engagement tracking
- Multi-tenant support for serving multiple organisations from a single deployment
- CLI with 8 domains covering every platform operation

---

## The Binary

The production binary is a single statically-compiled Rust executable.

| Property | Value |
|----------|-------|
| **Size** | ~50 MB |
| **Dependencies** | PostgreSQL (only external dependency) |
| **Runtime** | None — compiled native code, no interpreter or VM |
| **OS** | Linux (x86_64, ARM64) |
| **Startup time** | Sub-second |
| **Memory footprint** | ~30 MB base, scales with concurrent connections |
| **Configuration** | YAML profiles — same binary for dev, staging, production |

No Redis. No Kafka. No microservice mesh. No container orchestration required. One binary, one database.

### Deployment options

The binary runs anywhere Linux runs:

- **Bare metal** — Direct execution on a server
- **Docker** — Multi-stage build produces a minimal container image
- **Kubernetes** — Health check endpoints for liveness and readiness probes
- **Cloud VMs** — Any cloud provider (AWS, GCP, Azure, on-premise cloud)
- **Air-gapped networks** — No external network calls required after deployment

For detailed technical deployment models (sidecar, gateway, embedded, multi-tenant), see [Deployment Models](/documentation/deployment-models). For scaling strategies, see [Scaling Architecture](/documentation/scaling).

---

## Choosing Your Pattern

| Your situation | Recommended pattern |
|---------------|-------------------|
| Existing AI infrastructure needs governance | Pluggable governance layer |
| Building enterprise AI from scratch | Centralised platform |
| Small team, rapid deployment | Pluggable — deploy and govern in hours |
| Large enterprise, multiple departments | Centralised — full platform capabilities |
| Service provider hosting for clients | Centralised with multi-tenant mode |
| Hybrid — some teams have AI, others don't | Start pluggable, expand to centralised |

Both patterns use the same binary. The difference is configuration and scope — not a different product. You can start with a pluggable governance layer and expand to a full centralised platform as your AI adoption grows.

See [Configuration & Profiles](/documentation/configuration) for profile-based deployment configuration and [Installation & Setup](/documentation/installation) for getting started.
