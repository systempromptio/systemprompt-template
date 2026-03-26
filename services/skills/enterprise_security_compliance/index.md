# Security & Compliance

Enterprise security and compliance features for AI agent deployments. Covers audit trails, secret management, access control, and regulatory compliance.

## Audit Trail

### Complete Activity Logging

Every action on the platform is logged with full context:

```bash
# View recent activity logs
systemprompt infra logs view --since 1h

# Filter by log level
systemprompt infra logs view --level error --since 24h
systemprompt infra logs view --level warn --since 1h

# Search logs by pattern
systemprompt infra logs search "authentication" --since 24h

# Log summary statistics
systemprompt infra logs summary --since 24h
```

### AI Request Auditing

Full audit trail for every AI interaction:

```bash
# List recent AI requests
systemprompt infra logs request list --limit 10

# Filter by model or provider
systemprompt infra logs request list --model claude-3 --limit 5
systemprompt infra logs request list --provider anthropic --limit 5

# Full audit of a specific request (includes conversation context)
systemprompt infra logs audit <request-id> --full

# AI request statistics
systemprompt infra logs request stats --since 24h
```

### MCP Tool Execution Tracking

```bash
# List all MCP tool executions
systemprompt infra logs tools list

# Filter by status
systemprompt infra logs tools list --status error

# Filter by MCP server
systemprompt infra logs tools list --server <server-name>
```

### Session Transcript Capture

Full conversation transcripts stored for compliance:
- Every Claude Code session captured in JSONL format
- Indexed by user ID and session ID
- Queryable through database CLI

## Secret Management

### Encrypted Secret Storage

Secrets are encrypted at rest using ChaCha20-Poly1305 AEAD:

```bash
# Manage secrets through the platform
# All secret operations are automatically audit-logged

# Secret audit log tracks:
# - Who created/accessed/rotated/deleted each secret
# - Timestamp of every operation
# - IP address of the actor
# - Per-plugin isolation of secrets
```

### Key Management

- User-owned encryption keys with versioning
- Key rotation support without data loss
- Audit logging of all key operations

## Access Control

### Role-Based Access Control (RBAC)

```bash
# List users and their roles
systemprompt admin users list

# Assign roles
systemprompt admin users role assign <user-id> --roles admin

# Promote/demote users
systemprompt admin users role promote <identifier>
systemprompt admin users role demote <identifier>

# View user details including roles and activity
systemprompt admin users show <user-id> --sessions --activity
```

### OAuth2 Security

- JWT tokens with configurable expiration
- Multiple audiences: web, api, a2a, mcp
- Scope-based permissions per MCP server
- HttpOnly secure cookies with SameSite enforcement

### IP Security

```bash
# List banned IPs
systemprompt admin users ban list

# Ban an IP (permanent or temporary)
systemprompt admin users ban add <ip> --reason "suspicious activity" --permanent
systemprompt admin users ban add <ip> --reason "brute force" --duration 24h

# Check ban status
systemprompt admin users ban check <ip>

# Unban
systemprompt admin users ban remove <ip> -y
```

### Session Management

```bash
# List active sessions for a user
systemprompt admin users session list <user-id> --active

# End a specific session
systemprompt admin users session end <session-id>

# End all sessions for a user
systemprompt admin users session end --user <user-id> --all -y

# Cleanup old anonymous sessions
systemprompt admin users session cleanup --days 30 -y
```

## Rate Limiting

### Tiered Rate Limiting

Production rate limiting with per-endpoint and per-tier controls:

| Tier | Multiplier | Use Case |
|------|-----------|----------|
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
