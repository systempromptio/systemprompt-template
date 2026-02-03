---
title: "Provider Extension"
description: "Add LLM and tool providers to your extension."
author: "SystemPrompt Team"
slug: "extensions/traits/provider-extension"
keywords: "providers, llm, tools, ai, mcp"
image: ""
kind: "reference"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Provider Extension

Extensions add AI capabilities via the `llm_providers()` and `tool_providers()` methods.

## LLM Providers

```rust
fn llm_providers(&self) -> Vec<Arc<dyn LlmProvider>> {
    vec![
        Arc::new(OpenAIProvider::new(self.config.openai_key.clone())),
        Arc::new(AnthropicProvider::new(self.config.anthropic_key.clone())),
    ]
}
```

### LlmProvider Trait

```rust
use systemprompt_provider_contracts::LlmProvider;

pub struct OpenAIProvider {
    api_key: String,
    model: String,
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    fn provider_id(&self) -> &str {
        "openai"
    }

    fn model_id(&self) -> &str {
        &self.model
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // Call OpenAI API
    }

    async fn stream(&self, request: CompletionRequest) -> Result<CompletionStream> {
        // Stream from OpenAI API
    }
}
```

## Tool Providers

```rust
fn tool_providers(&self) -> Vec<Arc<dyn ToolProvider>> {
    vec![
        Arc::new(DatabaseToolProvider::new(self.pool.clone())),
        Arc::new(FileSystemToolProvider::new(self.storage_path.clone())),
    ]
}
```

### ToolProvider Trait

```rust
use systemprompt_provider_contracts::ToolProvider;

pub struct DatabaseToolProvider {
    pool: Arc<PgPool>,
}

#[async_trait]
impl ToolProvider for DatabaseToolProvider {
    fn provider_id(&self) -> &str {
        "database"
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition::new("query", "Execute a SQL query")
                .with_parameter("sql", "string", "The SQL query to execute"),
            ToolDefinition::new("insert", "Insert a record")
                .with_parameter("table", "string", "Table name")
                .with_parameter("data", "object", "Record data"),
        ]
    }

    async fn execute(&self, tool: &str, args: Value) -> Result<ToolResult> {
        match tool {
            "query" => self.execute_query(args).await,
            "insert" => self.execute_insert(args).await,
            _ => Err(anyhow!("Unknown tool: {}", tool)),
        }
    }
}
```

## Typed Extension

```rust
use systemprompt::extension::prelude::ProviderExtensionTyped;

impl ProviderExtensionTyped for MyExtension {
    fn llm_providers(&self) -> Vec<Arc<dyn LlmProvider>> {
        vec![Arc::new(OpenAIProvider::default())]
    }

    fn tool_providers(&self) -> Vec<Arc<dyn ToolProvider>> {
        vec![Arc::new(DatabaseToolProvider::new(self.pool.clone()))]
    }
}
```

## Configuration

Configure providers in `profile.yaml`:

```yaml
ai:
  default_provider: openai
  providers:
    openai:
      api_key: ${OPENAI_API_KEY}
      model: gpt-4
    anthropic:
      api_key: ${ANTHROPIC_API_KEY}
      model: claude-3-sonnet
```