---
title: "Integration: Claude AI (claude.ai)"
description: "How the governance platform extends to claude.ai through MCP servers, unified authentication, and consistent governance policies across all Claude surfaces."
author: "systemprompt.io"
slug: "integration-claude-ai"
keywords: "Claude AI, claude.ai, web, integration, governance, MCP, tools, authentication, WebAuthn"
kind: "guide"
public: true
tags: ["introduction", "integration", "claude-ai", "web", "mcp"]
published_at: "2026-03-27"
updated_at: "2026-03-27"
after_reading_this:
  - "Understand how the platform extends governance to claude.ai"
  - "Know how MCP servers bridge claude.ai to governed tools"
  - "See how unified authentication works across all Claude surfaces"
related_docs:
  - title: "Introduction to the Platform"
    url: "/documentation/introduction"
  - title: "MCP Servers"
    url: "/documentation/mcp-servers"
  - title: "Authentication"
    url: "/documentation/authentication"
  - title: "Tool Governance"
    url: "/documentation/tool-governance"
  - title: "Platform Overview"
    url: "/documentation/platform-overview"
---

# Integration: Claude AI (claude.ai)

**Claude AI at claude.ai is Anthropic's web-based AI assistant. The governance platform extends to claude.ai through MCP servers — the same Model Context Protocol infrastructure that governs Claude Code and Claude Cowork. Users authenticate once via the shared identity system, and every tool call from claude.ai passes through the same governance pipeline: access control, audit logging, cost tracking, and rate limiting.**

---

## How It Works

Claude AI connects to external tools through MCP servers. The governance platform manages these MCP servers centrally, so when a user in claude.ai invokes a tool, the request routes through the governance pipeline.

The flow:

1. **User authenticates** — OAuth-based authentication identifies the user and establishes their permissions
2. **Claude AI discovers tools** — MCP servers registered with the platform expose governed tools to claude.ai
3. **User invokes a tool** — The tool call passes through the governance pipeline
4. **Governance applied** — Authentication verified, access control checked, rate limits enforced, execution logged
5. **Result returned** — The tool result flows back to claude.ai with full audit trail recorded

## MCP Servers as the Bridge

MCP (Model Context Protocol) servers are the integration mechanism between claude.ai and your organisation's governed tools. The platform acts as the central registry and governance layer for all MCP servers.

Each MCP server:

- Has its own **OAuth authentication** configuration, managed centrally
- Exposes a defined set of **tools** that claude.ai can discover and invoke
- Operates under the **same governance policies** as tools accessed from Claude Code or Cowork
- Is **registered and monitored** through the admin dashboard

This means your organisation controls exactly which tools are available through claude.ai, who can access them, and what happens when they are used.

See [MCP Servers](/documentation/mcp-servers) for server configuration and management.

## Unified Authentication

The platform provides a single identity system that spans all Claude surfaces. A user authenticated in the admin dashboard, using Claude Code, or working in claude.ai is the same user — with the same roles, the same permissions, and the same audit identity.

### How Authentication Works

The platform supports multiple authentication methods, all tied to the same user identity:

| Method | Primary Use |
|--------|------------|
| **WebAuthn passkeys** | Primary credential — biometrics, hardware keys, platform authenticators |
| **OAuth 2.0 + PKCE** | Dashboard and API authentication flow |
| **Magic links** | Email-based alternative for marketplace access |
| **JWT tokens** | Stateless session tokens used across all surfaces |

### Third-Party Identity Provider Integration

The OAuth 2.0 authentication flow can be fully integrated with your existing identity provider:

- **Okta, Azure AD, Google Workspace** — Any OIDC-compliant provider can be configured as the upstream identity source
- **Single sign-on** — Users authenticate with their existing corporate credentials
- **Centralised user management** — User provisioning, role assignment, and deprovisioning flow through your existing IDP
- **Shared authentication** — The same identity is used for the admin dashboard, MCP server access, AI agent authentication, and API calls

This means there is no separate user database to manage. Your existing identity infrastructure is the source of truth, and the governance platform enforces AI-specific policies (roles, departments, tool access) on top of it.

See [Authentication](/documentation/authentication) for the full technical flow and [Users](/documentation/users) for user management.

## Consistent Governance Across Surfaces

The governance platform enforces the same policies regardless of which Claude surface a user is working in:

| Governance Capability | Claude Code | Claude Cowork | claude.ai |
|----------------------|-------------|---------------|-----------|
| **User authentication** | Plugin auth | Session auth | OAuth + MCP |
| **Access control** | Role-based | Role-based | Role-based |
| **Tool governance** | Per-tool | Per-tool | Per-tool |
| **Rate limiting** | Per-role | Per-role | Per-role |
| **Audit logging** | Full | Full | Full |
| **Cost tracking** | Per-user | Per-user | Per-user |

This consistency is the core value proposition. One governance layer governs all AI usage in your organisation. The surface changes; the governance does not.

See [Tool Governance](/documentation/tool-governance) for access control configuration and [Events & Audit Trails](/documentation/events) for the unified audit log.
