# Demo 02: Refused Path — Architecture

## What it does

User-scope agent receives the same message but has no MCP tools available. Access is denied at the mapping level.

## Flow

```
  CLI: admin agents message associate_agent
    │
    ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Context Creation                                       │
  │  core contexts create → ContextId (UUID)                │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Agent Process (associate_agent)                        │
  │  Scope: user                                            │
  │  MCP servers: NONE                                      │
  │  Tool list: EMPTY                                       │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  AI Request #1                                          │
  │  Model receives message + EMPTY tool list               │
  │  No tools to call → refuses naturally                   │
  │  "I do not have access to that tool"                    │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  NO governance hook fires                               │
  │  NO MCP tool calls                                      │
  │  Access denied at MAPPING level, not RULE level         │
  └─────────────────────────────────────────────────────────┘
```

## Contrast with Demo 01

```
  Demo 01 (admin scope)          Demo 02 (user scope)
  ─────────────────────          ─────────────────────
  MCP servers: 2                 MCP servers: 0
  Tool list: populated           Tool list: EMPTY
  AI calls tools → governance    AI sees no tools → refuses
  Result: structured artifact    Result: polite refusal
  Trace: ~11 events              Trace: ~4 events
```

## Why Rust

- **Type-safe agent config**: Agent scope and MCP server mappings are typed enums in the agent configuration — not string comparisons at runtime
- **Defense in depth**: Even if the mapping were bypassed, the governance layer (Demo 05) would still catch it at the rule level
- **Audit completeness**: The refusal is still traced — 4 events recorded with typed IDs, even though no tool was called
