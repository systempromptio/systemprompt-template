---
title: "AI Provider Configuration"
description: "Configure Anthropic, OpenAI, and Gemini with fallback, smart routing, and MCP integration."
author: "SystemPrompt"
slug: "domain-ai-providers"
keywords: "ai, providers, anthropic, openai, gemini, configuration"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# AI Provider Configuration

AI provider setup. Config: `services/ai/config.yaml`

> **Help**: `{ "command": "core playbooks show domain_ai-providers" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session](../cli/session.md)

---

## Configure Anthropic

Step 1: Set API key

{ "command": "cloud secrets set ANTHROPIC_API_KEY \"sk-ant-api03-...\"" }
{ "command": "cloud secrets list" }

Step 2: Configure provider in `services/ai/config.yaml`:

```yaml
ai:
  default_provider: anthropic
  default_max_output_tokens: 8192
  sampling:
    enable_smart_routing: false
    fallback_enabled: true
  providers:
    anthropic:
      enabled: true
      api_key: ${ANTHROPIC_API_KEY}
      default_model: claude-sonnet-4-20250514
    openai:
      enabled: false
      api_key: ${OPENAI_API_KEY}
      default_model: gpt-4-turbo
    gemini:
      enabled: false
      api_key: ${GEMINI_API_KEY}
      endpoint: https://generativelanguage.googleapis.com/v1beta
      default_model: gemini-2.5-flash
  mcp:
    auto_discover: true
    connect_timeout_ms: 5000
    execution_timeout_ms: 30000
    retry_attempts: 3
  history:
    retention_days: 30
    log_tool_executions: true
```

Step 3: Verify

{ "command": "admin config validate" }
{ "command": "admin config show --section ai" }
{ "command": "admin agents message welcome -m \"Hello\" --blocking" }

Anthropic models:

| Model | Use Case | Context |
|-------|----------|---------|
| `claude-opus-4-20250514` | Complex reasoning | 200K |
| `claude-sonnet-4-20250514` | Balanced | 200K |
| `claude-haiku-3-20240307` | Fast, economical | 200K |

---

## Configure OpenAI

{ "command": "cloud secrets set OPENAI_API_KEY \"sk-...\"" }

```yaml
ai:
  default_provider: openai
  providers:
    openai:
      enabled: true
      api_key: ${OPENAI_API_KEY}
      default_model: gpt-4-turbo
```

OpenAI models:

| Model | Use Case | Context |
|-------|----------|---------|
| `gpt-4-turbo` | Latest GPT-4 | 128K |
| `gpt-4o` | Optimized speed | 128K |
| `gpt-4o-mini` | Economical | 128K |
| `gpt-3.5-turbo` | Budget | 16K |

---

## Configure Gemini

{ "command": "cloud secrets set GEMINI_API_KEY \"AIza...\"" }

```yaml
ai:
  default_provider: gemini
  providers:
    gemini:
      enabled: true
      api_key: ${GEMINI_API_KEY}
      endpoint: https://generativelanguage.googleapis.com/v1beta
      default_model: gemini-2.5-flash
```

Gemini models:

| Model | Use Case | Context |
|-------|----------|---------|
| `gemini-2.5-flash` | Fast multimodal | 1M |
| `gemini-2.5-pro` | Advanced reasoning | 1M |
| `gemini-1.5-flash` | Stable | 1M |

---

## Multi-Provider Fallback

Step 1: Set all keys

{ "command": "cloud secrets set ANTHROPIC_API_KEY \"sk-ant-...\"" }
{ "command": "cloud secrets set OPENAI_API_KEY \"sk-...\"" }
{ "command": "cloud secrets set GEMINI_API_KEY \"AIza...\"" }

Step 2: Configure `services/ai/config.yaml`:

```yaml
ai:
  default_provider: anthropic
  default_max_output_tokens: 8192
  sampling:
    enable_smart_routing: false
    fallback_enabled: true
  providers:
    anthropic:
      enabled: true
      api_key: ${ANTHROPIC_API_KEY}
      default_model: claude-sonnet-4-20250514
    openai:
      enabled: true
      api_key: ${OPENAI_API_KEY}
      default_model: gpt-4-turbo
    gemini:
      enabled: true
      api_key: ${GEMINI_API_KEY}
      endpoint: https://generativelanguage.googleapis.com/v1beta
      default_model: gemini-2.5-flash
```

Fallback order: anthropic -> openai -> gemini

---

## Smart Routing

```yaml
ai:
  sampling:
    enable_smart_routing: true
    fallback_enabled: true
```

Smart routing selects provider by request type:

| Request Type | Provider |
|--------------|----------|
| Complex reasoning | Anthropic |
| Fast queries | Gemini |
| Code generation | OpenAI/Anthropic |
| Cost-sensitive | Gemini/GPT-3.5 |

{ "command": "analytics ai --period day" }
{ "command": "infra logs --context ai --limit 50" }

---

## MCP Integration

```yaml
ai:
  mcp:
    auto_discover: true
    connect_timeout_ms: 5000
    execution_timeout_ms: 30000
    retry_attempts: 3
```

{ "command": "plugins mcp list" }
{ "command": "admin agents tools welcome" }

---

## Token Limits

```yaml
ai:
  default_max_output_tokens: 8192
```

| Provider | Model | Input | Output |
|----------|-------|-------|--------|
| Anthropic | claude-opus-4 | 200K | 32K |
| Anthropic | claude-sonnet-4 | 200K | 16K |
| OpenAI | gpt-4-turbo | 128K | 4K |
| Gemini | gemini-2.5-flash | 1M | 8K |

{ "command": "analytics ai --period day" }

---

## Switch Provider

```yaml
ai:
  default_provider: gemini
```

{ "command": "admin config validate" }

---

## Troubleshooting

- Auth failed: `{ "command": "cloud secrets list" }`, `{ "command": "cloud secrets set ANTHROPIC_API_KEY \"new-key\"" }`
- No providers: Enable at least one `enabled: true` in config
- Rate limited: Enable `fallback_enabled: true`, add backup providers

---

## Quick Reference

| Task | Command |
|------|---------|
| Set key | `cloud secrets set <NAME> "value"` |
| List secrets | `cloud secrets list` |
| Show AI config | `admin config show --section ai` |
| Validate | `admin config validate` |
| AI logs | `infra logs --context ai` |
| Analytics | `analytics ai --period day` |
| Test | `admin agents message <name> -m "text" --blocking` |

---

## Configuration Reference

| Field | Description |
|-------|-------------|
| `default_provider` | Primary: anthropic, openai, gemini |
| `default_max_output_tokens` | Max response tokens |
| `sampling.enable_smart_routing` | Auto-select provider |
| `sampling.fallback_enabled` | Try other providers on failure |
| `providers.<name>.enabled` | Enable/disable |
| `providers.<name>.api_key` | API key (use ${VAR}) |
| `providers.<name>.default_model` | Default model |
| `mcp.auto_discover` | Auto-discover MCP servers |
| `mcp.execution_timeout_ms` | Tool timeout |

---

## Related

-> See [AI Troubleshooting](ai-troubleshooting.md)
-> See [MCP Servers](mcp-servers.md)
-> See [AI Service](/documentation/services/ai)