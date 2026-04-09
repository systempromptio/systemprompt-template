---
title: "AI Client Integration"
description: "How the control plane integrates with any AI client through open protocols — MCP for tool access, OAuth for authentication, plugin system for capability distribution, and REST API for programmatic access."
author: "systemprompt.io"
slug: "integration"
keywords: "integration, MCP, OAuth, plugin, API, AI client, CLI, collaborative AI, web AI, protocol"
kind: "guide"
public: true
tags: ["introduction", "integration", "mcp", "protocol"]
published_at: "2026-04-01"
updated_at: "2026-04-01"
after_reading_this:
  - "Understand how the control plane integrates with any AI client through open protocols"
  - "Know the four integration mechanisms: MCP, OAuth, plugins, and REST API"
  - "See how governance is applied at the protocol level regardless of client"
related_docs:
  - title: "Introduction"
    url: "/documentation/introduction"
  - title: "Platform Overview"
    url: "/documentation/platform-overview"
  - title: "MCP Servers"
    url: "/documentation/mcp-servers"
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "Tool Governance"
    url: "/documentation/tool-governance"
  - title: "Authentication"
    url: "/documentation/authentication"
  - title: "Distribution Channels"
    url: "/documentation/distribution-channels"
---

# AI Client Integration

## Integration Architecture

The control plane sits between AI clients and backend services. Integration happens at the protocol level, so it works with any client that speaks these standards:

```
┌─────────────────────────────────────────────────────────────┐
│                      AI Clients                              │
│  CLI Agents, Collaborative AI, Web AI Assistants,            │
│  Custom Agents, MCP Clients, REST Consumers                  │
├─────────────────────────────────────────────────────────────┤
│          ▼  Integration Protocols  ▼                         │
│  MCP (tools)  ·  OAuth (auth)  ·  Plugins (distribution)    │
│  REST API (programmatic)  ·  Git (marketplace)               │
├─────────────────────────────────────────────────────────────┤
│          ▼  Governance Control Plane  ▼                      │
│  Auth, access control, rate limiting, audit,                 │
│  cost tracking, compliance, tool governance                  │
├─────────────────────────────────────────────────────────────┤
│                    Backend Services                           │
│  LLM Providers, MCP Servers, Databases, APIs,                │
│  Internal Services, Data Pipelines, Vector Stores            │
└─────────────────────────────────────────────────────────────┘
```

## Four Integration Mechanisms

### 1. MCP — Tool Access

The **Model Context Protocol (MCP)** is the standard interface for tool access. Any MCP-compatible AI client can discover and invoke governed tools through the control plane.

Each MCP server managed by the control plane:

- Has its own **OAuth authentication** configuration
- Exposes a defined set of **tools** that clients can discover and invoke
- Operates under **centralised governance policies** — access control, rate limiting, audit logging
- Is **registered and monitored** through the admin dashboard
- Runs on an **isolated port** with per-server configuration

The control plane acts as the central registry for all MCP servers. When a client connects, it discovers available tools based on the user's role and department. Every tool invocation passes through the governance pipeline.

See [MCP Servers](/documentation/mcp-servers) for server configuration and [Tool Governance](/documentation/tool-governance) for access control.

### 2. OAuth — Authentication

The control plane provides a unified identity system that spans all AI surfaces. A user is authenticated once and that identity is enforced consistently across every interaction.

| Method | Use |
|--------|-----|
| **WebAuthn passkeys** | Primary credential — biometrics, hardware keys, platform authenticators |
| **OAuth 2.0 + PKCE** | Standard authentication flow for dashboards, APIs, and AI clients |
| **Magic links** | Email-based alternative for marketplace access |
| **JWT tokens** | Stateless session tokens used across all surfaces |

**Third-party identity provider integration** — The authentication flow integrates with your existing identity provider (Okta, Azure AD, Google Workspace, or any OIDC-compliant provider). There is no separate user database to manage.

See [Authentication](/documentation/authentication) for the full technical flow.

### 3. Plugins — Capability Distribution

Plugins are the unit of distribution for governed AI capabilities. A plugin bundles:

- **Skills** — Reusable AI capabilities (e.g., "summarise a document", "review a pull request")
- **Agents** — AI workers configured with specific instructions, tools, and system prompts
- **MCP Servers** — Tool server configurations
- **Hooks** — Event-driven triggers that fire on specific actions

Plugins are scoped by role and department. Administrators configure which capabilities are available to which users. Distribution happens through **git repositories** — the control plane serves each user's personalised marketplace as a git repo over HTTP.

```bash
# Install a governed plugin marketplace
claude plugin marketplace add https://your-instance.example.com/api/public/marketplace/{user_id}.git
```

Any git-compatible tooling can clone and inspect the marketplace. See [Plugins](/documentation/plugins) for the full guide and [Distribution Channels](/documentation/distribution-channels) for all distribution methods.

### 4. REST API — Programmatic Access

The control plane exposes a full HTTP API for programmatic integration. Every operation available in the admin dashboard and CLI is also available via the API:

- User and role management
- Plugin and skill CRUD
- Analytics and metrics queries
- Governance policy configuration
- Audit trail queries

The API uses JWT authentication and follows the same governance rules as every other surface.

---

## Protocol-Level Governance

Every interaction through any integration mechanism passes through the same governance pipeline:

| Governance Layer | What Happens |
|-----------------|-------------|
| **Authentication** | The user's identity is verified via the shared auth system |
| **Authorisation** | Access control checks whether this user can invoke this tool |
| **Rate Limiting** | Per-role rate limits prevent resource abuse |
| **Execution Logging** | The tool call, its parameters, and its result are recorded |
| **Event Hooks** | Configured hooks fire (e.g., notify a channel, trigger a workflow) |
| **Cost Tracking** | Token consumption is attributed to the user, department, and agent |

This governance is transparent to the end user. They use their AI client normally — the governance layer adds control without adding friction.

The key principle: **one governance layer, every surface**. Whether a user is in a CLI, a collaborative workspace, or a web browser, the same access control, audit trail, and cost tracking applies.

---

## Supported AI Clients

The control plane works with any AI client that supports MCP, OAuth, or REST. Current supported surfaces include:

| Surface | Integration | Status |
|---------|-------------|--------|
| **CLI agents** | Plugin system + MCP | Fully operational |
| **Collaborative AI workspaces** | MCP + plugin system | Supported |
| **Web-based AI assistants** | MCP servers with OAuth | Supported |
| **Custom MCP clients** | MCP protocol | Supported |
| **REST consumers** | HTTP API with JWT | Supported |

Because integration is protocol-based, any new AI client that supports MCP automatically works with the governance layer. No custom integration code required.

---

## Enterprise Deployment

For rolling out to a team, administrators can:

1. **Configure plugins centrally** — Skills, agents, MCP servers, and hooks are bundled into plugins and assigned to roles and departments
2. **Distribute via git** — Each user gets a personalised marketplace served as a git repository
3. **Integrate with existing identity** — OAuth connects to your existing IDP for seamless SSO
4. **Monitor from the dashboard** — Real-time visibility into who is using what, usage costs, and governance compliance

The install URL can be included in developer onboarding scripts, device management policies, or distributed through any internal channel.

See [Marketplace](/documentation/marketplace) for marketplace management and [Distribution Channels](/documentation/distribution-channels) for the full distribution guide.

## Feature Pages

For detailed feature information, see:

- [MCP Integration](/features/mcp-integration) — Native protocol support, OAuth2, and tool governance
- [No Vendor Lock-In](/features/no-vendor-lock-in) — Open standards and portable formats
- [Agent Governance](/features/agent-governance) — RBAC, audit trails, and secret detection
- [Skills & Plugins](/features/skills-and-plugins) — Governed bundles and multi-channel distribution
