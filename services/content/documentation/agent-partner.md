---
title: "Distribution Agent"
description: "Configuration and governance of the partner_agent for OTA management, channel distribution, rate parity, and partner relationship management."
author: "systemprompt.io"
slug: "agent-partner"
keywords: "distribution agent, OTA, channel management, rate parity, distribution"
kind: "guide"
public: false
tags: ["agents", "distribution", "enterprise", "channel-management"]
published_at: "2026-03-19"
updated_at: "2026-03-19"
after_reading_this:
  - "Understand the Distribution Agent's role in managing channel partner relationships"
  - "Read the agent's YAML configuration and data confidentiality model"
  - "Identify the strict data isolation between channel partners"
  - "Configure compliance verification and audit logging for partner interactions"
related_docs:
  - title: "Agents"
    url: "/documentation/agents"
  - title: "Secrets"
    url: "/documentation/secrets"
---

# Distribution Agent

**TL;DR:** The Distribution Agent (`partner_agent`) demonstrates a channel-partner-facing agent for hospitality distribution. It manages OTA and channel partner interactions -- onboarding, rate distribution, performance monitoring, parity enforcement, and marketplace operations. In our demo, it runs on port 9027 with `admin`-scoped OAuth and enforces strict data confidentiality between channel partners.

## Overview

The platform works with hundreds of OTAs and channel partners across every distribution channel. Each partner interaction involves sensitive data -- commission rates, contract terms, performance metrics, booking volumes, and compliance status. Before the Distribution Agent, these interactions were scattered across email, API dashboards, partner portals, and spreadsheets.

The Distribution Agent consolidates all partner-facing interactions into a single governed interface. A channel partner can check their booking performance, review rate parity requirements, and update their integration configuration -- all through one agent that enforces data isolation, audit logging, and regulatory compliance.

The defining characteristic of this agent is **data confidentiality**. The Distribution Agent handles data from competing channel partners simultaneously. OTA A must never see OTA B's commission rates, performance metrics, or contract terms. This is not just a feature -- it is a legal and regulatory requirement.

### Key Facts

| Property | Value |
|----------|-------|
| Agent ID | `partner_agent` |
| Foodles Name | Distribution Agent |
| Port | 9027 |
| Endpoint | `http://localhost:8080/api/v1/agents/partner_agent` |
| Protocol | A2A v0.3.0, JSONRPC |
| Streaming | Enabled |
| OAuth Scope | `admin` |
| OAuth Audience | `a2a` |
| MCP Servers | None (OTA integration added per channel partner) |
| Primary Agent | No |

## Configuration

The Distribution Agent is defined in `services/agents/partner_agent.yaml`:

```yaml
# Distribution Agent Configuration (Foodles Demo)
# Channel-partner-facing agent for OTA and distribution management

agents:
  partner_agent:
    name: partner_agent
    port: 9027
    endpoint: http://localhost:8080/api/v1/agents/partner_agent
    enabled: true
    dev_only: false
    is_primary: false
    default: false
```

### Port Assignment

Port 9027 hosts the Distribution Agent. Channel partner traffic is lower in volume than guest or revenue traffic but involves longer, more data-intensive sessions. Partner interactions often involve bulk data queries (booking manifests, rate distribution updates, compliance reports) that require sustained connections.

### A2A Protocol Card

```yaml
    card:
      protocolVersion: 0.3.0
      name: Distribution Agent
      displayName: Distribution Agent
      description: Channel partner and OTA-facing AI agent managing distribution
        relationships, rate parity, and marketplace operations
      version: 1.0.0
      preferredTransport: JSONRPC
      iconUrl: https://ui-avatars.com/api/?name=DA&background=76c043&color=fff
```

The icon uses a distribution green (`#76c043`), distinct from the platform agent's darker green. The A2A card enables other agents to discover the Distribution Agent's capabilities for cross-agent workflows -- for example, when the Guest Agent needs to check a rate discrepancy reported by a guest.

### Capabilities

```yaml
      capabilities:
        streaming: true
        pushNotifications: false
        stateTransitionHistory: false
```

Streaming is enabled for real-time feedback during data-heavy operations like rate distribution queries and compliance report generation.

### Skills on the A2A Card

```yaml
      skills:
      - id: general_assistance
        name: General Assistance
        description: Help with questions, explanations, and general tasks
        tags:
        - assistance
        - general
        examples:
        - Check channel partner compliance status
        - Review partner booking performance metrics
        - Track rate distribution status for this week
```

The examples reflect typical channel partner interactions -- compliance checks, performance reviews, and rate distribution tracking.

## System Prompt

```
You are the Distribution Agent, an AI assistant managing all OTA and channel
partner interactions. You handle distribution relationships, rate parity
coordination, and marketplace operations at enterprise scale.

## Capabilities

- **Partner Onboarding**: Guide new channel partners through compliance and integration
- **Performance Monitoring**: Track partner SLAs, booking metrics, and
  conversion performance
- **Rate Distribution**: Real-time rate push monitoring, parity enforcement,
  channel optimization
- **Marketplace Management**: Rate plan distribution, promotional coordination,
  availability management

## Operating Principles

1. **Data Confidentiality**: Never share one channel partner's data with another
2. **Compliance First**: Verify regulatory compliance before processing
   any partner action
3. **Audit Trail**: Log all partner communications and data access
4. **Fair Practices**: Apply consistent rules across all partner interactions

## Sub-Agent Orchestration

This agent orchestrates specialized sub-agents for:
- Partner compliance verification
- Rate distribution tracking and optimization
- Demand forecasting and availability planning
- Contract and commission management
```

### Data Confidentiality as the Top Priority

The Distribution Agent's operating principles lead with data confidentiality -- not security, not compliance, but data isolation. This is because:

1. **Legal exposure**: Sharing OTA A's commission rates with OTA B could constitute a violation of competition law
2. **Competitive intelligence**: Partners' commission rates, booking volumes, and performance metrics are trade secrets
3. **Contractual obligations**: Most distribution agreements include non-disclosure clauses
4. **Trust**: The entire partner relationship depends on confident data isolation

Every response, every tool call, and every log entry must be filtered through the lens of "Does this expose one partner's data to another?"

## Skills in Depth

### general_assistance (Distribution Context)

While the skill ID is `general_assistance` (shared across agents), the Distribution Agent's implementation is partner-specific:

- **"Check channel partner compliance status"** -- Queries the compliance verification system for a specific partner's current status, outstanding requirements, and upcoming deadlines.
- **"Review partner booking performance metrics"** -- Retrieves SLA adherence, conversion rates, booking volumes, and cancellation rates for the authenticated partner.
- **"Track rate distribution status for this week"** -- Shows rate push status, parity alerts, and distribution confirmations across channels.

In a full deployment, additional skills would include:

### partner_management

Covers the full partner lifecycle:

- **Onboarding**: Step-by-step guidance for new channel partner integration, including API configuration, rate mapping, and test bookings
- **Compliance verification**: Automated checks against regulatory requirements, certifications, and contractual obligations
- **Performance scorecards**: Aggregated metrics across booking volume, conversion, cancellation rates, and revenue contribution
- **Relationship management**: Communication history, escalation tracking, and quarterly business review preparation

### distribution_ops

Covers operational distribution workflows:

- **Rate distribution**: Real-time visibility into rate pushes from PMS to all connected channels
- **Parity monitoring**: Automated rate parity checking across all distribution channels
- **Availability optimization**: Collaborative availability planning between Foodles and channel partners
- **Booking management**: Reservation flow tracking, modification handling, and cancellation processing

## Security Configuration

```yaml
    oauth:
      required: true
      scopes:
        - admin
      audience: a2a
```

### Admin Scope for Partner Access

The Distribution Agent requires `admin` scope because:

- Channel partner data is highly sensitive (commission rates, contracts, performance metrics)
- Partner interactions have legal and regulatory implications
- The agent may initiate financial operations (commission adjustments, rate overrides)
- Cross-partner data isolation requires strict access controls

### Partner Identity and Data Isolation

In a production deployment, the OAuth token includes partner-specific claims:

| Claim | Purpose |
|-------|---------|
| `partner_id` | Identifies which channel partner the user represents |
| `partner_tier` | Determines the level of data access (strategic, preferred, standard) |
| `data_scope` | Limits which markets and property types the partner can query |
| `region` | Geographic scope for regional partners |

Every MCP tool call passes the partner identity from the token. Tools filter all data through the partner lens -- a partner can only see their own data, never another partner's.

### Data Isolation Implementation

Data isolation is enforced at multiple levels:

1. **Token level**: The OAuth token identifies the partner, and the platform rejects requests without partner identity
2. **Tool level**: MCP tools accept a partner ID parameter and filter all queries accordingly
3. **Response level**: The agent's responses are scanned for cross-partner data leaks before delivery
4. **Audit level**: All data access is logged with the partner context for compliance review

## Operating Principles in Depth

### Data Confidentiality

The Distribution Agent enforces strict data isolation:

| Data Type | Visibility Rule |
|-----------|----------------|
| Commission rates | Visible only to the partner who owns the distribution agreement |
| Performance metrics | Visible to the partner and to Foodles revenue management |
| Booking details | Visible to the partner, the property, and Foodles operations |
| Contract terms | Visible only to the partner and authorized Foodles staff |
| Rate data | Visible to the partner; aggregated anonymously for market trends |
| Compliance status | Visible to the partner and Foodles compliance team |

The agent never:

- Compares one partner's performance to another in a response
- Reveals another partner's commission rates or terms
- Suggests a partner match another partner's offer
- Aggregates data in ways that could identify individual partners

### Compliance First

Before processing any partner action, the agent verifies:

1. **Regulatory compliance**: Is the partner current on all required certifications?
2. **Integration compliance**: Does the partner maintain proper API connections?
3. **Contractual compliance**: Is the action permitted under the current agreement?
4. **Rate parity compliance**: Does the transaction comply with rate parity requirements?

If any compliance check fails, the agent blocks the action and explains why:

```
I cannot process this rate update because your API integration certificate
expired on March 15, 2026. Please renew your integration credentials,
and I will reprocess the rate distribution once compliance is verified.
```

### Audit Trail

Every partner interaction generates audit records:

| Event | What is Logged |
|-------|---------------|
| Data access | Which data was accessed, by which partner, when |
| Compliance check | Check performed, result, any blocking conditions |
| Rate operations | Rate push, modification, parity check with full details |
| Communication | All messages exchanged, including agent responses |
| Escalation | When and why an interaction was escalated to a human |

### Fair Practices

The agent applies consistent rules to all partners:

- Same compliance verification process regardless of partner size
- Same SLA thresholds for all partners in the same tier
- Consistent communication templates and response formats
- No preferential treatment in queue ordering or response priority

## Sub-Agent Orchestration

### Compliance Verification Sub-Agent

Handles the complexity of multi-jurisdictional compliance:

- Regional regulatory requirements for hospitality distribution
- Industry-specific certifications (PCI DSS for payment data, GDPR for EU guests)
- Integration compliance (API health, connection stability)
- Data protection compliance (guest data handling, privacy regulations)
- Rate parity compliance (contractual rate obligations)

### Rate Distribution Tracking Sub-Agent

Manages rate visibility across distribution channels:

- Multi-channel rate push monitoring
- Parity enforcement across OTAs and direct channels
- Rate override and exception management
- Distribution performance analytics
- Exception management (rate failures, parity violations)

### Demand Forecasting Sub-Agent

Provides AI-powered demand predictions:

- Historical booking data analysis (seasonality, trends, anomalies)
- External factor integration (events, weather, competitor actions)
- Promotional impact modeling (how planned promotions affect demand)
- Lead time optimization (when to adjust rates for maximum revenue)
- Availability buffer recommendations (overbooking thresholds)

### Contract Management Sub-Agent

Handles the commercial relationship:

- Distribution agreement creation and negotiation support
- Commission structure monitoring
- Rate plan agreement enforcement
- Volume commitment tracking
- Renewal and renegotiation scheduling

## Compliance Features

### Regulatory Verification Before Processing

The Distribution Agent implements a "verify before act" pattern:

```
1. Partner requests an action (e.g., update rate distribution)
2. Agent queries compliance status for the partner
3. If compliant -> proceed with the action
4. If non-compliant -> block the action, explain the gap, provide remediation steps
5. Log the compliance check result regardless of outcome
```

This pattern ensures that no partner action proceeds without a current compliance status. It is especially critical for:

- **Payment compliance**: PCI DSS compliance must be verified before processing payment-related data
- **Data protection**: GDPR/CCPA compliance before accessing guest data
- **Rate distribution**: Contractual compliance before rate push operations
- **Integration health**: API connectivity compliance before bulk operations

### Audit Logging of All Communications

Every message to and from the Distribution Agent is logged with:

- Timestamp (UTC)
- Partner identity
- Message content (with PII redaction where applicable)
- Agent response
- Any tools invoked and their results
- Compliance check outcomes
- Session context (conversation ID, session ID)

This audit trail supports:

- **SOC2 compliance**: Demonstrating controls over partner data access
- **Regulatory audits**: Providing evidence of compliance verification
- **Dispute resolution**: Complete record of partner communications
- **Internal audits**: Verifying fair practices across all partners

