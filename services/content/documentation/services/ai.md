---
title: "AI Services"
description: "Configure AI providers including Anthropic, OpenAI, and Gemini. Set up model parameters, MCP integration, and smart routing between providers."
author: "SystemPrompt Team"
slug: "services/ai"
keywords: "ai, providers, anthropic, openai, gemini, models, mcp"
image: "/files/images/docs/services-ai.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# AI Services

**TL;DR:** The AI service configures which LLM providers power your agents. It supports Anthropic, OpenAI, and Gemini with automatic fallback, smart routing, and MCP tool integration. The AI service is the bridge between agents and the language models that enable reasoning.

## The Problem

Agents need access to language models for reasoning and generation. Different providers have different strengths, pricing, and availability. You might want Anthropic for complex reasoning, OpenAI for compatibility, or Gemini for cost efficiency.

The AI service solves this by providing a unified configuration for multiple providers. You can enable one or many providers, set defaults, and configure fallback behavior. The same agent configuration works regardless of which provider is active.

## How AI Services Work

The AI service manages the connection between agents and LLM providers. When an agent needs to generate a response, it sends the conversation to the AI service. The AI service selects a provider, formats the request, handles tool calls through MCP, and returns the response.

The selection logic considers:
- Which providers are enabled
- The default provider setting
- Smart routing rules (if enabled)
- Fallback configuration (if the primary fails)

## Configuration

Configure AI providers in `services/ai/config.yaml`:

<details>
<summary>Full AI configuration</summary>

```yaml
# services/ai/config.yaml
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
      enabled: true
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

</details>

## Providers

### Anthropic

Anthropic provides Claude models known for strong reasoning and safety:

```yaml
anthropic:
  enabled: true
  api_key: ${ANTHROPIC_API_KEY}
  default_model: claude-sonnet-4-20250514
```

Available models:
- `claude-sonnet-4-20250514` - Balanced performance and cost
- `claude-opus-4-20250514` - Most capable for complex tasks
- `claude-haiku-3-20240307` - Fast and economical

### OpenAI

OpenAI provides GPT models with broad compatibility:

```yaml
openai:
  enabled: true
  api_key: ${OPENAI_API_KEY}
  default_model: gpt-4-turbo
```

Available models:
- `gpt-4-turbo` - Latest GPT-4 with large context
- `gpt-4o` - Optimized for speed
- `gpt-3.5-turbo` - Fast and economical

### Gemini

Google's Gemini provides multimodal capabilities:

```yaml
gemini:
  enabled: true
  api_key: ${GEMINI_API_KEY}
  endpoint: https://generativelanguage.googleapis.com/v1beta
  default_model: gemini-2.5-flash
```

Available models:
- `gemini-2.5-flash` - Fast multimodal processing
- `gemini-2.5-pro` - Advanced reasoning

## Web Search

Some providers support web search for grounded responses with real-time information.

### Enabling Web Search

Add `google_search_enabled: true` to a provider's configuration:

```yaml
providers:
  openai:
    enabled: true
    api_key: ${OPENAI_API_KEY}
    default_model: gpt-4-turbo
    google_search_enabled: true  # Enables OpenAI web search

  gemini:
    enabled: true
    api_key: ${GEMINI_API_KEY}
    default_model: gemini-2.5-flash
    google_search_enabled: true  # Enables Google Search grounding
```

### Web Search Capabilities

| Provider | Implementation | Notes |
|----------|---------------|-------|
| Gemini | Google Search grounding | Native integration, returns sources |
| OpenAI | `/responses` API with web search tool | Uses `web_search` tool type |
| Anthropic | Not yet implemented | API supports `web_search_20250305` tool |

Web search is used by tools like `research_blog` in the content-manager MCP server.

## Image Generation

The AI service also supports image generation through the same provider configuration.

### Supported Image Providers

| Provider | Model | Description |
|----------|-------|-------------|
| Gemini | `gemini-2.5-flash-image` | Fast image generation |
| Gemini | `gemini-3-pro-image-preview` | Higher quality images |
| OpenAI | `dall-e-3` | DALL-E 3 image generation |
| OpenAI | `dall-e-2` | DALL-E 2 image generation |

### Configuration

Image generation uses the same provider configuration as text generation. If a provider is enabled with a valid API key, image generation is automatically available:

```yaml
providers:
  gemini:
    enabled: true
    api_key: ${GEMINI_API_KEY}
    # Image generation uses same credentials

  openai:
    enabled: true
    api_key: ${OPENAI_API_KEY}
    # DALL-E uses same credentials
```

### Capabilities

| Provider | Resolutions | Aspect Ratios | Batch | Editing |
|----------|-------------|---------------|-------|---------|
| Gemini | 1K, 2K, 4K | Square, 16:9, 9:16, 4:3, 3:4, UltraWide | Yes | Yes |
| OpenAI | 1K | Square, 16:9, 9:16 | No | Yes |

### Automatic Resolution Selection

The image service automatically selects the best resolution supported by the configured provider. When requesting an image, the service queries the provider's capabilities and chooses the highest supported resolution (4K > 2K > 1K).

This ensures cross-provider compatibility without hardcoding resolution values.

### Usage

Image generation is available through MCP tools like `generate_featured_image` in the content-manager server:

```bash
systemprompt plugins mcp call content-manager generate_featured_image -a '{
  "skill_id": "blog_image_generation",
  "topic": "AI Development",
  "title": "Building with AI",
  "summary": "A guide to AI development"
}' --timeout 120
```

Generated images are stored in `/files/images/generated/` and accessible via the configured URL prefix.

## MCP Integration

The AI service auto-discovers MCP servers and makes their tools available to the language model:

```yaml
mcp:
  auto_discover: true          # Find MCP servers automatically
  connect_timeout_ms: 5000     # Connection timeout
  execution_timeout_ms: 30000  # Tool execution timeout
  retry_attempts: 3            # Retry failed tool calls
```

When `auto_discover` is enabled, the AI service finds all configured MCP servers and registers their tools. During conversations, the language model can call these tools to perform actions.

## Smart Routing

When enabled, smart routing selects the best provider for each request:

```yaml
sampling:
  enable_smart_routing: true
  fallback_enabled: true
```

Smart routing considers task complexity, cost, and provider availability. If the primary provider fails and fallback is enabled, the AI service tries alternative providers.

## Environment Variables

Store API keys securely using environment variables:

```bash
# Set via CLI
systemprompt cloud secrets set ANTHROPIC_API_KEY "sk-ant-..."
systemprompt cloud secrets set OPENAI_API_KEY "sk-..."
systemprompt cloud secrets set GEMINI_API_KEY "AIza..."
```

Never commit API keys to configuration files. Use the `${VAR_NAME}` syntax to reference environment variables.

## Service Relationships

The AI service connects to:

- **Agents** - Provides LLM capabilities for agent reasoning
- **MCP servers** - Auto-discovers tools for language model access
- **Config service** - Included through the aggregation pattern
- **History** - Logs conversations and tool executions

## Configuration Reference

| Field | Type | Description |
|-------|------|-------------|
| `default_provider` | string | Primary provider (anthropic, openai, gemini) |
| `default_max_output_tokens` | number | Maximum tokens for responses |
| `sampling.enable_smart_routing` | boolean | Enable intelligent provider selection |
| `sampling.fallback_enabled` | boolean | Try other providers on failure |
| `mcp.auto_discover` | boolean | Auto-discover MCP servers |
| `history.retention_days` | number | Days to retain conversation history |

## CLI Reference

### Provider Management

| Command | Description |
|---------|-------------|
| `systemprompt admin config provider list` | View all providers with status |
| `systemprompt admin config provider set <PROVIDER>` | Set default provider |
| `systemprompt admin config provider enable <PROVIDER>` | Enable a provider |
| `systemprompt admin config provider disable <PROVIDER>` | Disable a provider |

### Secrets Management

| Command | Description |
|---------|-------------|
| `systemprompt cloud secrets set ANTHROPIC_API_KEY <key>` | Set Anthropic API key |
| `systemprompt cloud secrets set OPENAI_API_KEY <key>` | Set OpenAI API key |
| `systemprompt cloud secrets set GEMINI_API_KEY <key>` | Set Gemini API key |
| `systemprompt cloud secrets list` | List configured secrets |

### Other AI Commands

| Command | Description |
|---------|-------------|
| `systemprompt admin config show` | Show current configuration including AI settings |
| `systemprompt plugins mcp list` | List MCP servers (AI integrates with these) |

See `systemprompt admin config provider --help` for detailed options.

## Troubleshooting

**Provider authentication failed** -- Verify the API key is set correctly. Check that the environment variable is available to the application.

**Tool execution timeout** -- The MCP tool took too long. Increase `execution_timeout_ms` or optimize the tool.

**No providers available** -- At least one provider must be enabled with valid credentials.