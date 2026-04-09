---
title: "Deployment Models"
description: "Four deployment models for the systemprompt.io AI governance platform: sidecar, gateway, embedded, and multi-tenant. Choose the model that fits your infrastructure."
author: "systemprompt.io"
slug: "deployment-models"
keywords: "deployment, sidecar, gateway, embedded, multi-tenant, Docker, infrastructure"
kind: "guide"
public: true
tags: ["devops", "deployment", "infrastructure"]
published_at: "2026-03-25"
updated_at: "2026-03-25"
after_reading_this:
  - "Understand the four deployment models and when to use each"
  - "Know how to configure Docker containers for production"
  - "Choose between sidecar, gateway, embedded, and multi-tenant architectures"
related_docs:
  - title: "Architecture Overview"
    url: "/documentation/architecture"
  - title: "Installation & Setup"
    url: "/documentation/installation"
  - title: "Scaling Architecture"
    url: "/documentation/scaling"
  - title: "Configuration & Profiles"
    url: "/documentation/configuration"
---

# Deployment Models

The platform supports four deployment models. Each model serves a different architectural need — choose the one that fits your infrastructure, or combine models across environments.

## Sidecar Mode

Run a platform instance alongside each agent instance. Each sidecar handles governance for its own agent process with no shared state beyond the database.

**When to use:** You need process-level isolation between agents, or your agents run on separate machines.

**Architecture:** One platform binary per agent. Each sidecar connects to the shared PostgreSQL database. Agent-to-sidecar communication is localhost-only with no network exposure.

**Advantages:**
- Complete process isolation between agents
- Failure in one sidecar does not affect others
- Simple horizontal scaling — add agents, add sidecars

**Trade-offs:**
- Higher resource usage (one binary per agent)
- Database connection pool sizing requires attention at scale

## Gateway Mode

A single platform instance serves as a centralised governance layer for all agents. All AI traffic routes through the gateway for authentication, authorisation, rate limiting, and auditing.

**When to use:** You want centralised governance with a single point of control, or you are starting with a small deployment and want simplicity.

**Architecture:** One platform binary handling all governance. Agents connect to the gateway over HTTP. The gateway proxies requests to MCP servers and applies governance policies.

**Advantages:**
- Single deployment to manage
- Centralised audit trail and configuration
- Lowest resource overhead

**Trade-offs:**
- Single point of failure (mitigate with load balancer and health checks)
- All traffic passes through one process

## Embedded Mode

Compile the governance library directly into your application. Zero network overhead between your application and governance logic.

**When to use:** Maximum performance is required, or you are building a custom application that embeds AI governance as a library.

**Architecture:** The platform is a Rust library — not a framework. Your application links against it and calls governance functions directly. No HTTP, no serialisation, no network hop.

**Advantages:**
- Zero network overhead
- Single process deployment
- Full control over the governance lifecycle

**Trade-offs:**
- Requires Rust integration
- Governance updates require recompiling your application

## Multi-Tenant Mode

A single deployment serves multiple organisations with tenant-level data isolation. Each tenant has its own users, agents, plugins, and analytics, all stored in the same database with tenant-scoped queries.

**When to use:** You are a service provider hosting the platform for multiple clients, or your organisation has divisions that need complete data isolation.

**Architecture:** One platform binary with tenant context injected per request. Database queries are scoped by tenant ID. Each tenant sees only its own data through the admin dashboard and API.

**Advantages:**
- Single deployment for multiple organisations
- Shared infrastructure with data isolation
- Centralised platform management

**Trade-offs:**
- More complex access control
- Shared database requires careful capacity planning

## Docker Deployment

All deployment models support Docker. The platform provides multi-stage Dockerfiles that produce minimal production images:

**Build stage:** Compiles the Rust binary with all dependencies. Build tools and source code are discarded.

**Runtime stage:** Copies only the compiled binary and static assets into a slim base image. The result is a small, secure container with no development dependencies.

Container images work with any orchestration platform: Docker Compose for local development, Kubernetes for production, or managed services like ECS and Cloud Run.

Health check endpoints (`/health/live` and `/health/ready`) provide liveness and readiness signals for container orchestrators.

## Choosing a Model

| Requirement | Recommended Model |
|-------------|-------------------|
| Starting out, small team | Gateway |
| Process isolation between agents | Sidecar |
| Maximum performance | Embedded |
| Hosting for multiple clients | Multi-Tenant |
| Development and testing | Gateway + Docker Compose |
| Production Kubernetes | Sidecar or Gateway behind load balancer |
