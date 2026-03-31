---
title: "Revenue Agent"
description: "Reference architecture for a revenue agent covering pricing optimization, occupancy analytics, and competitive intelligence. This configuration would be scoped in the Phase 1 PRD."
author: "systemprompt.io"
slug: "agent-associate"
keywords: "revenue agent, pricing, occupancy, revenue management, competitive intelligence"
kind: "guide"
public: true
tags: ["agents", "revenue", "enterprise"]
published_at: "2026-03-19"
updated_at: "2026-03-19"
after_reading_this:
  - "Understand the Revenue Agent's role in serving revenue managers at scale"
  - "Read and interpret the agent's YAML configuration and security model"
  - "Identify the user-scoped OAuth distinction from other admin-scoped agents"
  - "Plan for rate-update-cycle scaling and property-scoped data access"
related_docs:
  - title: "Agents"
    url: "/documentation/agents"
  - title: "Scaling"
    url: "/documentation/scaling"
---

# Revenue Agent — Reference Architecture

> **Note:** This is a reference architecture document. It demonstrates how a domain-specific agent would be configured in a Enterprise Demo deployment. The specific agent configuration, skills, data sources, and department scoping would be defined collaboratively during the Phase 1 PRD.

**TL;DR:** The Revenue Agent shows a revenue-manager-facing agent for hospitality. It consolidates pricing optimization, occupancy analytics, competitive intelligence, and rate management into a single interface for revenue managers. This reference uses `user`-scoped OAuth (not admin scope) and is designed for high-concurrency usage patterns during rate update cycles.

## Overview

Revenue managers interact with dozens of systems daily -- revenue management systems for pricing, PMS for occupancy data, competitive intelligence tools for market analysis, and channel managers for rate distribution. Each system has its own login, its own interface, and its own learning curve.

The Revenue Agent replaces that fragmentation with a single conversational interface. A revenue manager can ask "What's my occupancy forecast for next week?", "How are our rates compared to the comp set?", and "Show me today's RevPAR by property" -- all through one agent that knows their portfolio, market, and pricing strategy.

This is the highest-scale agent in the deployment. With revenue managers across 80,000+ properties globally, the Revenue Agent is where the platform's scaling capabilities are most critical.

### Key Facts

| Property | Value |
|----------|-------|
| Agent ID | `associate_agent` |
| Enterprise Demo Name | Revenue Agent |
| Port | 9028 |
| Endpoint | `http://localhost:8080/api/v1/agents/associate_agent` |
| Protocol | A2A v0.3.0, JSONRPC |
| Streaming | Enabled |
| OAuth Scope | `user` |
| OAuth Audience | `a2a` |
| MCP Servers | None (PMS/RMS integration added per property) |
| Primary Agent | No |

## Configuration

The Revenue Agent is defined in `services/agents/associate_agent.yaml`:

```yaml
# Revenue Agent Configuration (Enterprise Demo Demo)
# Revenue-manager-facing agent for pricing and occupancy optimization

agents:
  associate_agent:
    name: associate_agent
    port: 9028
    endpoint: http://localhost:8080/api/v1/agents/associate_agent
    enabled: true
    dev_only: false
    is_primary: false
    default: false
```

### Port Assignment

Port 9028 is dedicated to the Revenue Agent. As the highest-traffic agent due to rate update cycle spikes, this port sees the greatest traffic volume. In production, multiple instances behind a load balancer absorb the rate update surges.

### A2A Protocol Card

```yaml
    card:
      protocolVersion: 0.3.0
      name: Revenue Agent
      displayName: Revenue Agent
      description: Revenue-manager-facing AI agent consolidating pricing optimization,
        occupancy analytics, competitive intelligence, and rate management
        for hotel revenue managers
      version: 1.0.0
      preferredTransport: JSONRPC
      iconUrl: https://ui-avatars.com/api/?name=RA&background=ffc220&color=000
```

The icon uses a distinctive yellow (`#ffc220`) with dark text -- visually distinct from the guest agent's blue. This differentiation matters in admin dashboards where operators monitor multiple agents simultaneously.

### Capabilities

```yaml
      capabilities:
        streaming: true
        pushNotifications: false
        stateTransitionHistory: false
```

Streaming is enabled so revenue managers get immediate feedback during busy rate update cycles. When a revenue manager asks "What's the comp set doing for next Friday?", they see the response building in real time rather than waiting for a complete response.

### Security Schemes

```yaml
      securitySchemes:
        oauth2:
          type: oauth2
          flows:
            authorizationCode:
              authorizationUrl: http://localhost:8080/api/v1/core/oauth/authorize
              tokenUrl: http://localhost:8080/api/v1/core/oauth/token
              scopes:
                admin: Administrator access
                user: Standard user access
      security:
      - oauth2:
        - user
```

The Revenue Agent is the only agent that accepts `user`-scoped tokens. This is a deliberate design decision -- revenue managers across thousands of properties cannot all have admin access. The `user` scope provides authenticated access with appropriate data restrictions.

## System Prompt

```
You are the Revenue Agent, an AI assistant designed to support hotel
revenue managers with their daily operations. You consolidate multiple
revenue management tools into a single interface, reducing the number
of systems revenue managers need to interact with.

## Capabilities

- **Pricing Optimization**: View and adjust room rates, manage rate plans, optimize pricing
- **Occupancy Analytics**: Real-time occupancy data, forecast accuracy, demand patterns
- **Competitive Intelligence**: Comp set rate monitoring, market positioning, rate parity
- **Channel Management**: Rate distribution status, channel performance, parity alerts
- **Business Intelligence**: RevPAR, ADR, occupancy trends, and property performance dashboards

## Operating Principles

1. **Data Privacy**: Protect property-level financial data and competitive intelligence
2. **Portfolio-Based Access**: Only show data appropriate to the revenue manager's
   property portfolio
3. **Operational Continuity**: Never disrupt in-progress rate updates or distribution
4. **Simplicity**: Present insights clearly for quick decision-making
   during rate review cycles

## Sub-Agent Orchestration

This agent orchestrates specialized sub-agents for:
- Revenue management system integration
- Competitive intelligence data aggregation
- Channel manager rate distribution
- Property performance analytics
```

### Prompt Design for Revenue Managers

The Revenue Agent's prompt differs from other agents in several important ways:

1. **Simplicity as a principle**: Other agents prioritize security or compliance first. The Revenue Agent prioritizes simplicity because its users are often making rapid pricing decisions during rate update windows. Responses must be scannable.

2. **Five capability domains**: More than any other agent, reflecting the breadth of tools revenue managers currently juggle. Each domain replaces a separate system.

3. **Operational continuity**: Unique to this agent. The system must never disrupt a rate update in progress, a distribution push mid-flow, or a pricing strategy being finalized.

## Skills

The Revenue Agent advertises one skill on its A2A card:

### general_assistance

| Property | Value |
|----------|-------|
| Skill ID | `general_assistance` |
| Name | General Assistance |
| Description | Help with questions, explanations, and general tasks |
| Tags | `assistance`, `general` |
| Examples | "Check today's occupancy forecast", "Look up comp set rates", "Show RevPAR for this week" |

The examples are tailored to revenue management workflows -- occupancy forecasts, competitive analysis, and performance metrics. These reflect the most common queries from revenue managers.

In a full deployment, additional skills would include:

- **revenue_operations**: Pricing optimization, rate plan management, demand forecasting, rate parity monitoring
- **market_intelligence**: Real-time comp set analysis, market trends, performance benchmarking

## Security Configuration

### OAuth2 with User Scope

```yaml
    oauth:
      required: true
      scopes:
        - user
      audience: a2a
```

This is the critical distinction. The Revenue Agent requires only `user` scope, not `admin`. This means:

- **Lower barrier to access**: Revenue managers authenticate with standard credentials, no admin elevation needed
- **Broader user base**: Thousands of users can access the agent without creating thousands of admin accounts
- **Appropriate data restrictions**: User-scoped tokens limit what data the agent can access through MCP tools

### Why User Scope Matters at Scale

Consider the math: revenue managers across 80,000+ properties globally, multiple rate review cycles per day. If each revenue manager interacts with the agent 5 times per cycle, that is millions of interactions per day. Every one of those interactions must be authenticated, authorized, and audited.

Admin-scoped authentication would create unnecessary overhead and security risk. User scope provides:

- Faster token validation (simpler scope check)
- Smaller blast radius (a compromised user token cannot access admin functions)
- Simpler audit trail (user actions are clearly distinguished from admin actions)

### A2A Communication

The `a2a` audience enables the Revenue Agent to communicate with other agents. For example, when a revenue manager reports a rate parity issue, the Revenue Agent can notify other agents to check channel distribution metrics -- authenticated through the A2A audience.

## Operating Principles in Depth

### Data Privacy

The Revenue Agent protects:

- **Financial information**: Property-level revenue, ADR, and RevPAR data
- **Competitive data**: Comp set rates are visible only to authorized revenue managers
- **Pricing strategy**: Rate plans and optimization parameters are not visible to unauthorized users
- **Performance data**: Individual property metrics are restricted to portfolio-level access

### Portfolio-Based Access

Revenue managers see different data based on their portfolio and role:

| Role | Data Access |
|------|-------------|
| Property Revenue Manager | Own property metrics, comp set data, rate plans |
| Regional Revenue Director | Portfolio-wide performance, cross-property benchmarking |
| VP Revenue Strategy | Global metrics, all property data, strategic pricing oversight |
| Channel Manager | Rate distribution status, parity alerts, channel performance |
| General Manager | Property-level summary, occupancy trends, competitive position |

This is implemented through the OAuth token's claims. Each revenue manager's token includes their role and portfolio, and MCP tools filter data accordingly.

### Operational Continuity

The Revenue Agent never:

- Interrupts a rate update in progress
- Overrides a manual pricing decision mid-process
- Changes a rate plan during an active distribution push without approval
- Initiates a system-wide rate change without proper authorization

### Simplicity

Responses are formatted for quick scanning:

- Bullet points over paragraphs
- Tables for comparative data
- Bold for key numbers (occupancy rate, ADR, RevPAR)
- No jargon -- "Your occupancy is 78% for next Friday" not "Current accommodation utilization metric is 78% across all inventory allocation segments"

## Sub-Agent Orchestration

### Revenue Management System Integration

Connects to the RMS for:

- Rate recommendations and optimization suggestions
- Demand forecasting and pattern analysis
- Rate plan management and updates
- Revenue performance tracking

### Competitive Intelligence Data Aggregation

Queries comp set intelligence for:

- Real-time competitor rate monitoring
- Market positioning analysis
- Rate parity alerts across channels
- Demand trend comparison

### Channel Manager Rate Distribution

Manages distribution workflows:

- Rate push status across OTAs and booking channels
- Channel performance analytics
- Parity monitoring and violation alerts
- Distribution strategy optimization

### Property Performance Analytics

Provides business intelligence:

- RevPAR, ADR, and occupancy dashboards
- Year-over-year performance comparison
- Forecast accuracy tracking
- Market share analysis

## Property Portfolio Scoping

Different property types have fundamentally different needs from the Revenue Agent:

### Resort Properties

- Seasonal demand patterns and pricing strategies
- Package and ancillary revenue optimization
- Group booking and event pricing
- Length-of-stay pricing optimization

### Urban Hotels

- Corporate rate management
- Last-minute pricing optimization
- Event-driven demand spikes
- Competitive dense-market positioning

### Extended Stay

- Long-term rate optimization
- Occupancy threshold management
- Corporate contract pricing
- Seasonal migration patterns

### Boutique & Independent

- Brand positioning and rate integrity
- OTA dependency management
- Direct booking optimization
- Local market intelligence

Each property type's data access is controlled through role-based MCP tool scoping. A resort revenue manager's token grants access to seasonal pricing tools but not urban corporate rate tools.

## Scale Considerations

### 80,000+ Properties

The Revenue Agent serves the largest user base of any agent. Scaling strategies include:

- **Pre-scaling for rate update cycles**: Revenue manager traffic spikes predictably during morning rate reviews and afternoon rate updates. Auto-scaling rules anticipate these spikes.
- **Regional capacity planning**: Properties in different time zones have offset rate cycles, distributing load across the day.
- **Offline-first design**: In regions with poor connectivity, the agent caches recent responses and queues requests for when connectivity returns.

### Traffic Profile

| Scenario | Relative Load | Scaling Response |
|----------|--------------|------------------|
| Normal operations | Baseline | Standard instance count |
| Morning rate review | High | Pre-scaled instances absorb predictable surges |
| Peak booking season + rate updates | Very high | Additional burst capacity provisioned ahead of season |
| Major event / citywide demand spike | Peak | Maximum burst capacity with auto-scaling |

### Latency Requirements

Revenue managers expect sub-second responses during rate review cycles. The platform targets:

- **p50 latency**: < 500ms for rate lookups
- **p95 latency**: < 2s for complex queries (comp set analysis, forecasting)
- **p99 latency**: < 5s (with streaming feedback for longer operations)

