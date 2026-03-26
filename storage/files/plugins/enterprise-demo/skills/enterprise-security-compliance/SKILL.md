---
name: "Security & Compliance"
description: "Enterprise security and compliance - audit trails, secret management, IP enforcement, session control, and regulatory compliance for AI agent deployments"
---

|-----------|----------|
| Admin | 10x | Platform administrators |
| User | 1x | Standard authenticated users |
| A2A | 5x | Agent-to-agent communication |
| MCP | 5x | MCP server tool calls |
| Service | 5x | Internal service accounts |
| Anonymous | 0.5x | Unauthenticated requests |

Burst multiplier of 3x allows temporary overages for legitimate traffic spikes.

### Per-Endpoint Limits

Each API endpoint has configurable rate limits:
- OAuth: 10 req/s
- Contexts: 100 req/s
- Agent operations: 20 req/s
- MCP operations: 200 req/s
- Streaming: 100 req/s

## Security Headers

Production deployment includes:

| Header | Value |
|--------|-------|
| HSTS | max-age=63072000; includeSubDomains; preload |
| X-Frame-Options | DENY |
| X-Content-Type-Options | nosniff |
| Referrer-Policy | strict-origin-when-cross-origin |
| Permissions-Policy | camera=(), microphone=(), geolocation=() |

## Compliance Patterns

### For InfoSec Teams

1. **Agent Inventory**: Complete registry of all agents with their capabilities and access levels
2. **Data Flow Mapping**: Track which agents access which data through MCP tool logging
3. **Access Reviews**: RBAC with department-based rules enables periodic access reviews
4. **Incident Response**: IP banning, session termination, and log forensics available through CLI
5. **Change Management**: Agent configuration changes tracked through version control
