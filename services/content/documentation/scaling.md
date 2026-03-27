---
title: "Scaling Architecture"
description: "How the platform scales across deployment models with stateless architecture, PostgreSQL optimization, tiered rate limiting, and the narrow-waist design that adapts to any existing infrastructure."
author: "systemprompt.io"
slug: "scaling"
keywords: "scaling, enterprise, horizontal scaling, performance, architecture, narrow waist, deployment models"
kind: "guide"
public: true
tags: ["scaling", "enterprise", "infrastructure"]
published_at: "2026-03-19"
updated_at: "2026-03-20"
after_reading_this:
  - "Explain why the platform scales horizontally without application changes"
  - "Choose the right deployment model for your infrastructure"
  - "Configure tiered rate limiting for different user classes"
  - "Optimize PostgreSQL for high-concurrency AI workloads"
related_docs:
  - title: "Platform Architecture"
    url: "/documentation/architecture"
  - title: "Rate Limiting & Compliance"
    url: "/documentation/rate-limiting"
  - title: "Cost Tracking & Model Usage"
    url: "/documentation/cost-tracking"
  - title: "Access Control"
    url: "/documentation/access-control"
  - title: "Presentation"
    url: "/documentation/presentation"
---

# Scaling Architecture

> **See this in the presentation:** [Slide 11: Personalization & Ownership](/documentation/presentation#slide-11)

**TL;DR:** The platform is the narrow waist between your client stacks and your backend stacks. It scales horizontally because it is stateless — JWT tokens carry identity, PostgreSQL holds all persistent state, and application instances are interchangeable. It deploys as a sidecar, a standalone service, a centralized multi-tenant gateway, or an embedded library. AI implementations are fragmented and complex. This adapts to whatever architecture exists.

## The Narrow Waist

Enterprise AI infrastructure is fragmented. Different teams use different agent frameworks, different model providers, different tool ecosystems. The governance layer cannot impose a single stack — it must sit between all of them.

```
┌──────────────────────────────────────────────────────┐
│                   Client Stacks                       │
│  Claude Code, Custom Agents, Chat UIs, A2A Clients,  │
│  MCP Clients, REST Consumers, Internal Tools          │
├──────────────────────────────────────────────────────┤
│              ▼  systemprompt  ▼                        │
│  The narrow waist: governance, auth, rate limiting,   │
│  audit, access control, cost tracking, compliance     │
├──────────────────────────────────────────────────────┤
│                   Backend Stacks                      │
│  LLM Providers, MCP Servers, Databases, APIs,         │
│  Internal Services, Data Pipelines, Vector Stores     │
└──────────────────────────────────────────────────────┘
```

The platform does not replace anything above or below it. It governs the boundary between them — authenticating requests, enforcing access policies, tracking costs, rate limiting traffic, and logging everything for audit. Whatever client frameworks and backend services exist today, the governance layer adapts to them.

## Deployment Models

The platform is a single Rust binary with PostgreSQL as its only dependency. This gives you four deployment options depending on your infrastructure requirements.

### Sidecar Deployment

Deploy an instance alongside each agent or service. The platform runs as a companion container in the same pod or task definition, governing that agent's traffic locally.

```
┌────────────────────┐  ┌────────────────────┐
│  Agent Pod A        │  │  Agent Pod B        │
│  ┌──────┐┌───────┐ │  │  ┌──────┐┌───────┐ │
│  │Agent ││systemprompt│ │  │Agent ││systemprompt│
│  └──────┘└───────┘ │  │  └──────┘└───────┘ │
└────────┬───────────┘  └────────┬───────────┘
         │                       │
         └───────────┬───────────┘
                     │
              ┌──────▼──────┐
              │ PostgreSQL  │
              └─────────────┘
```

**Best for:** Teams that want governance at the agent level with minimal network hops. Each sidecar shares the same PostgreSQL database for centralized policy and audit.

### Standalone Service

Deploy as an independent service behind a load balancer. Multiple stateless instances handle all governance traffic centrally.

```
                ┌─────────────┐
                │   DNS / CDN  │
                └──────┬──────┘
                       │
                ┌──────▼──────┐
                │    Load     │
                │  Balancer   │
                └──────┬──────┘
                       │
        ┌──────────────┼──────────────┐
        │              │              │
 ┌──────▼──────┐┌─────▼───────┐┌─────▼───────┐
 │  Instance 1 ││  Instance 2 ││  Instance N │
 │  (Rust bin) ││  (Rust bin) ││  (Rust bin) │
 └──────┬──────┘└──────┬──────┘└──────┬──────┘
        │              │              │
        └──────────────┼──────────────┘
                       │
                ┌──────▼──────┐
                │ PostgreSQL  │
                │  Primary    │
                └──────┬──────┘
                       │
                ┌──────▼──────┐
                │  Read       │
                │  Replicas   │
                └─────────────┘
```

**Best for:** Centralized governance across many agents and teams. The standard enterprise-scale deployment model.

### Centralized Multi-Tenant

A single deployment serves multiple teams, departments, or organizational units. Tenant isolation is enforced through access control, rate limiting tiers, and scoped JWT claims — all within the same infrastructure.

**Best for:** Organizations that need unified governance with per-team isolation. Reduces operational overhead while maintaining strict boundaries between organizational units.

### Embedded Library

The core is a Rust library. It compiles directly into your existing infrastructure — your own binary, your own deployment pipeline. No separate service to manage.

**Best for:** Teams with existing Rust infrastructure who want to add governance without introducing a new service boundary. Maximum performance, minimum operational overhead.

## Why It Scales

Three architectural properties make horizontal scaling possible without application changes:

| Property | Effect |
|----------|--------|
| **Stateless application** | No session state in the application layer. Every request is self-contained. |
| **JWT authentication** | Tokens carry identity and claims. No server-side session lookup. |
| **PostgreSQL as single state store** | All persistent data in one database. No distributed state to synchronize. |

This means any request can hit any instance. Load balancers need no sticky sessions. Adding capacity means adding instances.

### Adding Instances

Each application instance is a container running the single Rust binary. To add capacity:

1. Deploy additional containers with the same production profile
2. Register them with the load balancer
3. Health checks confirm readiness before traffic is routed

No coordination between instances is required. No leader election, no cluster membership, no distributed locks.

### Load Balancer Requirements

| Requirement | Detail |
|-------------|--------|
| **Algorithm** | Round-robin or least-connections. No sticky sessions needed. |
| **Health check** | HTTP GET to the health endpoint. Remove unhealthy instances automatically. |
| **TLS termination** | Terminate TLS at the load balancer or at each instance (both supported). |
| **WebSocket support** | Required for streaming AI responses (`/api/v1/stream`). |

## PostgreSQL Scaling

PostgreSQL is the only stateful component. Scaling it requires attention to connection pooling, read replicas, and indexing.

### Connection Pooling

The platform uses SQLx with async connection pooling. Each application instance maintains a connection pool to PostgreSQL.

| Parameter | Recommendation at Scale |
|-----------|------------------------|
| **Pool size per instance** | 20-50 connections |
| **Total connections** | Pool size x number of instances |
| **Max PostgreSQL connections** | Set `max_connections` to total + 20% headroom |
| **Connection timeout** | 30 seconds (default) |

For example, with 10 application instances and 30 connections each, PostgreSQL needs to handle 300 connections plus headroom — well within PostgreSQL's capabilities with proper tuning.

### Read Replicas

For read-heavy workloads (analytics queries, dashboard rendering, skill lookups), configure PostgreSQL read replicas:

| Workload | Route To |
|----------|----------|
| **Writes** (create user, log event, update skill) | Primary |
| **Reads** (dashboard queries, analytics, skill list) | Read replicas |
| **AI request logging** | Primary (write) then replica (read for audit) |

At enterprise scale, analytics queries are the heaviest read workload. Routing them to read replicas keeps the primary responsive for writes.

### Indexing

High-cardinality columns that benefit from indexes at scale:

| Table | Column | Why |
|-------|--------|-----|
| **events** | `created_at`, `user_id`, `event_type` | Event log queries filter by time, user, and type |
| **sessions** | `user_id`, `created_at` | Session lookups by user |
| **ai_requests** | `agent_id`, `created_at`, `status` | Request audit by agent and time |
| **access_rules** | `entity_type`, `entity_id` | Access control lookups on every request |

## Tiered Rate Limiting

Rate limiting protects the platform at enterprise scale. The production profile configures two dimensions: **tier multipliers** (who is making the request) and **per-endpoint limits** (what they are requesting).

### Tier Multipliers

Each user class has a multiplier applied to base rate limits:

| Tier | Multiplier | Rationale |
|------|-----------|-----------|
| **Admin** | 10.0x | Platform administrators need full throughput for management operations |
| **User** | 1.0x | Standard authenticated users — the baseline rate |
| **A2A** | 5.0x | Agent-to-agent communication — elevated because orchestrating agents coordinate sub-agents |
| **MCP** | 5.0x | MCP server requests — elevated because tool-heavy workflows make many calls |
| **Service** | 5.0x | Internal service communication — trusted traffic |
| **Anon** | 0.5x | Unauthenticated requests — most restricted to prevent abuse |

The A2A and MCP tiers are critical for agent orchestration. When an agent coordinates sub-agents for multi-step workflows, each orchestration step generates A2A and MCP traffic that needs elevated rate limits.

### Per-Endpoint Limits

Base rates from the production profile (before tier multipliers apply):

| Endpoint | Requests/Second | At Admin Tier (10x) | At User Tier (1x) |
|----------|----------------:|--------------------:|-------------------:|
| **OAuth** | 10 | 100 | 10 |
| **Contexts** | 100 | 1,000 | 100 |
| **Tasks** | 50 | 500 | 50 |
| **Artifacts** | 50 | 500 | 50 |
| **Agent Registry** | 50 | 500 | 50 |
| **Agents** | 20 | 200 | 20 |
| **MCP Registry** | 50 | 500 | 50 |
| **MCP** | 200 | 2,000 | 200 |
| **Stream** | 100 | 1,000 | 100 |
| **Content** | 50 | 500 | 50 |

### Burst Multiplier

The burst multiplier of **3x** allows temporary spikes above the sustained rate. This uses a token bucket algorithm:

- Tokens accumulate at the sustained rate
- Bucket holds up to 3x the sustained rate
- Burst traffic drains the bucket; sustained over-limit traffic is throttled

This handles legitimate traffic patterns like page loads (many parallel requests) and agent orchestration (burst of sub-agent calls) without dropping requests.

## Port Management

The platform manages ports for agents and MCP servers within defined ranges:

| Service Type | Port Range | Example |
|-------------|-----------|---------|
| **Agents** | 9000-9999 | Agents allocated from this range |
| **MCP Servers** | 5000-5999 | MCP servers allocated from this range |
| **Main HTTP** | 8080 | Primary API and web interface |

In the standalone deployment model, each application instance runs its own agents and MCP servers on these ports. The load balancer routes to the appropriate instance based on the request path.

## Health Checks

Every application instance exposes a health check endpoint. The infrastructure monitors:

| Check | What It Verifies |
|-------|-----------------|
| **HTTP health** | Application is accepting requests |
| **Database connectivity** | PostgreSQL connection pool is healthy |
| **Agent health** | Registered agents are responding |
| **MCP server health** | MCP servers are reachable on their assigned ports |

Unhealthy instances are automatically removed from the load balancer rotation. The CLI provides manual health checking:

```bash
# Check all service health
systemprompt infra services status

# Check for errors
systemprompt infra logs view --level error --since 5m
```

## Infrastructure Topology

### Reference Configuration

The platform scales horizontally to match your requirements. A representative enterprise topology:

| Component | Specification | Scaling Strategy |
|-----------|--------------|---------|
| **Load Balancer** | Layer 7 with WebSocket support, TLS termination | HA pair |
| **Application Instances** | 4 vCPU, 8 GB RAM, container | Add instances as demand grows |
| **PostgreSQL Primary** | 8 vCPU, 32 GB RAM, SSD storage | Vertical scaling for write throughput |
| **PostgreSQL Read Replicas** | 4 vCPU, 16 GB RAM, SSD storage | Add replicas for read-heavy workloads |
| **CDN** | Static asset caching (CSS, JS, images) | Offload static traffic |

### Scaling Progression

| Stage | App Instances | DB Config | Notes |
|------:|-------------:|-----------|-------|
| Pilot | 1 | Single PostgreSQL | Development or proof-of-concept |
| Team | 2-3 | Primary + 1 replica | Standard deployment |
| Department | 5-8 | Primary + 2 replicas | Add CDN for static assets |
| Enterprise-wide | 8+ | Primary + 3+ replicas | Full production topology |

### Production Deployment

The production container runs as a non-root user with automated health checks:

| Setting | Value |
|---------|-------|
| **User** | Non-root (security best practice) |
| **Health check** | Automated HTTP health endpoint |
| **Environment** | Production profile loaded from environment |
| **Secrets** | Loaded from environment variables (not files) |
| **Logging** | JSON format for log aggregation |

## Scaling Checklist

Use this checklist when scaling for enterprise deployments:

| Step | Action | Verification |
|------|--------|-------------|
| 1 | Choose deployment model (sidecar, standalone, centralized, embedded) | Matches your infrastructure and team topology |
| 2 | Add application instances behind load balancer | `systemprompt infra services status` on each instance |
| 3 | Increase PostgreSQL `max_connections` | Check connection count vs limit in PostgreSQL |
| 4 | Add read replicas for analytics queries | Verify replication lag is under 1 second |
| 5 | Review rate limits for your traffic pattern | `systemprompt infra logs view --level warn` for rate limit hits |
| 6 | Enable CDN for static assets | Verify cache hit ratio for CSS/JS/images |
| 7 | Monitor agent port allocation | Ensure port ranges are not exhausted across instances |
| 8 | Set up log aggregation | JSON logs from all instances to central logging |
| 9 | Configure alerting | Health check failures, error rate spikes, database connection saturation |
