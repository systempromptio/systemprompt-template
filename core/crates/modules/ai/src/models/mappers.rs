use crate::models::ai::{AiMessage, MessageRole};
use crate::models::providers::anthropic::AnthropicMessage;
use crate::models::providers::gemini::GeminiContent;
use crate::models::providers::openai::OpenAiMessage;

impl From<&AiMessage> for OpenAiMessage {
    fn from(msg: &AiMessage) -> Self {
        Self {
            role: match msg.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
            }
            .to_string(),
            content: msg.content.clone(),
        }
    }
}

impl From<&AiMessage> for AnthropicMessage {
    fn from(msg: &AiMessage) -> Self {
        use crate::models::providers::anthropic::AnthropicContent;

        Self {
            role: match msg.role {
                MessageRole::System | MessageRole::Assistant => "assistant",
                MessageRole::User => "user",
            }
            .to_string(),
            content: AnthropicContent::Text(msg.content.clone()),
        }
    }
}

impl From<&AiMessage> for GeminiContent {
    fn from(msg: &AiMessage) -> Self {
        use crate::models::providers::gemini::GeminiPart;

        Self {
            role: match msg.role {
                MessageRole::System | MessageRole::User => "user",
                MessageRole::Assistant => "model",
            }
            .to_string(),
            parts: vec![GeminiPart::Text {
                text: msg.content.clone(),
            }],
        }
    }
}
