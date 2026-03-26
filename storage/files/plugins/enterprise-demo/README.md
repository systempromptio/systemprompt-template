# Enterprise Demo — foodles.com

Demonstrates enterprise governance for Claude Code using HTTP hooks, MCP servers, and secret detection policies.

## What's Inside

- **2 skills**: `example-web-search` (allowed) and `use-dangerous-secret` (blocked by governance)
- **2 MCP servers**: `systemprompt` (admin tools) and `skill-manager` (user tools)
- **HTTP hooks**: PreToolUse governance hook that blocks plaintext secrets, plus tracking hooks for all events

## Install

```bash
claude plugin marketplace add https://github.com/systempromptio/enterprise-demo.git
```

## Setup

After installing, add your plugin token to Claude Code settings:

```bash
claude settings set env.SYSTEMPROMPT_PLUGIN_TOKEN "your-token-here"
```

Get your token by signing up at [foodles.com](https://foodles.com) or by authenticating with one of the MCP servers — the platform issues a token during the OAuth flow.

## Try It

### 1. Web Search (allowed)

Ask Claude to search the web for something. The governance hook evaluates the tool call, allows it, and tracks the event.

### 2. Dangerous Secret (blocked)

Ask Claude to use the dangerous secret skill. It will attempt to write a file containing `sk-ant-demo-FAKE12345678901234567890`. The PreToolUse governance hook detects the secret pattern and blocks the tool call.

## How It Works

### HTTP Hooks

All hooks use `type: "http"` — Claude Code POSTs the event payload directly to the platform endpoint. No shell scripts required.

- **Governance** (`PreToolUse`): Synchronous hook that calls `/api/public/hooks/govern`. Returns `allow` or `deny` with a reason.
- **Tracking** (all other events): Async hooks that call `/api/public/hooks/track`. Fire-and-forget analytics.

The `Authorization` header uses `$SYSTEMPROMPT_PLUGIN_TOKEN` — Claude Code resolves this from your environment at runtime.

### MCP Servers

Both MCP servers authenticate via OAuth. Claude Code handles the OAuth flow automatically when you first use a tool from either server.

- **systemprompt**: Platform administration tools (admin scope)
- **skill-manager**: Skill and agent management tools (user scope)

### Governance Rules

The governance endpoint evaluates four rules in order:

1. **Secret detection** — scans tool inputs for API keys, tokens, passwords, and connection strings
2. **Scope check** — enforces admin-only tool restrictions based on agent scope
3. **Tool blocklist** — blocks destructive operations (delete, drop, destroy) for non-admin scopes
4. **Rate limiting** — 60 tool calls per minute per session
