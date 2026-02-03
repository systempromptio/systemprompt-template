---
title: "Rate Limits"
description: "API request throttling with per-endpoint limits and user tier multipliers."
author: "SystemPrompt Team"
slug: "config/rate-limits"
keywords: "rate-limits, throttling, api, burst, tiers, security"
image: "/files/images/docs/config-rate-limits.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Rate Limits

Rate limits protect your API from abuse by throttling requests per endpoint and user tier.

## Configuration

```yaml
# .systemprompt/profiles/local/profile.yaml
rate_limits:
  disabled: false
  oauth_public_per_second: 10
  contexts_per_second: 100
  agents_per_second: 20
  mcp_per_second: 200
  burst_multiplier: 3
  tier_multipliers:
    admin: 10.0
    user: 1.0
    anon: 0.5
```

## Disable for Development

```yaml
rate_limits:
  disabled: true
```

## Default Values

| Setting | Default |
|---------|---------|
| `oauth_public_per_second` | 10 |
| `oauth_auth_per_second` | 10 |
| `contexts_per_second` | 100 |
| `tasks_per_second` | 50 |
| `artifacts_per_second` | 50 |
| `agents_per_second` | 20 |
| `mcp_per_second` | 200 |
| `stream_per_second` | 100 |
| `content_per_second` | 50 |
| `burst_multiplier` | 3 |

## Tier Multipliers

Different user types get different rate limits:

| Tier | Default | Effective Rate |
|------|---------|----------------|
| `admin` | 10.0x | 10x base rate |
| `user` | 1.0x | Base rate |
| `a2a` | 5.0x | 5x base rate |
| `mcp` | 5.0x | 5x base rate |
| `service` | 5.0x | 5x base rate |
| `anon` | 0.5x | Half base rate |

## Burst Handling

The `burst_multiplier` allows temporary spikes above the base rate.

**Example**: With `contexts_per_second: 100` and `burst_multiplier: 3`:
- Steady rate: 100 req/sec
- Burst capacity: 300 requests

## Production Example

```yaml
rate_limits:
  disabled: false
  oauth_public_per_second: 5
  oauth_auth_per_second: 10
  contexts_per_second: 200
  burst_multiplier: 5
  tier_multipliers:
    admin: 20.0
    user: 1.0
    anon: 0.1
```

## Troubleshooting

**HTTP 429 "Rate limit exceeded"**
- Wait and retry with exponential backoff
- Check if using correct tier (admin vs user)
- Increase limits for specific endpoints

See the [Rate Limits Playbook](/playbooks/config-rate-limits) for detailed technical information.