---
title: "AI Provider Configuration Guide"
description: "Configure AI providers (Anthropic, OpenAI, Gemini) via CLI. View, switch, enable/disable providers and manage API keys."
keywords:
  - ai
  - provider
  - anthropic
  - openai
  - gemini
  - configuration
category: guide
---

# AI Provider Configuration Guide

Configure AI providers for text generation, web search, and image creation. This guide covers viewing, switching, and managing providers via CLI.

## Prerequisites

**Load the [Session Playbook](../cli/session.md) first.** Verify your session and profile.

```json
{ "command": "admin session show" }
```

---

## Quick Reference: Provider Commands

| Command | Purpose |
|---------|---------|
| `admin config provider list` | View all providers and status |
| `admin config provider set <PROVIDER>` | Set default provider |
| `admin config provider enable <PROVIDER>` | Enable a provider |
| `admin config provider disable <PROVIDER>` | Disable a provider |

---

## Section 1: View Current Configuration

### List All Providers

```json
{ "command": "admin config provider list" }
```

Output shows:
- Provider name
- Enabled status (true/false)
- Is default (true/false)
- Default model
- API endpoint

---

## Section 2: Switch Default Provider

### Set Anthropic as Default

```json
{ "command": "admin config provider enable anthropic" }
```

```json
{ "command": "admin config provider set anthropic" }
```

### Set OpenAI as Default

```json
{ "command": "admin config provider enable openai" }
```

```json
{ "command": "admin config provider set openai" }
```

### Set Gemini as Default

```json
{ "command": "admin config provider enable gemini" }
```

```json
{ "command": "admin config provider set gemini" }
```

**Important:** After changing providers, restart the API service to reload configuration.

---

## Section 3: API Keys Configuration

API keys are stored in the secrets file:

```
.systemprompt/profiles/local/secrets.json
```

### Required Keys by Provider

| Provider | Secret Key | Environment Variable |
|----------|-----------|---------------------|
| Anthropic | `anthropic` | `ANTHROPIC_API_KEY` |
| OpenAI | `openai` | `OPENAI_API_KEY` |
| Gemini | `gemini` | `GEMINI_API_KEY` |

### Secrets File Format

```json
{
  "anthropic": "sk-ant-api03-...",
  "openai": "sk-proj-...",
  "gemini": "AIzaSy..."
}
```

---

## Section 4: Provider Capabilities Matrix

**Feature support varies by provider.**

| Feature | Gemini | OpenAI | Anthropic |
|---------|--------|--------|-----------|
| Text Generation | Yes | Yes | Yes |
| Web Search | Yes (Google Search) | Yes (`web_search_20250305`) | Not yet implemented |
| Image Generation | Yes (up to 4K) | Yes (1K only) | **No** |
| Max Output Tokens | 8192 | 4096 | 8192 |

### Web Search Configuration

To enable web search for a provider, add `google_search_enabled: true` to the provider config:

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

### Image Resolution by Provider

| Provider | Supported Resolutions |
|----------|----------------------|
| Gemini | 1K, 2K, 4K |
| OpenAI | 1K only |
| Anthropic | Not supported |

The image service automatically selects the best resolution supported by the configured provider.

### Cost Comparison (Cheapest Models)

| Provider | Model | Cost (per 1K tokens) |
|----------|-------|---------------------|
| Gemini | gemini-2.5-flash | ~$0.0004 |
| OpenAI | gpt-4o-mini | ~$0.00015 |
| Anthropic | claude-3-5-haiku | ~$0.004 |

---

## Section 5: Manual Configuration (Advanced)

For advanced users, edit the config file directly:

```
services/ai/config.yaml
```

### Configuration Structure

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
      google_search_enabled: true  # Reserved for future implementation
    openai:
      enabled: true
      api_key: ${OPENAI_API_KEY}
      default_model: gpt-4-turbo
      google_search_enabled: true
    gemini:
      enabled: false
      api_key: ${GEMINI_API_KEY}
      endpoint: https://generativelanguage.googleapis.com/v1beta
      default_model: gemini-2.5-flash
      google_search_enabled: true
```

---

## Section 6: Multi-Provider Behavior

### Text Generation (`create_blog_post`)

Uses the configured `default_provider` and `default_model`. All three providers work:

- **Anthropic**: Claude models (claude-sonnet-4, etc.)
- **OpenAI**: GPT models (gpt-4-turbo, gpt-4o, etc.)
- **Gemini**: Gemini models (gemini-2.5-flash, etc.)

Max tokens is set to 4096 for cross-provider compatibility.

### Web Search (`research_blog`)

Requires a provider with web search support:

- **Gemini**: Uses Google Search grounding (native integration)
- **OpenAI**: Uses `/responses` API with web search tool
- **Anthropic**: Not yet implemented (API supports `web_search_20250305` tool)

If the configured provider doesn't support web search, the tool will fail.

### Image Generation (`generate_featured_image`)

Uses the first available image provider:

1. Checks if `default_provider` supports images → uses it
2. Falls back to first available image provider (OpenAI or Gemini)

Resolution is automatically selected based on provider capabilities.

---

## Section 7: Testing Provider Configuration

### Quick Test Sequence

1. **List current config:**
   ```json
   { "command": "admin config provider list" }
   ```

2. **Enable desired provider:**
   ```json
   { "command": "admin config provider enable <provider>" }
   ```

3. **Set as default:**
   ```json
   { "command": "admin config provider set <provider>" }
   ```

4. **Restart API service** to reload configuration

5. **Test text generation (announcement doesn't need research):**
   ```bash
   systemprompt plugins mcp call content-manager create_blog_post --args '{
     "skill_id": "announcement_writing",
     "artifact_id": "",
     "slug": "test-provider",
     "description": "Test",
     "keywords": ["test"],
     "instructions": "Write a 2 sentence test announcement.",
     "category": "announcement"
   }'
   ```

### Verified Test Results (2026-02-02)

| Provider | Text Generation | Web Search | Image Generation |
|----------|----------------|------------|------------------|
| Anthropic | ✅ Works | ❌ Not implemented | ❌ N/A (uses fallback) |
| OpenAI | ✅ Works | ✅ Works | ✅ Works (1K) |
| Gemini | ✅ Works | ✅ Works | ✅ Works (up to 4K) |

---

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| "Unknown provider" | Provider not in config | Check `services/ai/config.yaml` |
| "API key not found" | Missing secret | Add key to `secrets.json` |
| "No provider with Google Search support" | Provider doesn't support web search | Enable OpenAI or Gemini with `google_search_enabled: true` |
| "Resolution not supported" | Provider doesn't support requested resolution | Automatic fallback handles this |
| Image generation fails with Anthropic | Anthropic has no image support | Enable OpenAI or Gemini for images |
| Config changes not taking effect | API not restarted | Restart API service |

---

## Related Playbooks

- [Session Playbook](../cli/session.md) - Authentication and session management
- [Blog Playbook](../content/blog.md) - Blog content creation
- [MCP Artifacts](../build/mcp-artifacts.md) - MCP tool patterns
