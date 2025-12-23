use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;
use systemprompt::ai::AiService;
use systemprompt::models::ai::{AiMessage, AiRequest, MessageRole, StructuredOutputOptions};
use systemprompt::models::execution::context::RequestContext;

pub async fn generate_structured<T: DeserializeOwned>(
    ai_service: &AiService,
    system_prompt: &str,
    user_prompt: &str,
    schema: JsonValue,
    ctx: RequestContext,
) -> Result<T> {
    let messages = vec![
        AiMessage {
            role: MessageRole::System,
            content: system_prompt.to_string(),
        },
        AiMessage {
            role: MessageRole::User,
            content: user_prompt.to_string(),
        },
    ];

    let model_config = ctx
        .tool_model_config()
        .ok_or_else(|| anyhow!("Missing tool_model_config in request context"))?
        .clone();

    let provider = model_config
        .provider
        .as_deref()
        .ok_or_else(|| anyhow!("Missing provider in tool_model_config"))?;

    let model = model_config
        .model
        .as_deref()
        .ok_or_else(|| anyhow!("Missing model in tool_model_config"))?;

    let max_tokens = model_config
        .max_output_tokens
        .ok_or_else(|| anyhow!("Missing max_output_tokens in tool_model_config"))?;

    let request = AiRequest::builder(messages, provider, model, max_tokens, ctx)
        .with_structured_output(StructuredOutputOptions::with_schema(schema))
        .build();

    let response = ai_service.generate(&request).await?;

    serde_json::from_str(&response.content)
        .map_err(|e| anyhow!("Failed to parse structured output: {}", e))
}
