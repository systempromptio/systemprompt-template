---
title: "Governance & Policy"
description: "End-to-end overview of the systemprompt.io governance model: evaluation pipeline, RBAC, secret detection, audit trails, SIEM integration, rate limiting, and tool governance."
author: "systemprompt.io"
slug: "governance-overview"
keywords: "governance, policy, security, RBAC, audit, SIEM, secrets, rate limiting, tool governance, compliance"
kind: "guide"
public: true
tags: ["governance", "security", "policy", "enterprise"]
published_at: "2026-04-01"
updated_at: "2026-04-01"
after_reading_this:
  - "Understand the end-to-end governance model and how every AI tool call is evaluated before execution"
  - "Know how policies are defined through RBAC, department scoping, and allow/deny rules"
  - "Understand how to integrate audit data with your existing SIEM infrastructure"
  - "Know the full audit trail capabilities from identity through to cost tracking"
related_docs:
  - title: "Access Control & RBAC"
    url: "/documentation/access-control"
  - title: "Tool Governance"
    url: "/documentation/tool-governance"
  - title: "Audit Trails & Events"
    url: "/documentation/events"
  - title: "Secrets & Encryption"
    url: "/documentation/secrets"
  - title: "Rate Limiting"
    url: "/documentation/rate-limiting"
  - title: "Hooks & Automation"
    url: "/documentation/hooks"
---

# Governance & Policy

Every AI interaction in the platform passes through a governance layer before anything executes. This page explains the full model -- how policies are defined, how they are enforced, and how every action is recorded. Each section links to a detailed page covering that capability.

## The Governance Pipeline

Every tool call passes through a synchronous four-layer evaluation pipeline. The call is blocked until all layers pass. If any layer rejects, the call is denied and the rejection is logged.

```
Incoming Tool Call
       │
       ▼
┌──────────────┐
│  Scope Check │  Is this user/role/department allowed to call this tool?
└──────┬───────┘
       │ pass
       ▼
┌──────────────┐
│  Secret Scan │  Does the input contain API keys, tokens, or credentials?
└──────┬───────┘
       │ pass
       ▼
┌──────────────┐
│  Blocklist   │  Is this tool, server, or action on the deny list?
└──────┬───────┘
       │ pass
       ▼
┌──────────────┐
│  Rate Limit  │  Has this caller exceeded their request budget?
└──────┬───────┘
       │ pass
       ▼
   Tool Executes
```

Each layer operates independently. A request must pass all four checks in sequence. Failures at any layer return a structured error to the caller and emit an audit event.

## Policy Model

Policies control who can access what. Three mechanisms work together:

**Role-Based Access Control (RBAC)** -- Six role tiers define baseline permissions:

| Tier | Role | Purpose |
|------|------|---------|
| 1 | Admin | Full platform access |
| 2 | User | Standard interactive access |
| 3 | A2A | Agent-to-agent communication |
| 4 | MCP | Tool server service accounts |
| 5 | Service | Internal platform services |
| 6 | Anonymous | Unauthenticated public access |

**Department scoping** constrains access within team boundaries. A user in the "Engineering" department only sees plugins, agents, and MCP servers assigned to Engineering -- even if their role tier would otherwise grant broader access.

**Allow/deny rules** provide per-entity overrides. Each plugin, agent, and MCP server can have explicit allow or deny rules by role and department. Deny rules always take precedence over allow rules.

See [Access Control & RBAC](/documentation/access-control) for dashboard configuration and bulk assignment.

## Secret Detection

The secret scanner inspects every tool call input for credentials before execution. It matches 35+ patterns covering:

- **Cloud providers** -- AWS access keys, Azure tokens, GCP service account keys
- **Code platforms** -- GitHub PATs, GitLab tokens, Bitbucket credentials
- **Communication** -- Slack tokens, Discord webhooks, Twilio keys
- **Payment** -- Stripe secret keys, PayPal credentials
- **Certificates** -- PEM private keys, PKCS8 keys
- **Databases** -- Connection strings with embedded credentials
- **AI providers** -- OpenAI, Anthropic, and Cohere API keys

Detected secrets are never logged in plaintext. Secrets stored by the platform use ChaCha20-Poly1305 authenticated encryption at rest, with per-secret nonces and admin-rotatable encryption keys.

See [Secrets & Encryption](/documentation/secrets) for secret management and key rotation.

## Audit Trails

Every meaningful action in the platform emits a structured event. Sixteen event hooks capture the full lifecycle:

- Session start/end
- Tool call request and result
- Agent assignment and messaging
- Permission grants and denials
- Secret access and rotation
- Plugin installation and removal
- Configuration changes
- Error and rejection events

Each event records a full request trace: **identity** (who) -> **agent** (which AI) -> **permissions** (what was allowed) -> **tool calls** (what executed) -> **result** (what happened) -> **cost** (what it cost in tokens and money).

Events are stored as JSONB for flexible querying. The admin Events dashboard provides filtering by type, user, tool, session, and time range. Events are searchable and exportable.

See [Audit Trails & Events](/documentation/events) for the event browser and filtering interface.

## SIEM Integration

Audit events are emitted as structured JSON, ready for forwarding to your existing security infrastructure -- Splunk, ELK, Datadog, or any system that ingests JSON logs.

Example log entry for a tool call event:

```json
{
  "timestamp": "2026-04-01T14:32:07.891Z",
  "event_type": "tool_call",
  "level": "info",
  "user_id": "usr_8f3a2b",
  "role": "user",
  "department": "engineering",
  "agent": "developer-agent",
  "mcp_server": "github-tools",
  "tool": "create_pull_request",
  "status": "success",
  "duration_ms": 342,
  "tokens_in": 1250,
  "tokens_out": 89,
  "cost_usd": 0.0043,
  "session_id": "ses_7c91ef",
  "request_id": "req_a4b8d2"
}
```

Three integration paths are available:

- **Log forwarding** -- Structured JSON written to stdout/file for collection by your log agent (Fluentd, Filebeat, Vector)
- **Real-time streaming** -- SSE endpoint for live event consumption
- **CLI queries** -- `systemprompt infra logs view` and `systemprompt analytics` commands for ad-hoc investigation

See [Analytics & Observability](/features/analytics-and-observability) for the full observability feature set.

## Rate Limiting

Rate limits prevent abuse and ensure fair resource allocation. Limits are tiered by role with a token bucket algorithm:

| Role | Multiplier | Effect |
|------|-----------|--------|
| Admin | 10x | Highest throughput for platform operators |
| A2A / MCP | 5x | Elevated limits for service-to-service workloads |
| User | 1x | Baseline rate for interactive users |
| Anonymous | 0.5x | Restricted access for unauthenticated callers |

Each tier applies a burst multiplier allowing short spikes above the sustained rate. When a caller exceeds their budget, requests return a 429 status with a `Retry-After` header.

Rate limit rejections are logged as audit events and count toward anomaly detection.

See [Rate Limiting](/documentation/rate-limiting) for per-endpoint configuration and tuning guidance.

## Tool Governance

MCP tool access is governed at the protocol level. Each tool call is subject to:

- **OAuth scope validation** -- The calling agent must hold a valid scope grant for the target MCP server and tool
- **Execution logging** -- Every tool invocation is recorded with input parameters, output, duration, and cost
- **Event hooks** -- Tool calls can trigger hooks for notifications, compliance checks, or automated responses

Tool governance ensures that even when an agent has access to an MCP server, individual tools within that server can be independently allowed or denied.

See [Tool Governance](/documentation/tool-governance) for per-tool access control configuration and CLI commands.
