# Demo 09: Agent Tracing — Architecture

## What it does

Runs a live agent through the platform runtime, showing AI reasoning, MCP tool calls, artifact creation, and full execution tracing. This is the only demo that uses a live AI agent.

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
  │  AI Request #3                                          │
  │  Model finalizes response                               │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Artifact Storage                                       │
  │  Typed artifact (JSON) stored with ContextId            │
  │  Retrievable by any surface: web, CLI, mobile           │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Trace: ~11 events                                      │
  │  ├── 3 AI requests (with tokens, cost per request)      │
  │  ├── 1 MCP tool call (tool_name, server, duration)      │
  │  ├── 7 execution steps (timing per step)                │
  │  └── Dashboard: /admin/traces?session_id=...            │
  └─────────────────────────────────────────────────────────┘
```

## Why 3 AI Requests

1. AI receives the message, sees MCP tools, decides to call `systemprompt`
2. MCP tool returns result, AI processes the tool output
3. AI formats the final response

Normal multi-turn tool use. Each step traced and costed separately.

## Typed IDs Created

| ID | Type | When |
|----|------|------|
| ContextId | `ContextId(String)` | Context creation |
| TraceId | `TraceId(String)` | Per AI request |
| SessionId | `SessionId(String)` | Agent session |
| UserId | `UserId(String)` | JWT authentication |
| ArtifactId | `ArtifactId(String)` | Artifact storage |

## Contrast with Demos 01-08

Demos 01-08 call the governance API directly with curl — they simulate the PreToolUse hook workflow without running an AI agent. Demo 09 is the only demo that runs a live agent, producing real AI requests, MCP tool calls, and artifacts.

## Why Rust

- **Newtype IDs**: All IDs are distinct types — the compiler prevents confusing `SessionId` with `TraceId`
- **Compile-time SQL**: Every trace query uses `sqlx::query_as!{}` — validated at build time
- **Zero-cost abstractions**: Newtype wrappers compile to bare strings at runtime
