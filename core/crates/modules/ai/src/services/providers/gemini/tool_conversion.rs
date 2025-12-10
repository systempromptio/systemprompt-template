use anyhow::{anyhow, Result};
use std::collections::HashSet;

use crate::models::providers::gemini::{
    GeminiFunctionDeclaration, GeminiPart, GeminiResponse, GeminiThinkingConfig, GeminiTool,
    GoogleSearch,
};
use crate::models::tools::{McpTool, ToolCall};
use crate::services::schema::{
    DiscriminatedUnion, ProviderCapabilities, SchemaTransformer, TransformedTool,
};
use systemprompt_identifiers::AiToolCallId;
use uuid::Uuid;

use super::constants::tokens;
use super::provider::GeminiProvider;

pub fn convert_tools(provider: &GeminiProvider, tools: Vec<McpTool>) -> Result<Vec<GeminiTool>> {
    let transformer = SchemaTransformer::new(ProviderCapabilities::gemini());
    let mut mapper = provider
        .tool_mapper
        .lock()
        .map_err(|e| anyhow!("Lock poisoned: {e}"))?;

    let transformed_tools: Vec<TransformedTool> = tools
        .into_iter()
        .map(|tool| {
            let schema = tool.input_schema.as_ref();
            let discriminator_field = schema
                .and_then(DiscriminatedUnion::detect)
                .map(|u| u.discriminator_field);

            let result = transformer.transform(&tool)?;

            for t in &result {
                mapper.register_transformation(t, discriminator_field.clone());
            }

            Ok(result)
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();

    let mut seen_names = HashSet::new();
    let deduplicated_tools: Vec<_> = transformed_tools
        .into_iter()
        .filter(|tool| seen_names.insert(tool.name.clone()))
        .collect();

    let mut gemini_tools = Vec::new();

    if !deduplicated_tools.is_empty() {
        gemini_tools.push(GeminiTool {
            function_declarations: Some(
                deduplicated_tools
                    .into_iter()
                    .map(|tool| GeminiFunctionDeclaration {
                        name: tool.name,
                        description: Some(tool.description),
                        parameters: tool.input_schema,
                    })
                    .collect(),
            ),
            google_search: None,
            url_context: None,
            code_execution: None,
        });
    } else if provider.google_search_enabled {
        gemini_tools.push(GeminiTool {
            function_declarations: None,
            google_search: Some(GoogleSearch {}),
            url_context: None,
            code_execution: None,
        });
    }

    Ok(gemini_tools)
}

pub async fn extract_tool_response(
    provider: &GeminiProvider,
    response: &GeminiResponse,
    logger: &Option<systemprompt_core_logging::LogService>,
) -> Result<(String, Vec<ToolCall>)> {
    let candidate = response
        .candidates
        .first()
        .ok_or_else(|| anyhow!("No response from Gemini"))?;

    let mut content = String::new();
    let mut tool_calls = Vec::new();

    if let Some(candidate_content) = &candidate.content {
        let mapper = provider
            .tool_mapper
            .lock()
            .map_err(|e| anyhow!("Lock poisoned: {e}"))?;

        for part in &candidate_content.parts {
            match part {
                GeminiPart::Text { text } => {
                    content.push_str(text);
                },
                GeminiPart::FunctionCall { function_call } => {
                    let (original_name, resolved_args) =
                        mapper.resolve_tool_call(&function_call.name, function_call.args.clone());

                    tool_calls.push(ToolCall {
                        ai_tool_call_id: AiToolCallId::from(Uuid::new_v4().to_string()),
                        name: original_name,
                        arguments: resolved_args,
                    });
                },
                _ => {},
            }
        }
    } else {
        let reason = candidate.finish_reason.as_deref().unwrap_or("UNKNOWN");

        if reason == "MALFORMED_FUNCTION_CALL" {
            if let Some(ref log) = logger {
                log.error(
                    "gemini_tools",
                    "Gemini MALFORMED_FUNCTION_CALL - model generated invalid tool call JSON",
                )
                .await
                .ok();
            }
            return Err(anyhow!(
                "Gemini returned no content. Finish reason: MALFORMED_FUNCTION_CALL"
            ));
        }

        return Err(anyhow!(
            "Gemini returned no content. Finish reason: {reason}"
        ));
    }

    Ok((content, tool_calls))
}

pub fn build_thinking_config(model: &str) -> Option<GeminiThinkingConfig> {
    if model.contains("2.5") {
        Some(GeminiThinkingConfig {
            thinking_budget: Some(tokens::THINKING_BUDGET),
            include_thoughts: Some(false),
        })
    } else {
        None
    }
}
