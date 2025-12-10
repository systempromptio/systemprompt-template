//! AI Executor Functions
//!
//! Provides text generation and synthesis utilities for strategies.

use anyhow::Result;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::models::a2a::Artifact;
use crate::services::SkillService;
use systemprompt_core_ai::{
    AiMessage, AiRequest, AiService, CallToolResult, MessageRole, ToolCall, ToolResultFormatter,
};
use systemprompt_core_logging::LogService;
use systemprompt_core_system::RequestContext;

use super::message::StreamEvent;
use crate::models::AgentRuntimeInfo;

pub async fn synthesize_tool_results_with_artifacts(
    ai_service: Arc<AiService>,
    agent_runtime: &AgentRuntimeInfo,
    original_messages: Vec<AiMessage>,
    initial_response: &str,
    tool_calls: &[ToolCall],
    tool_results: &[CallToolResult],
    artifacts: &[Artifact],
    tx: mpsc::UnboundedSender<StreamEvent>,
    log: &LogService,
    request_context: RequestContext,
    _skill_service: Arc<SkillService>,
) -> Result<String, ()> {
    let tool_results_context = ToolResultFormatter::format_for_synthesis(tool_calls, tool_results);
    let artifact_references = build_artifact_references(artifacts);

    let synthesis_prompt = build_synthesis_prompt(
        tool_calls.len(),
        &tool_results_context,
        &artifact_references,
    );

    let mut synthesis_messages = original_messages;
    synthesis_messages.push(AiMessage {
        role: MessageRole::Assistant,
        content: initial_response.to_string(),
    });
    synthesis_messages.push(AiMessage {
        role: MessageRole::User,
        content: synthesis_prompt,
    });

    log.info(
        "ai_executor",
        &format!(
            "Calling AI to synthesize {} tool results",
            tool_results.len()
        ),
    )
    .await
    .ok();

    let mut synthesis_request = AiRequest::new(synthesis_messages);

    if let Some(provider) = &agent_runtime.provider {
        synthesis_request = synthesis_request.with_provider(provider.clone());
    }
    if let Some(model) = &agent_runtime.model {
        synthesis_request = synthesis_request.with_model(model.clone());
    }

    match ai_service.generate(synthesis_request, request_context).await {
        Ok(response) => {
            let synthesized_text = response.content;

            log.info(
                "ai_executor",
                &format!("Synthesis complete: {} chars", synthesized_text.len()),
            )
            .await
            .ok();

            tx.send(StreamEvent::Text(synthesized_text.clone())).ok();

            Ok(synthesized_text)
        },
        Err(e) => {
            log.error("ai_executor", &format!("Synthesis failed: {e}"))
                .await
                .ok();
            Err(())
        },
    }
}

pub async fn process_without_tools(
    ai_service: Arc<AiService>,
    agent_runtime: &AgentRuntimeInfo,
    ai_messages: Vec<AiMessage>,
    tx: mpsc::UnboundedSender<StreamEvent>,
    request_context: RequestContext,
) -> Result<(String, Vec<ToolCall>, Vec<CallToolResult>), ()> {
    let mut generate_request = AiRequest::new(ai_messages);

    if let Some(provider) = &agent_runtime.provider {
        generate_request = generate_request.with_provider(provider.clone());
    }
    if let Some(model) = &agent_runtime.model {
        generate_request = generate_request.with_model(model.clone());
    }

    match ai_service
        .generate_stream(generate_request, request_context)
        .await
    {
        Ok(mut stream) => {
            let mut accumulated_text = String::new();
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(text) => {
                        accumulated_text.push_str(&text);
                        tx.send(StreamEvent::Text(text)).ok();
                    },
                    Err(e) => {
                        tx.send(StreamEvent::Error(e.to_string())).ok();
                        return Err(());
                    },
                }
            }
            Ok((accumulated_text, Vec::new(), Vec::new()))
        },
        Err(e) => {
            tx.send(StreamEvent::Error(e.to_string())).ok();
            Err(())
        },
    }
}

fn build_synthesis_prompt(
    tool_count: usize,
    tool_results_context: &str,
    artifact_references: &str,
) -> String {
    format!(
        r#"# Tool Execution Complete

You executed {} tool(s). Now provide a BRIEF conversational response.

## Tool Results Summary

{}

## Artifacts Created

{}

## CRITICAL RULES - READ CAREFULLY

1. **NEVER repeat artifact content** - The user sees artifacts separately. Your message should REFERENCE them, never duplicate their content.
2. **Maximum 100 words** - Be extremely concise. 2-3 sentences is ideal.
3. **Describe what was done, not what it contains** - Say "I've created a blog post about X" NOT "Here is the blog post: [full content]"
4. **Be conversational** - Natural, friendly summary. Not a report or transcript.
5. **Reference artifacts naturally** - Use format like "(see the artifact for the full content)"

## BAD EXAMPLE (DO NOT DO THIS)
"I've created your blog post. Here's the content:

[2000 words of article text]

Let me know if you'd like any changes."

## GOOD EXAMPLE
"Done! I've created a blog post exploring the Human-AI collaboration workflow. The article covers the key differences between automation and augmentation approaches, with practical steps for maintaining your authentic voice. Take a look at the artifact and let me know if you'd like any adjustments."

---

Provide your brief, conversational response now. Remember: the artifact has the content - your message is just the friendly summary."#,
        tool_count, tool_results_context, artifact_references
    )
}

fn build_artifact_references(artifacts: &[Artifact]) -> String {
    if artifacts.is_empty() {
        return "No artifacts were created.".to_string();
    }

    artifacts
        .iter()
        .map(|artifact| {
            let artifact_type = &artifact.metadata.artifact_type;
            let artifact_name = artifact
                .name
                .clone()
                .unwrap_or_else(|| artifact.artifact_id.clone());

            format!(
                "- **{}** ({}): Reference as '(see {} for details)'",
                artifact_name, artifact_type, artifact_name
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
