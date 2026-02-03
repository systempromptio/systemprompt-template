---
title: "AI Troubleshooting"
description: "Diagnose and fix AI provider issues: auth failures, rate limiting, model errors, tool timeouts."
author: "SystemPrompt"
slug: "domain-ai-troubleshooting"
keywords: "ai, troubleshooting, debug, errors, providers"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# AI Troubleshooting

Diagnose and fix AI provider issues. Config: `services/ai/config.yaml`

> **Help**: `{ "command": "core playbooks show domain_ai-troubleshooting" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session](../cli/session.md)

---

## Diagnostic Checklist

{ "command": "infra services status" }
{ "command": "admin config show --section ai" }
{ "command": "cloud secrets list" }
{ "command": "infra logs --context ai --limit 50" }
{ "command": "plugins mcp status" }

---

## Issue: Provider Authentication Failed

Symptoms: "Invalid API key", "Unauthorized", requests fail immediately

Step 1: Check secret is set

{ "command": "cloud secrets list" }

Step 2: Check config uses `${VAR_NAME}` syntax

{ "command": "admin config show --section ai" }

Step 3: View error logs

{ "command": "infra logs --context ai --level error --limit 20" }

Solutions:

Secret not set:

{ "command": "cloud secrets set ANTHROPIC_API_KEY \"sk-ant-api03-...\"" }

Secret invalid (update with correct key):

{ "command": "cloud secrets set ANTHROPIC_API_KEY \"correct-api-key\"" }

Key prefixes:
- Anthropic: `sk-ant-`
- OpenAI: `sk-`
- Gemini: `AIza`

---

## Issue: Rate Limiting

Symptoms: "Rate limit exceeded", "Too many requests", 429 errors

Step 1: Check request volume

{ "command": "analytics ai --period hour" }

Step 2: Check fallback config

{ "command": "admin config show --section ai" }

Solutions:

Enable fallback in `services/ai/config.yaml`:

```yaml
ai:
  sampling:
    fallback_enabled: true
  providers:
    anthropic:
      enabled: true
    gemini:
      enabled: true
```

---

## Issue: Model Not Available

Symptoms: "Model not found", "Invalid model"

{ "command": "admin config show --section ai" }

Valid model names:

```yaml
providers:
  anthropic:
    default_model: claude-sonnet-4-20250514
  openai:
    default_model: gpt-4-turbo
  gemini:
    default_model: gemini-2.5-flash
```

Anthropic: `claude-opus-4-20250514`, `claude-sonnet-4-20250514`, `claude-haiku-3-20240307`
OpenAI: `gpt-4-turbo`, `gpt-4o`, `gpt-4o-mini`, `gpt-3.5-turbo`
Gemini: `gemini-2.5-flash`, `gemini-2.5-pro`, `gemini-1.5-flash`

---

## Issue: Token Limit Exceeded

Symptoms: "Token limit exceeded", "Input too long", responses cut off

{ "command": "admin config show --section ai" }

Token limits:

| Provider | Model | Input | Output |
|----------|-------|-------|--------|
| Anthropic | claude-opus-4 | 200K | 32K |
| Anthropic | claude-sonnet-4 | 200K | 16K |
| OpenAI | gpt-4-turbo | 128K | 4K |
| Gemini | gemini-2.5-flash | 1M | 8K |

Solutions:

Increase output limit in `services/ai/config.yaml`:

```yaml
ai:
  default_max_output_tokens: 16384
```

Use larger context model:

```yaml
providers:
  gemini:
    enabled: true
    default_model: gemini-2.5-flash
```

---

## Issue: Tool Execution Timeout

Symptoms: "Tool execution timed out", long delays

Step 1: Check timeout config

{ "command": "admin config show --section ai" }

Step 2: Check MCP status

{ "command": "plugins mcp status" }

Step 3: View MCP logs

{ "command": "plugins mcp logs <server_name>" }

Solutions:

Increase timeout in `services/ai/config.yaml`:

```yaml
ai:
  mcp:
    execution_timeout_ms: 60000
    retry_attempts: 3
```

Restart MCP:

{ "command": "plugins mcp restart <server_name>" }

---

## Issue: No Providers Available

Symptoms: "No providers available", all requests fail

{ "command": "admin config show --section ai" }
{ "command": "cloud secrets list" }

Solution: Enable at least one provider:

```yaml
providers:
  anthropic:
    enabled: true
    api_key: ${ANTHROPIC_API_KEY}
```

{ "command": "cloud secrets set ANTHROPIC_API_KEY \"sk-ant-...\"" }

---

## Issue: Smart Routing Not Working

Symptoms: Requests always go to default provider

{ "command": "admin config show --section ai" }

Solution: Enable smart routing with multiple providers:

```yaml
ai:
  sampling:
    enable_smart_routing: true
  providers:
    anthropic:
      enabled: true
    openai:
      enabled: true
    gemini:
      enabled: true
```

---

## Issue: Fallback Not Working

Symptoms: Primary fails, no fallback occurs

{ "command": "admin config show --section ai" }

Solution: Enable fallback with backup providers:

```yaml
ai:
  sampling:
    fallback_enabled: true
  providers:
    anthropic:
      enabled: true
      api_key: ${ANTHROPIC_API_KEY}
    openai:
      enabled: true
      api_key: ${OPENAI_API_KEY}
```

Set all keys:

{ "command": "cloud secrets set ANTHROPIC_API_KEY \"...\"" }
{ "command": "cloud secrets set OPENAI_API_KEY \"...\"" }

---

## Issue: Slow Responses

Symptoms: Long response times, timeouts on complex queries

{ "command": "infra logs --context ai --limit 50" }
{ "command": "analytics ai --period hour" }

Solutions:

Use faster model:

```yaml
providers:
  gemini:
    enabled: true
    default_model: gemini-2.5-flash
```

Enable smart routing:

```yaml
sampling:
  enable_smart_routing: true
```

---

## Log Messages

| Message | Meaning |
|---------|---------|
| `Provider request started` | Request sent |
| `Provider response received` | Success |
| `Provider error: rate_limit` | Rate limited |
| `Provider error: auth_failed` | Invalid key |
| `Tool execution started` | MCP tool called |
| `Tool execution timeout` | Tool too slow |
| `Fallback triggered` | Trying backup |

{ "command": "infra logs --context ai --level error" }
{ "command": "infra logs --follow" }

---

## Quick Reference

| Problem | First Command |
|---------|---------------|
| Auth failures | `cloud secrets list` |
| Rate limiting | `analytics ai --period hour` |
| Model errors | `admin config show --section ai` |
| Token limits | `admin config show --section ai` |
| Tool timeouts | `plugins mcp status` |
| No providers | `admin config show --section ai` |
| Any issue | `infra logs --context ai --level error` |

---

## Related

-> See [AI Providers](ai-providers.md)
-> See [MCP Troubleshooting](mcp-troubleshooting.md)
-> See [Agent Troubleshooting](agents-troubleshooting.md)
-> See [AI Service](/documentation/services/ai)