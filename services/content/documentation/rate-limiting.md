---
title: "Rate Limiting & Compliance"
description: "Tiered rate limiting controls with per-endpoint and per-role granularity. Defense-in-depth approach combining rate limits, token expiry, and scope validation."
author: "systemprompt.io"
slug: "rate-limiting"
keywords: "rate limiting, compliance, security, throttling, tiers"
kind: "guide"
public: true
tags: ["security", "rate-limiting", "compliance"]
published_at: "2026-03-19"
updated_at: "2026-03-19"
after_reading_this:
  - "Understand the tiered rate limiting model and how multipliers apply"
  - "Configure per-endpoint rate limits for your traffic pattern"
  - "Explain the defense-in-depth approach: rate limits, token expiry, scope validation"
  - "Tune rate limits for agent-to-agent and MCP workloads at scale"
related_docs:
  - title: "Platform Architecture"
    url: "/documentation/architecture"
  - title: "Scaling Architecture"
    url: "/documentation/scaling"
  - title: "Access Control"
    url: "/documentation/access-control"
  - title: "Authentication"
    url: "/documentation/authentication"
  - title: "Secrets"
    url: "/documentation/secrets"
---

# Rate Limiting & Compliance

**TL;DR:** The platform uses a defense-in-depth approach combining tiered rate limiting, JWT token expiry, OAuth scope validation, and secret resolution tokens. Rate limits apply per-tier (admin, user, a2a, mcp, service, anon) and per-endpoint, with a 3x burst multiplier for traffic spikes. Every layer is independently configurable from the production profile.

## Defense-in-Depth Overview

Rate limiting is one layer in a multi-layer security model. No single layer is sufficient — they work together:

```
┌─────────────────────────────────────────────┐
│            Rate Limiting                     │
│  Per-tier multipliers + per-endpoint limits  │
├─────────────────────────────────────────────┤
│            Token Expiry                      │
│  Access: 30 days, Refresh: 180 days          │
├─────────────────────────────────────────────┤
│            OAuth Scope Validation            │
│  Per-server, per-agent scope enforcement     │
├─────────────────────────────────────────────┤
│            Secret Resolution Tokens          │
│  Short-lived, single-use                     │
├─────────────────────────────────────────────┤
│            RBAC + Department Scoping         │
│  Role and department access rules            │
└─────────────────────────────────────────────┘
```

Each layer stops a different class of abuse. Rate limiting stops volume attacks. Token expiry limits the window of a compromised credential. Scope validation prevents privilege escalation. Secret resolution tokens prevent secret exfiltration. RBAC prevents unauthorized access.

## Rate Limit Tiers

Every request is classified into a tier based on the authentication context. The tier determines a multiplier applied to base rate limits.

| Tier | Multiplier | Who | Rationale |
|------|-----------|-----|-----------|
| **Admin** | 10.0x | Platform administrators | Full throughput for management, bulk operations, and debugging |
| **User** | 1.0x | Standard authenticated users | Baseline rate — normal interactive usage |
| **A2A** | 5.0x | Agent-to-agent communication | Elevated for orchestration — agents calling sub-agents generate burst traffic |
| **MCP** | 5.0x | MCP server tool calls | Elevated for tool-heavy workflows — a single user action may trigger many tool calls |
| **Service** | 5.0x | Internal service communication | Trusted internal traffic between platform components |
| **Anon** | 0.5x | Unauthenticated requests | Most restricted — public endpoints only, abuse prevention |

### How Tiers Are Determined

The platform determines the tier from the JWT token's audience claim:

| JWT Audience | Tier |
|-------------|------|
| `web` or `api` | User (or Admin if the user has admin role) |
| `a2a` | A2A |
| `mcp` | MCP |
| No token | Anon |

Service tier is assigned to requests from internal platform components using service credentials.

### Tier Multiplier Example

For the MCP endpoint with a base rate of 200 requests/second:

| Tier | Calculation | Effective Rate |
|------|------------|---------------:|
| Admin | 200 x 10.0 | **2,000 req/s** |
| User | 200 x 1.0 | **200 req/s** |
| A2A | 200 x 5.0 | **1,000 req/s** |
| MCP | 200 x 5.0 | **1,000 req/s** |
| Service | 200 x 5.0 | **1,000 req/s** |
| Anon | 200 x 0.5 | **100 req/s** |

## Per-Endpoint Rate Limits

Each API endpoint category has its own base rate limit. These are the production values:

| Endpoint | Base Rate (req/s) | Purpose |
|----------|------------------:|---------|
| **OAuth (public)** | 10 | Token issuance, authorization — low rate to prevent brute force |
| **OAuth (authenticated)** | 10 | Token refresh, revocation |
| **Contexts** | 100 | Conversation context management — high rate for active conversations |
| **Tasks** | 50 | Task creation and management |
| **Artifacts** | 50 | Artifact upload and retrieval |
| **Agent Registry** | 50 | Agent discovery and registration |
| **Agents** | 20 | Agent interaction — lower rate because each request may trigger AI inference |
| **MCP Registry** | 50 | MCP server discovery and registration |
| **MCP** | 200 | MCP tool calls — highest rate because tool-heavy workflows make many calls |
| **Stream** | 100 | Streaming AI responses — high rate for real-time interaction |
| **Content** | 50 | Content and documentation serving |

### Why MCP Has the Highest Rate

A single user action — like asking an agent to check platform status — can generate multiple MCP tool calls:

1. User asks: "What is the current platform status?"
2. Agent calls service status tool (1 MCP call)
3. Agent calls agent list tool (1 MCP call)
4. Agent calls log check tool (1 MCP call)
5. Agent calls analytics tool (1 MCP call)

Five MCP calls from one user message. At scale with thousands of concurrent users, the MCP endpoint needs the highest base rate.

## Burst Multiplier

The burst multiplier of **3x** allows temporary traffic spikes above the sustained rate. This uses a token bucket algorithm.

### How Token Bucket Works

| Parameter | Value |
|-----------|-------|
| **Fill rate** | Base rate x tier multiplier (sustained rate) |
| **Bucket size** | Sustained rate x burst multiplier (3x) |
| **Burst capacity** | 3x the sustained rate |

**Example:** A user-tier request to the MCP endpoint:
- Sustained rate: 200 req/s x 1.0 (user) = 200 req/s
- Bucket size: 200 x 3 = 600 tokens
- The user can burst to 600 req/s momentarily, but sustained traffic above 200 req/s will be throttled

### Why 3x

The 3x multiplier handles legitimate traffic patterns:

| Pattern | Why It Bursts |
|---------|--------------|
| **Page load** | Browser makes 10-20 parallel requests for resources |
| **Agent orchestration** | Agent calls multiple sub-agents simultaneously |
| **Dashboard refresh** | Admin dashboard makes 5-10 API calls to populate widgets |
| **Bulk operations** | Admin applies access rules to 20 entities at once |

A multiplier below 2x drops legitimate bursts. Above 5x allows sustained abuse before throttling. 3x is the empirical balance.

## JWT Token Expiry

Token expiry limits the damage window of a compromised credential.

| Token Type | Expiry | Purpose |
|-----------|--------|---------|
| **Access token** | 30 days (2,592,000 seconds) | Primary authentication for API requests |
| **Refresh token** | 180 days (15,552,000 seconds) | Obtain new access tokens without re-authentication |

### Token Audiences

JWT tokens are scoped to specific audiences. A token issued for one audience cannot be used for another:

| Audience | Used For |
|----------|----------|
| **web** | Browser-based dashboard access |
| **api** | Programmatic API access |
| **a2a** | Agent-to-agent communication |
| **mcp** | MCP server tool calls |

### Token Lifecycle

1. User authenticates via OAuth → receives access token + refresh token
2. Access token is sent with every request in the Authorization header
3. When access token expires, client uses refresh token to obtain a new access token
4. When refresh token expires, user must re-authenticate

## Secret Resolution Tokens

Secrets (API keys, database credentials, encryption keys) are never exposed directly. Instead, the platform issues **secret resolution tokens** — short-lived, single-use tokens that resolve to the secret value at point of use.

| Property | Value |
|----------|-------|
| **Lifetime** | Short-lived (minutes, not days) |
| **Usage** | Single-use — consumed on resolution |
| **Audit** | Every resolution is logged with actor, timestamp, and source |
| **Encryption** | Secrets encrypted at rest with ChaCha20-Poly1305 AEAD |

This means even if a resolution token is intercepted, it can only be used once and expires quickly.

## OAuth Scope Validation

Each MCP server has its own OAuth configuration with specific scopes. Agents must be authorized for the specific scopes required by the MCP server they are calling.

| Validation | What It Checks |
|-----------|----------------|
| **Server scopes** | Does the MCP server require specific OAuth scopes? |
| **Agent authorization** | Is the agent authorized for those scopes? |
| **Token scopes** | Does the JWT token carry the required scopes? |
| **Audience match** | Does the token audience match the expected audience? |

Scope validation prevents an agent authorized for "product search" from calling "returns processing" — even if both tools are on the same MCP server.

## Rate Limits at Scale

At enterprise scale, rate limiting is critical. The platform scales horizontally, and rate limits apply **per instance** -- so aggregate capacity grows linearly with the number of instances behind a load balancer.

### Capacity Scaling

As your deployment grows, add application instances to increase aggregate throughput. The per-instance rate limits remain constant; total capacity is the per-instance limit multiplied by the number of instances. This linear scaling model makes capacity planning straightforward.

### Rate Limit Headers

When a request is rate-limited, the platform returns:

| Header | Value |
|--------|-------|
| **HTTP Status** | `429 Too Many Requests` |
| **Retry-After** | Seconds until the next request will be accepted |

Clients should implement exponential backoff when receiving 429 responses.

## Configuration

Rate limits are configured in the production profile YAML. Here is the structure:

```yaml
rate_limits:
  disabled: false
  oauth_public_per_second: 10
  oauth_auth_per_second: 10
  contexts_per_second: 100
  tasks_per_second: 50
  artifacts_per_second: 50
  agent_registry_per_second: 50
  agents_per_second: 20
  mcp_registry_per_second: 50
  mcp_per_second: 200
  stream_per_second: 100
  content_per_second: 50
  burst_multiplier: 3
  tier_multipliers:
    admin: 10.0
    user: 1.0
    a2a: 5.0
    mcp: 5.0
    service: 5.0
    anon: 0.5
```

### Disabling Rate Limits

For development and testing, rate limits can be disabled entirely:

```yaml
rate_limits:
  disabled: true
```

This should never be done in production. The local development profile has rate limits disabled by default.

### Tuning for Your Traffic Pattern

If your deployment has different traffic characteristics, adjust the per-endpoint rates:

| If You See | Adjust |
|-----------|--------|
| 429 errors on MCP endpoints | Increase `mcp_per_second` or the MCP tier multiplier |
| 429 errors on stream endpoints | Increase `stream_per_second` — you have many concurrent AI conversations |
| 429 errors on agent endpoints | Increase `agents_per_second` — but consider whether this indicates a retry storm |
| 429 errors on OAuth endpoints | Do NOT increase — this may indicate a brute force attack. Check logs. |

## Monitoring Rate Limits

Use the CLI to monitor rate limit events:

```bash
# Check for rate limit warnings
systemprompt infra logs view --level warn --since 1h

# View recent requests to identify patterns
systemprompt infra logs request list -n 20

# Full audit of a specific request
systemprompt infra logs audit <request-id> --full
```

Rate limit events are logged at the `warn` level. If you see sustained 429 errors, investigate whether the traffic is legitimate (increase limits) or abusive (keep limits, investigate source).
