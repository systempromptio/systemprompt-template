---
title: "Rate Limits Configuration"
description: "Configure API request throttling with per-endpoint limits and tier multipliers."
author: "SystemPrompt"
slug: "config-rate-limits"
keywords: "rate-limits, throttling, api, burst, tiers, security"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Rate Limits Configuration

Configure API request throttling with per-endpoint limits and tier multipliers.

> **Help**: `{ "command": "admin config rate-limits show" }` via `systemprompt_help`
> **Requires**: Profile configured -> See [Profiles Playbook](../profiles/index.md)

RateLimitsConfig controls API request throttling with per-endpoint limits and user tier multipliers.

---

## RateLimitsConfig Struct

**Source**: `crates/shared/models/src/profile/rate_limits.rs:58-100`

```rust
pub struct RateLimitsConfig {
    #[serde(default)]
    pub disabled: bool,                        // Default: false

    // Per-endpoint limits (requests per second)
    #[serde(default = "default_oauth_public")]
    pub oauth_public_per_second: u64,          // Default: 10
    #[serde(default = "default_oauth_auth")]
    pub oauth_auth_per_second: u64,            // Default: 10
    #[serde(default = "default_contexts")]
    pub contexts_per_second: u64,              // Default: 100
    #[serde(default = "default_tasks")]
    pub tasks_per_second: u64,                 // Default: 50
    #[serde(default = "default_artifacts")]
    pub artifacts_per_second: u64,             // Default: 50
    #[serde(default = "default_agent_registry")]
    pub agent_registry_per_second: u64,        // Default: 50
    #[serde(default = "default_agents")]
    pub agents_per_second: u64,                // Default: 20
    #[serde(default = "default_mcp_registry")]
    pub mcp_registry_per_second: u64,          // Default: 50
    #[serde(default = "default_mcp")]
    pub mcp_per_second: u64,                   // Default: 200
    #[serde(default = "default_stream")]
    pub stream_per_second: u64,                // Default: 100
    #[serde(default = "default_content")]
    pub content_per_second: u64,               // Default: 50

    // Burst and tier configuration
    #[serde(default = "default_burst")]
    pub burst_multiplier: u64,                 // Default: 3
    #[serde(default)]
    pub tier_multipliers: TierMultipliers,     // Default: TierMultipliers::default()
}
```

---

## Default Values

| Endpoint | Default (req/sec) | Purpose |
|----------|-------------------|---------|
| `oauth_public_per_second` | 10 | Public OAuth endpoints |
| `oauth_auth_per_second` | 10 | Authenticated OAuth |
| `contexts_per_second` | 100 | Context operations |
| `tasks_per_second` | 50 | Task management |
| `artifacts_per_second` | 50 | Artifact operations |
| `agent_registry_per_second` | 50 | Agent registry |
| `agents_per_second` | 20 | Agent operations |
| `mcp_registry_per_second` | 50 | MCP server registry |
| `mcp_per_second` | 200 | MCP protocol |
| `stream_per_second` | 100 | Streaming endpoints |
| `content_per_second` | 50 | Content operations |
| `burst_multiplier` | 3 | Burst allowance |

---

## TierMultipliers Struct

**Source**: `crates/shared/models/src/profile/rate_limits.rs`

```rust
pub struct TierMultipliers {
    #[serde(default = "default_admin_multiplier")]
    pub admin: f64,                            // Default: 10.0
    #[serde(default = "default_user_multiplier")]
    pub user: f64,                             // Default: 1.0
    #[serde(default = "default_a2a_multiplier")]
    pub a2a: f64,                              // Default: 5.0
    #[serde(default = "default_mcp_multiplier")]
    pub mcp: f64,                              // Default: 5.0
    #[serde(default = "default_service_multiplier")]
    pub service: f64,                          // Default: 5.0
    #[serde(default = "default_anon_multiplier")]
    pub anon: f64,                             // Default: 0.5
}
```

### Tier Multiplier Defaults

| Tier | Multiplier | Effective Rate |
|------|------------|----------------|
| `admin` | 10.0x | 10x base rate |
| `user` | 1.0x | Base rate |
| `a2a` | 5.0x | 5x base rate |
| `mcp` | 5.0x | 5x base rate |
| `service` | 5.0x | 5x base rate |
| `anon` | 0.5x | Half base rate |

### Effective Rate Calculation

```
Effective Rate = Base Rate × Tier Multiplier × Burst Multiplier (for bursts)
```

**Example**: Admin user on contexts endpoint:
- Base: 100 req/sec
- Multiplier: 10.0x
- Effective: 1000 req/sec
- Burst: 1000 × 3 = 3000 req/sec (short bursts)

---

## Configuration Examples

### Development (Disabled)

```yaml
rate_limits:
  disabled: true
```

### Production (Default)

```yaml
rate_limits:
  disabled: false
  # Uses all defaults
```

### Production (Custom)

```yaml
rate_limits:
  disabled: false
  oauth_public_per_second: 5
  oauth_auth_per_second: 20
  contexts_per_second: 200
  tasks_per_second: 100
  artifacts_per_second: 100
  agent_registry_per_second: 50
  agents_per_second: 50
  mcp_registry_per_second: 50
  mcp_per_second: 500
  stream_per_second: 200
  content_per_second: 100
  burst_multiplier: 5
  tier_multipliers:
    admin: 20.0
    user: 1.0
    a2a: 10.0
    mcp: 10.0
    service: 10.0
    anon: 0.25
```

### High-Traffic API

```yaml
rate_limits:
  disabled: false
  contexts_per_second: 500
  tasks_per_second: 200
  mcp_per_second: 1000
  stream_per_second: 500
  burst_multiplier: 10
```

---

## Validation Rules

**Source**: `crates/shared/models/src/profile/validation.rs:168-200`

When `disabled: false`:

```rust
fn validate_rate_limits(&self) -> Result<()> {
    if self.rate_limits.disabled {
        return Ok(());
    }

    if self.rate_limits.burst_multiplier == 0 {
        return Err(ProfileError::InvalidRateLimit("burst_multiplier must be > 0"));
    }

    let limits_to_check = [
        ("oauth_public_per_second", self.rate_limits.oauth_public_per_second),
        ("oauth_auth_per_second", self.rate_limits.oauth_auth_per_second),
        ("contexts_per_second", self.rate_limits.contexts_per_second),
        ("tasks_per_second", self.rate_limits.tasks_per_second),
        ("artifacts_per_second", self.rate_limits.artifacts_per_second),
        ("agents_per_second", self.rate_limits.agents_per_second),
        ("mcp_per_second", self.rate_limits.mcp_per_second),
        ("stream_per_second", self.rate_limits.stream_per_second),
        ("content_per_second", self.rate_limits.content_per_second),
    ];

    for (name, value) in limits_to_check {
        if value == 0 {
            return Err(ProfileError::InvalidRateLimit(
                format!("{} must be > 0", name)
            ));
        }
    }

    Ok(())
}
```

### Validation Summary

| Condition | Rule |
|-----------|------|
| `disabled: true` | Skip all validation |
| `burst_multiplier` | Must be > 0 |
| All rate limits | Must be > 0 |
| Tier multipliers | No validation (can be 0 for blocking) |

---

## Endpoint Categories

### OAuth Endpoints

| Setting | Endpoints |
|---------|-----------|
| `oauth_public_per_second` | `/auth/login`, `/auth/register`, `/auth/oauth/*` |
| `oauth_auth_per_second` | `/auth/refresh`, `/auth/logout`, `/auth/me` |

### Core API Endpoints

| Setting | Endpoints |
|---------|-----------|
| `contexts_per_second` | `/api/v1/contexts/*` |
| `tasks_per_second` | `/api/v1/tasks/*` |
| `artifacts_per_second` | `/api/v1/artifacts/*` |

### Agent Endpoints

| Setting | Endpoints |
|---------|-----------|
| `agent_registry_per_second` | `/api/v1/agents` (list, registry) |
| `agents_per_second` | `/api/v1/agents/{id}/*` |

### MCP Endpoints

| Setting | Endpoints |
|---------|-----------|
| `mcp_registry_per_second` | `/api/v1/mcp/servers` (list) |
| `mcp_per_second` | `/api/v1/mcp/*`, `/mcp/*` |

### Content Endpoints

| Setting | Endpoints |
|---------|-----------|
| `stream_per_second` | `/api/v1/stream/*`, SSE endpoints |
| `content_per_second` | `/api/v1/content/*`, static content |

---

## Burst Handling

The `burst_multiplier` allows temporary spikes above the base rate.

### How Bursts Work

```
Token Bucket Algorithm:
- Bucket capacity = base_rate × burst_multiplier
- Tokens added at base_rate per second
- Each request consumes 1 token
- Requests blocked when bucket empty
```

### Example

With `contexts_per_second: 100` and `burst_multiplier: 3`:
- Steady rate: 100 req/sec
- Burst capacity: 300 requests
- Can handle short bursts of 300 requests
- Refills at 100 tokens/sec

---

## User Tier Detection

Rate limits are applied based on user tier detected from authentication:

| Auth Type | Tier |
|-----------|------|
| Admin token | `admin` |
| User token | `user` |
| A2A token | `a2a` |
| MCP token | `mcp` |
| Service token | `service` |
| No auth | `anon` |

---

## Disabling Rate Limits

For development, disable all rate limits:

```yaml
rate_limits:
  disabled: true
```

**Warning**: Never disable in production. This exposes your API to:
- Denial of service attacks
- Resource exhaustion
- Abuse by bad actors

---

## Troubleshooting

**"Rate limit exceeded"** (HTTP 429)
- Wait and retry with exponential backoff
- Check if using correct tier (admin vs user)
- Increase limits for specific endpoints

**"Validation failed: must be > 0"**
- All rate limits must be positive when enabled
- Either set values or use `disabled: true`

**"Burst too high"**
- Reduce `burst_multiplier`
- Consider memory implications of large token buckets

**"Anonymous users blocked"**
- `anon` multiplier defaults to 0.5x
- Set to 0 to completely block anonymous
- Set higher for public APIs

---

## Quick Reference

### Disable for Development

```yaml
rate_limits:
  disabled: true
```

### Sensible Production Defaults

```yaml
rate_limits:
  disabled: false
  # All defaults apply
```

### High-Security Production

```yaml
rate_limits:
  disabled: false
  oauth_public_per_second: 5
  oauth_auth_per_second: 10
  burst_multiplier: 2
  tier_multipliers:
    admin: 10.0
    user: 1.0
    anon: 0.1
```

### Default Values Table

| Setting | Default |
|---------|---------|
| `disabled` | `false` |
| `oauth_public_per_second` | 10 |
| `oauth_auth_per_second` | 10 |
| `contexts_per_second` | 100 |
| `tasks_per_second` | 50 |
| `artifacts_per_second` | 50 |
| `agent_registry_per_second` | 50 |
| `agents_per_second` | 20 |
| `mcp_registry_per_second` | 50 |
| `mcp_per_second` | 200 |
| `stream_per_second` | 100 |
| `content_per_second` | 50 |
| `burst_multiplier` | 3 |
| `tier_multipliers.admin` | 10.0 |
| `tier_multipliers.user` | 1.0 |
| `tier_multipliers.a2a` | 5.0 |
| `tier_multipliers.mcp` | 5.0 |
| `tier_multipliers.service` | 5.0 |
| `tier_multipliers.anon` | 0.5 |