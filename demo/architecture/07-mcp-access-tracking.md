# Demo 07: MCP Access Tracking — Architecture

## What it does

Demonstrates governance (allow + deny), live MCP tool calls with OAuth, and database audit trail.

## Flow

```
  ┌─────────────────────────────────────────────────────────┐
  │  Part 1: Governance ALLOW                               │
  │  curl → POST /hooks/govern                              │
  │  Clean Read tool input → all rules pass → ALLOW         │
  └─────────────────────────────────────────────────────────┘
                         │
  ┌─────────────────────────────────────────────────────────┐
  │  Part 2: Governance DENY                                │
  │  curl → POST /hooks/govern                              │
  │  AWS key in Bash command → secret_injection → DENY      │
  └─────────────────────────────────────────────────────────┘
                         │
  ┌─────────────────────────────────────────────────────────┐
  │  Part 3: MCP Tool Call                                  │
  │                                                         │
  │  CLI: plugins mcp call skill-manager list_plugins       │
  │    │                                                    │
  │    ▼                                                    │
  │  ┌───────────────────────────────────────┐              │
  │  │  OAuth Authentication                 │              │
  │  │  Client credentials → access token    │              │
  │  │  Token validated against MCP server   │              │
  │  └──────────────────┬────────────────────┘              │
  │                     │                                   │
  │                     ▼                                   │
  │  ┌───────────────────────────────────────┐              │
  │  │  Tool Execution                       │              │
  │  │  skill-manager.list_plugins()         │              │
  │  │  Returns: plugin list JSON            │              │
  │  └──────────────────┬────────────────────┘              │
  │                     │                                   │
  │                     ▼                                   │
  │  ┌───────────────────────────────────────┐              │
  │  │  Access Recording                     │              │
  │  │  record_mcp_access() →                │              │
  │  │  INSERT INTO user_activity            │              │
  │  │  category="mcp_access"                │              │
  │  │  metadata: { tool_name, server }      │              │
  │  └───────────────────────────────────────┘              │
  └─────────────────────────────────────────────────────────┘
                         │
  ┌─────────────────────────────────────────────────────────┐
  │  Part 4: Audit Trail                                    │
  │                                                         │
  │  governance_decisions table:                            │
  │    SELECT decision, tool_name, reason                   │
  │    → Shows allow + deny from Parts 1-2                  │
  │                                                         │
  │  user_activity table:                                   │
  │    SELECT action, entity_name, description              │
  │    WHERE category = 'mcp_access'                        │
  │    → Shows authenticated + used events from Part 3      │
  └─────────────────────────────────────────────────────────┘
```

## Database Tables

```
  governance_decisions                 user_activity
  ────────────────────                 ─────────────
  id (UUID)                           id (UUID)
  user_id (UserId)                    user_id (UserId)
  session_id (SessionId)              category ("mcp_access")
  tool_name                           action ("authenticated"/"used")
  decision ("allow"/"deny")           entity_name (server name)
  policy                              description
  reason                              metadata (JSONB)
  evaluated_rules (JSONB)             created_at
  created_at
```

## Why Rust

- **OAuth flow is typed**: The OAuth client credentials exchange returns typed `TokenResponse { access_token, token_type, expires_in }` — not a raw JSON blob
- **MCP access recording**: `record_mcp_access()` takes typed `McpServerId` and `UserId` parameters — you can't accidentally log the wrong server or user
- **JSONB metadata**: The `metadata` column stores `serde_json::Value` serialized from typed Rust structs — queryable via PostgreSQL JSON operators
- **Compile-time queries**: Both `governance_decisions` and `user_activity` inserts are `sqlx::query!{}` — schema changes break the build, not production
