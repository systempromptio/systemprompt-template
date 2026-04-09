---
title: "systemprompt.io Agent Architecture"
description: "How systemprompt.io's four super-agents map to the platform's plugin-agent-skill architecture. Architecture overview for guest, revenue, platform, and distribution agents."
author: "systemprompt.io"
slug: "enterprise-demo-agents"
keywords: "enterprise-demo, agents, super-agents, architecture, governance"
kind: "guide"
public: false
tags: ["enterprise-demo", "agents", "enterprise", "architecture"]
published_at: "2026-03-19"
updated_at: "2026-03-19"
after_reading_this:
  - "Understand how systemprompt.io's four super-agents map to the platform's plugin-agent-skill model"
  - "Identify which agent serves which user population and on which port"
  - "Trace the governance controls applied to each agent domain"
  - "Navigate to individual agent documentation for deeper configuration details"
related_docs:
  - title: "Agents"
    url: "/documentation/agents"
  - title: "Access Control"
    url: "/documentation/access-control"
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "systemprompt.io"
    url: "/documentation/demo"
---

# systemprompt.io Agent Architecture

**TL;DR:** This page maps systemprompt.io's four super-agent strategy to the platform's agent configurations. Each super-agent maps to a platform agent backed by the Enterprise Agent Governance plugin, with dedicated ports, skills, security scopes, and audit controls.

> **Sources:** This architecture is based on systemprompt.io's AI strategy:
> - [systemprompt.io AI Strategy](https://www.systemprompt.io/blog) -- describes the four super-agent strategy for hospitality technology
> - Platform Agent uses MCP + A2A for developer orchestration
> - Port assignments, OAuth scopes, and skill configurations below are **our platform's demonstration** of these capabilities, not systemprompt.io's internal implementation.

## Why Four Super-Agents

Large enterprises accumulate AI sprawl quickly. Guest services has one chatbot, revenue management has another, engineering has its own copilot, and distribution partner management runs on spreadsheets and email. The result is inconsistent governance, duplicated infrastructure, and security blind spots.

systemprompt.io's strategy consolidates AI interactions into four domains, each served by a single super-agent:

1. **Guest Agent** -- guest-facing interactions: hotel search, booking assistance, stay management
2. **Revenue Agent** -- revenue manager workflows: pricing optimization, occupancy data, competitive intelligence
3. **Platform Agent** -- developer orchestration via MCP + A2A protocols
4. **Distribution Agent** -- OTA and channel partner onboarding, rate management, and campaigns

Four agents means four security boundaries, four audit domains, and four independently scalable services -- all governed by a single plugin.

## Our Platform's Demonstration

The table below shows how our platform demonstrates these capabilities. Port assignments and OAuth scopes are our configuration, not systemprompt.io's.

| Agent ID | Published Name | Demo Port | Target Users | Purpose | OAuth Scope |
|----------|-------------|------|--------------|---------|-------------|
| `customer_agent` | Guest Agent | 9029 | Hotel guests (app/web) | Booking, stay management, recommendations, concierge services | `admin` |
| `associate_agent` | Revenue Agent | 9028 | Revenue managers | Pricing optimization, occupancy analytics, competitive intelligence | `user` |
| `developer_agent` | Platform Agent | 9026 | Engineering teams | Code review, CI/CD, security scanning | `admin` |
| `partner_agent` | Distribution Agent | 9027 | OTAs/channel partners | Channel management, rate parity, distribution optimization | `admin` |

Each agent runs on its own port, has its own A2A protocol card, and declares its own skills and security requirements. The Enterprise Agent Governance plugin bundles all four agents together with shared MCP servers and governance skills.

## Architecture Overview

```
Enterprise Agent Governance Plugin (enterprise-demo)
│
├── Shared Skills
│   ├── enterprise_agent_governance
│   ├── enterprise_security_compliance
│   ├── enterprise_mcp_management
│   └── systemprompt_cli + admin skills
│
├── customer_agent (port 9029)
│   ├── Skills: general_assistance
│   ├── MCP Servers: (none configured — domain tools added per deployment)
│   ├── Security: OAuth2, scope: admin, audience: a2a
│   ├── Transport: JSONRPC, streaming enabled
│   └── Target: Hotel guests via web, mobile, in-property kiosks
│
├── associate_agent (port 9028)
│   ├── Skills: general_assistance
│   ├── MCP Servers: (none configured — PMS/RMS integration added per property)
│   ├── Security: OAuth2, scope: user, audience: a2a
│   ├── Transport: JSONRPC, streaming enabled
│   └── Target: Revenue managers and hotel operators
│
├── developer_agent (port 9026)
│   ├── Skills: general_assistance, dev_rust_standards, dev_architecture_standards
│   ├── MCP Servers: systemprompt
│   ├── Security: OAuth2, scope: admin, audience: a2a
│   ├── Transport: JSONRPC, streaming enabled
│   └── Target: Engineering teams via CLI, IDE, chat
│
└── partner_agent (port 9027)
    ├── Skills: general_assistance
    ├── MCP Servers: (none configured — OTA integration added per channel partner)
    ├── Security: OAuth2, scope: admin, audience: a2a
    ├── Transport: JSONRPC, streaming enabled
    └── Target: OTAs, channel managers, and distribution partners
```

## The Enterprise Agent Governance Plugin

All four agents are bundled under the `enterprise-demo` plugin. This plugin is the single governance boundary that controls:

- **Which agents exist** -- the plugin's `agents.include` list explicitly names all four agents plus the admin agent
- **Which skills are available** -- 11 skills covering CLI operations, admin management, and enterprise governance
- **Which MCP servers are accessible** -- the `systemprompt` MCP server is shared across the plugin
- **Which roles can access the plugin** -- currently scoped to the `admin` role

```yaml
# From services/plugins/enterprise-demo/config.yaml
plugin:
  id: enterprise-demo
  name: Enterprise Agent Governance
  description: Enterprise-scale AI agent governance platform...
  agents:
    source: explicit
    include:
    - systemprompt_admin
    - customer_agent
    - associate_agent
    - developer_agent
    - partner_agent
  mcp_servers:
  - systemprompt
  roles:
  - admin
```

This means a single plugin configuration governs authentication, authorization, tool access, and audit policy for all four super-agents. Changes to the plugin propagate to every agent simultaneously.

## How the Platform Governs These Agents

### RBAC: Who Can Access Which Agent

Role-based access control determines which users see which agents. The plugin is assigned to the `admin` role, but individual agents further restrict access through OAuth scopes:

- **`admin` scope agents** (guest, platform, distribution): Require elevated privileges. These agents handle sensitive data -- guest PII, source code, partner contracts.
- **`user` scope agents** (revenue): Available to standard authenticated users. This is deliberate -- revenue managers need access without admin overhead.

Non-admin users only see agents from plugins assigned to their roles. A revenue manager logging into the platform sees only the Revenue Agent. A developer with admin access sees all four.

### Audit Trails: Every Interaction Logged

Every conversation, tool call, and agent action generates audit events. The platform captures:

- **Session events**: Who started a conversation with which agent, when, and from where
- **Tool usage**: Which MCP tools were invoked, their inputs (sanitized), outputs, and execution time
- **Agent decisions**: When an agent escalates, delegates, or refuses a request

Query audit data with:

```bash
# Recent errors across all agents
systemprompt infra logs view --level error --since 1h

# Request-level audit trail
systemprompt infra logs request list --limit 20

# Full conversation context for a specific request
systemprompt infra logs audit <request-id> --full
```

### MCP Tool Scoping: Which Tools Each Agent Can Use

Each agent declares its MCP server connections in its YAML configuration. The Platform Agent connects to the `systemprompt` MCP server; the other three agents currently have no MCP servers configured, meaning they operate without external tool access until domain-specific servers are added.

This is a security feature. An agent with no MCP servers cannot access external systems, databases, or APIs. Tools are added explicitly, not inherited.

### Rate Limiting: Per-Tier Controls

The platform enforces rate limits at multiple levels:

- **Per-user**: Prevents any single user from monopolizing agent capacity
- **Per-agent**: Each agent port has independent capacity limits
- **Per-tier**: Service and MCP tiers get higher limits (5x base rate)
- **Per-tool**: Individual MCP tool calls are rate-limited at 200 req/s base

## Scaling to systemprompt.io's User Base

The enterprise demo is designed for enterprise-scale deployments across all four agent domains. The architecture supports this through:

### Stateless Agent Design

Each agent is stateless -- conversation context is managed by the platform, not by the agent process. This means:

- Agents can be horizontally scaled behind a load balancer
- Any instance of `customer_agent` can handle any guest request
- No sticky sessions required
- Agent restarts do not lose conversation state

### Per-Agent Port Management

Dedicated ports per agent (9026-9029) enable independent scaling:

- High-traffic agents (guest, revenue) can run more instances
- Low-traffic agents (platform, distribution) can run fewer
- Port-based routing simplifies load balancer configuration
- Health checks target individual agent ports

### Capacity Planning by Agent Domain

| Agent | Relative Volume | Traffic Pattern | Scaling Strategy |
|-------|----------------|-----------------|------------------|
| Guest | Highest | Spiky (peak booking seasons, holidays) | Auto-scale on queue depth |
| Revenue | High | Business hours (rate update windows, morning reviews) | Pre-scale for rate update cycles |
| Platform | Moderate | Business hours, CI/CD spikes | Scale on build queue |
| Distribution | Lower | Business hours, end-of-quarter | Fixed capacity with burst |

## Agent-to-Agent Communication

The platform supports the A2A (Agent-to-Agent) protocol for cross-agent orchestration. Each agent's configuration includes:

- **Protocol version**: 0.3.0 (current A2A specification)
- **Preferred transport**: JSONRPC
- **OAuth audience**: `a2a` (all agents share this audience for mutual authentication)

### Cross-Agent Scenarios

Real enterprise workflows frequently cross agent boundaries:

1. **Guest complaints involving rate discrepancies**: Guest Agent detects a pattern of complaints about pricing, triggers Revenue Agent to investigate rate parity metrics.

2. **Occupancy alerts to revenue managers**: Distribution Agent receives a booking surge notification from an OTA, Revenue Agent alerts revenue managers to adjust pricing strategy.

3. **Security incident response**: Platform Agent detects a vulnerability in a channel integration, Distribution Agent is notified to review affected partner data flows, Guest Agent adjusts service messaging.

### Communication Patterns

A2A communication supports both synchronous and asynchronous patterns:

- **Blocking**: Agent A sends a request to Agent B and waits for a response. Used for real-time data lookups.
- **Async**: Agent A sends a request and continues processing. Agent B responds when ready. Used for long-running operations like compliance checks.
- **Timeout controls**: Each cross-agent call has configurable timeouts to prevent cascading failures.

## Security Boundaries Between Agents

Each agent operates within its own security boundary:

### Data Isolation

- Guest Agent cannot access revenue manager pricing strategies or performance data
- Revenue Agent cannot access distribution partner contracts or commission rates
- Platform Agent cannot access guest PII
- Distribution Agent cannot access another channel partner's data

### Authentication Isolation

- Each agent validates OAuth tokens independently
- Scope requirements differ per agent (admin vs. user)
- Cross-agent calls require mutual A2A authentication

### Tool Isolation

- MCP server connections are per-agent, not shared
- The Platform Agent's access to the `systemprompt` MCP server does not grant the Guest Agent the same access
- Adding a new MCP server to one agent does not affect others

## Next Steps

Each agent has its own detailed documentation page covering configuration, system prompts, skills, and operational guidance:

- [Guest Agent](/documentation/agent-customer) -- hotel guest services: booking, stay management, concierge
- [Revenue Agent](/documentation/agent-associate) -- revenue management and pricing optimization
- [Platform Agent](/documentation/agent-developer) -- code, CI/CD, security
- [Distribution Agent](/documentation/agent-partner) -- OTAs, channel management, rate distribution
For tool governance across all agents, see [Tool Governance](/documentation/tool-governance).
