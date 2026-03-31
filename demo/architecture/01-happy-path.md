# Demo 01: Happy Path — Architecture

## What it does

Admin-scope agent receives a message, calls an MCP tool, and returns a structured artifact.

## Flow

```
  CLI: admin agents message developer_agent
    │
    ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Context Creation                                       │
  │  core contexts create → ContextId (UUID)                │
  │  Isolates this conversation from all others             │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Agent Process (developer_agent)                        │
  │  Scope: admin                                           │
  │  MCP servers: systemprompt, skill-manager                │
  │  Skills: loaded from agent config                       │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  AI Request #1                                          │
  │  Model receives message + tool list                     │
  │  Decides to call: mcp__systemprompt__list_agents        │
  │  TraceId generated (UUID v4)                            │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  PreToolUse Hook (governance)                           │
  │  POST /api/public/hooks/govern                          │
  │  agent_id=developer_agent → scope=admin → ALLOW         │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  MCP Tool Execution                                     │
  │  systemprompt MCP server receives tool call              │
  │  Executes: systemprompt admin agents list                │
  │  Returns: JSON agent list                               │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  AI Request #2                                          │
  │  Model processes tool output                            │
  │  Formats response + creates artifact                    │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  PostToolUse Hook (tracking)                            │
  │  POST /api/public/hooks/track                           │
  │  Records: tool_name, timing, input/output bytes         │
  │  INSERT INTO plugin_usage (compile-time checked SQL)    │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Artifact Storage                                       │
  │  Typed artifact (JSON) stored with ContextId            │
  │  Retrievable by any surface: web, CLI, mobile           │
  └─────────────────────────────────────────────────────────┘
```

## Typed IDs Created

| ID | Type | When |
|----|------|------|
| ContextId | `ContextId(String)` | Context creation |
| TraceId | `TraceId(String)` | Per AI request |
| SessionId | `SessionId(String)` | Agent session |
| UserId | `UserId(String)` | JWT authentication |

## Why Rust

- **Newtype IDs**: `ContextId`, `TraceId`, `SessionId` are distinct types — the compiler prevents passing a `SessionId` where a `TraceId` is expected
- **Compile-time SQL**: Every `INSERT INTO plugin_usage` and `SELECT` uses `sqlx::query!{}` — validated against the live schema at build time
- **Zero-cost abstractions**: The newtype wrappers compile to bare strings at runtime, no boxing or indirection
- **Serde boundary**: MCP tool results arrive as JSON, immediately deserialized into typed Rust structs
