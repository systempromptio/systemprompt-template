---
title: "Guest Agent"
description: "Configuration and governance of the customer_agent for hotel guest services, booking management, and concierge assistance across all guest touchpoints."
author: "systemprompt.io"
slug: "agent-customer"
keywords: "guest agent, hotel, booking, concierge, guest services"
kind: "guide"
public: false
tags: ["agents", "guest", "enterprise"]
published_at: "2026-03-19"
updated_at: "2026-03-19"
after_reading_this:
  - "Understand the Guest Agent's role and configuration"
  - "Read and interpret the agent's YAML configuration"
  - "Identify the skills, security settings, and sub-agent orchestration patterns"
  - "Monitor and troubleshoot the Guest Agent in production"
related_docs:
  - title: "Agents"
    url: "/documentation/agents"
  - title: "Access Control"
    url: "/documentation/access-control"
---

# Guest Agent

**TL;DR:** The Guest Agent (`customer_agent`) demonstrates a guest-facing agent for hospitality. It handles guest-facing interactions -- hotel search, booking assistance, stay management, concierge services, and loyalty program management. In our demo, it runs on port 9029 with `admin`-scoped OAuth and A2A protocol via JSONRPC transport.

## Overview

Foodles's hotel guests interact with AI across dozens of touchpoints: the mobile app, the website, in-property kiosks, and guest service channels. Before consolidation, each touchpoint had its own chatbot with its own logic, training data, and governance gaps.

The Guest Agent replaces all of those with a single governed agent. Whether a guest asks about room availability on their phone or requests a late checkout on the website, the same agent handles it -- with the same system prompt, the same security controls, and the same audit trail.

### Key Facts

| Property | Value |
|----------|-------|
| Agent ID | `customer_agent` |
| Foodles Name | Guest Agent |
| Port | 9029 |
| Endpoint | `http://localhost:8080/api/v1/agents/customer_agent` |
| Protocol | A2A v0.3.0, JSONRPC |
| Streaming | Enabled |
| OAuth Scope | `admin` |
| OAuth Audience | `a2a` |
| MCP Servers | None (domain-specific tools added per deployment) |
| Primary Agent | No |

## Configuration

The Guest Agent is defined in `services/agents/customer_agent.yaml`. Here is the full configuration with explanations:

```yaml
# Guest Agent Configuration (Foodles Demo)
# Guest-facing agent for hotel booking and concierge services

agents:
  customer_agent:
    name: customer_agent
    port: 9029
    endpoint: http://localhost:8080/api/v1/agents/customer_agent
    enabled: true
    dev_only: false
    is_primary: false
    default: false
```

### Port Assignment

Port 9029 is dedicated to the Guest Agent. In a production deployment, a load balancer routes guest traffic to one or more instances listening on this port. The port is distinct from other agents (revenue on 9028, distribution on 9027, platform on 9026), enabling independent scaling and monitoring.

### A2A Protocol Card

The agent's card defines how it presents itself to other agents and the platform:

```yaml
    card:
      protocolVersion: 0.3.0
      name: Guest Agent
      displayName: Guest Agent
      description: Guest-facing AI agent handling hotel search, booking assistance,
        stay management, concierge services, and loyalty program management
        across all guest touchpoints
      version: 1.0.0
      preferredTransport: JSONRPC
      iconUrl: https://ui-avatars.com/api/?name=GA&background=0071ce&color=fff
```

The protocol version (0.3.0) indicates compatibility with the current A2A specification, enabling agent-to-agent communication for cross-domain workflows.

### Capabilities

```yaml
      capabilities:
        streaming: true
        pushNotifications: false
        stateTransitionHistory: false
```

- **Streaming**: Enabled. Guests see responses as they are generated, which is critical for perceived responsiveness in booking and concierge scenarios.
- **Push notifications**: Disabled. The agent does not push unsolicited messages to guests.
- **State transition history**: Disabled. Conversation state changes are not exposed to the client.

## System Prompt

The system prompt defines the Guest Agent's behavior, capabilities, and constraints:

```
You are the Guest Agent, a guest-facing AI assistant designed to handle
all hotel guest interactions at enterprise scale. You serve as the primary point
of contact for guests across web, mobile, and in-property digital touchpoints.

## Capabilities

- **Hotel Search & Booking**: Help guests find properties, compare options,
  and complete reservations
- **Stay Management**: Manage reservations, handle room upgrades, late checkout requests
- **Concierge Services**: Provide personalized recommendations for dining,
  activities, and local attractions
- **Guest Support**: Answer questions about properties, policies, and amenities

## Operating Principles

1. **Privacy First**: Never expose guest PII or booking history without
   proper authentication
2. **Accurate Information**: Only provide verified room availability and pricing
3. **Escalation Protocol**: Route complex issues to human agents when confidence
   is below threshold
4. **Compliance**: Follow all consumer protection regulations and company policies

## Sub-Agent Orchestration

This agent orchestrates specialized sub-agents for:
- Property search and availability lookup
- Rate optimization and booking completion
- Stay modification and cancellation processing
- Loyalty program management
```

### Prompt Design Principles

The system prompt follows enterprise patterns:

1. **Role clarity**: The agent knows it is guest-facing and should behave accordingly -- professional, helpful, never sarcastic or dismissive.

2. **Capability boundaries**: Four explicit domains (search, stay management, concierge, support) define what the agent can and cannot do. Anything outside these domains triggers the escalation protocol.

3. **Operating principles as guardrails**: Privacy, accuracy, escalation, and compliance are not suggestions -- they are hard constraints that the agent must follow in every interaction.

4. **Sub-agent delegation**: The prompt acknowledges that complex tasks are delegated to specialized sub-agents. This is how a single agent handles the breadth of guest interactions without becoming a monolithic prompt.

## Skills

The Guest Agent advertises one skill on its A2A card:

### general_assistance

| Property | Value |
|----------|-------|
| Skill ID | `general_assistance` |
| Name | General Assistance |
| Description | Help with questions, explanations, and general tasks |
| Tags | `assistance`, `general` |
| Examples | "Answer guest questions", "Explain property amenities" |

In a production deployment, additional skills would be registered for domain-specific capabilities:

- **guest_experience**: Personalized booking flows, recommendation engines, loyalty management
- **stay_operations**: Reservation tracking, room upgrades, checkout management

Skills are declared on the A2A card so that other agents and the platform know what this agent can do. When the Revenue Agent needs to check a guest's booking status (with proper authorization), it discovers the Guest Agent's `general_assistance` skill through the A2A protocol.

## Security Configuration

### OAuth2 Authentication

```yaml
    oauth:
      required: true
      scopes:
        - admin
      audience: a2a
```

Every request to the Guest Agent must include a valid OAuth2 token with the `admin` scope. The audience is set to `a2a`, which is the shared audience across all agents for mutual authentication.

### Why Admin Scope for a Guest Agent?

This may seem counterintuitive -- why does a guest-facing agent require admin scope? In the enterprise demo configuration, `admin` scope indicates that the agent is accessed through the platform's internal API, not directly by end guests. Guest-facing applications (the mobile app, the website) authenticate through their own OAuth flows and then call the Guest Agent's API with an admin-scoped service token.

In a production deployment, you would typically add a `guest` scope and configure the OAuth flow to issue guest-scoped tokens:

```yaml
      securitySchemes:
        oauth2:
          flows:
            authorizationCode:
              scopes:
                admin: Administrator access
                guest: Guest access
```

### A2A Security

The `a2a` audience enables agent-to-agent communication. When the Distribution Agent needs to notify the Guest Agent about a rate change, it authenticates with an A2A token validated against this audience.

## Operating Principles in Depth

### Privacy First

The Guest Agent never exposes:

- Guest names, addresses, or contact information in logs
- Booking history to unauthenticated requests
- Payment information under any circumstances
- Cross-guest data (one guest's data to another)

Privacy enforcement happens at multiple levels:
1. The system prompt instructs the agent to protect PII
2. OAuth scoping prevents unauthorized access
3. Audit logging captures access attempts for compliance review
4. MCP tool responses are sanitized before reaching the agent

### Accurate Information

The agent only reports room availability and pricing from verified data sources. It does not guess, estimate, or cache stale data. When connected to PMS/RMS MCP tools, the agent queries real-time availability rather than relying on training data.

### Escalation Protocol

When the agent's confidence drops below a configurable threshold, it escalates to a human agent. Escalation scenarios include:

- Billing disputes (always escalate)
- Medical or safety concerns at the property
- Legal questions about cancellation policies or liability
- Repeated failed attempts to resolve an issue

### Compliance

The agent follows consumer protection regulations including:

- Truth in advertising (no misleading property claims)
- Cancellation policy accuracy
- Rate transparency and parity
- Accessibility requirements for digital interactions

## Sub-Agent Orchestration

The Guest Agent delegates specialized tasks to sub-agents:

### Property Search and Availability Lookup

When a guest asks "Do you have ocean-view rooms available next weekend?", the Guest Agent delegates to a property search sub-agent that queries the PMS. The sub-agent handles the complexity of multi-property inventory, room type availability, and real-time rate updates.

### Rate Optimization and Booking Completion

Booking involves rate calculations, promotional offers, and guest preferences. The Guest Agent delegates this to a booking sub-agent that can find the best available rate and complete the reservation.

### Stay Modification and Cancellation Processing

Modification eligibility, cancellation policies, and refund calculations are handled by a dedicated sub-agent. The Guest Agent provides the conversational interface while the sub-agent processes the business logic.

### Loyalty Program Management

Points balance, tier status, and reward redemptions are managed by a loyalty sub-agent. This keeps loyalty program complexity out of the main agent's context window.

## Access Control

### Who Can Interact with This Agent

| Role | Access Level | How |
|------|-------------|-----|
| Admin | Full access | Direct API access, admin dashboard |
| Guest (via app) | Scoped access | Through guest-facing application with service token |
| Other agents | A2A access | Through A2A protocol with mutual authentication |
| Non-admin platform users | No access | Agent not visible unless plugin is assigned to their role |

### Plugin Membership

The Guest Agent belongs to the `enterprise-demo` plugin. Users must have the `admin` role to see this agent in the platform. In a production deployment, you would create a separate plugin for guest-facing access with appropriate role assignments.

