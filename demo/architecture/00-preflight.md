# Demo 00: Preflight & Token Acquisition — Architecture

## What it does

1. Validates that all platform services are running
2. Creates a local admin session (JWT)
3. Extracts the plugin token (SYSTEMPROMPT_TOKEN) from the dashboard
4. Saves the token to `demo/.token` for subsequent demos

## Flow

```
  Step 1: Service Health
  ─────────────────────
  CLI Binary (target/debug|release/systemprompt)
    │
    ▼
  systemprompt infra services status
    │
    ▼
  ┌─────────────────────────────────────────────────┐
  │  Service Registry (database)                    │
  │  Queries: agents, MCP servers                   │
  │  • 3 agents (developer, associate, admin)       │
  │  • 2 MCP servers (systemprompt, skill-manager)  │
  └─────────────────────────────────────────────────┘

  Step 2: Admin Session Token
  ───────────────────────────
  systemprompt admin session login --token-only
    │
    ▼
  ┌─────────────────────────────────────────────────┐
  │  Credentials Bootstrap                          │
  │  1. Read .systemprompt/credentials.json         │
  │  2. Lookup user in PostgreSQL                   │
  │  3. Sign JWT with jwt_secret from secrets.json  │
  │  Returns: scope=admin, 24h expiry               │
  └─────────────────────────────────────────────────┘

  Step 3: Plugin Token Extraction
  ───────────────────────────────
  curl http://localhost:8080/admin/profile (with admin cookie)
    │
    ▼
  ┌─────────────────────────────────────────────────┐
  │  Dashboard HTML                                 │
  │  Extract SYSTEMPROMPT_TOKEN from widget          │
  │  Returns: scope=service, 365-day expiry         │
  └─────────────────────────────────────────────────┘

  Step 5: Save Token
  ──────────────────
  Plugin token → demo/.token
  (Read by demos 05, 06, 07, 08)
```

## Two Token Model

| Property | Admin Session | Plugin Token |
|----------|--------------|--------------|
| scope | admin | service |
| user_type | admin | service |
| expiry | 24 hours | 365 days |
| session_id | sess_\<uuid\> | plugin_\<bundle\> |
| used by | CLI, dashboard | Claude Code hooks |
| sub (user_id) | same | same |
| signing key | same jwt_secret | same jwt_secret |

## Components

| Component | Type | Purpose |
|-----------|------|---------|
| CLI binary | Rust binary | Entry point for all CLI commands |
| Service registry | PostgreSQL table | Tracks running services |
| SessionGenerator | Rust struct | Creates signed JWTs with typed claims |
| credentials.json | JSON file | Cloud identity (email, user_id) |
| secrets.json | JSON file | jwt_secret for local token signing |
| Agent processes | Subprocess (Claude Code) | AI agents with tool access |
| MCP servers | Subprocess (TCP/stdio) | Tool providers for agents |

## Why Rust

JWT claims are a typed struct — `UserId`, `SessionId`, `UserType`, `RateLimitTier` are all newtypes enforced at compile time. `SessionGenerator::new(secret, issuer)` takes typed `SessionParams`. Token validation returns typed `JwtClaims`, not a raw map — `.sub` is a `UserId`, not a `String`. The `jsonwebtoken` crate signs with HS256. All database queries use `sqlx::query!{}` macros checked against the live schema at compile time.
