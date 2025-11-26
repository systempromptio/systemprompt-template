use anyhow::Result;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::models::a2a::Artifact;
use crate::services::SkillService;
use systemprompt_core_ai::{
    AiMessage, AiService, CallToolResult, GenerateRequest, MessageRole, SamplingMetadata, ToolCall,
    ToolResultFormatter, TooledRequest,
};
use systemprompt_core_logging::LogService;
use systemprompt_core_system::RequestContext;
use systemprompt_identifiers::AgentName;

use super::message::StreamEvent;
use crate::models::AgentRuntimeInfo;

pub async fn process_with_agentic_tools(
    ai_service: Arc<AiService>,
    agent_name: &AgentName,
    agent_runtime: &AgentRuntimeInfo,
    ai_messages: Vec<AiMessage>,
    tx: mpsc::UnboundedSender<StreamEvent>,
    log: LogService,
    request_context: RequestContext,
    skill_service: Arc<SkillService>,
) -> Result<(String, Vec<ToolCall>, Vec<CallToolResult>, usize), ()> {
    let mut ai_messages = ai_messages;

    if !agent_runtime.skills.is_empty() {
        log.info(
            "ai_executor",
            &format!(
                "Loading {} skills for agent: {:?}",
                agent_runtime.skills.len(),
                agent_runtime.skills
            ),
        )
        .await
        .ok();

        let mut skills_prompt = String::from("# Your Skills\n\nYou have the following skills that define your capabilities and writing style:\n\n");

        for skill_id in &agent_runtime.skills {
            match skill_service.load_skill(skill_id, &request_context).await {
                Ok(skill_content) => {
                    log.info(
                        "ai_executor",
                        &format!(
                            "✅ Loaded skill '{}' ({} chars)",
                            skill_id,
                            skill_content.len()
                        ),
                    )
                    .await
                    .ok();
                    skills_prompt.push_str(&format!(
                        "## {} Skill\n\n{}\n\n---\n\n",
                        skill_id, skill_content
                    ));
                },
                Err(e) => {
                    log.warn(
                        "ai_executor",
                        &format!("⚠️  Failed to load skill '{}': {}", skill_id, e),
                    )
                    .await
                    .ok();
                },
            }
        }

        ai_messages.insert(
            0,
            AiMessage {
                role: MessageRole::System,
                content: skills_prompt,
            },
        );

        log.info("ai_executor", "Skills injected into agent system prompt")
            .await
            .ok();
    }
    match ai_service
        .list_available_tools_for_agent(agent_name, &request_context)
        .await
    {
        Ok(mut tools) if !tools.is_empty() => {
            let original_count = tools.len();
            tools.sort_by(|a, b| a.name.cmp(&b.name));
            tools.dedup_by(|a, b| a.name == b.name);

            if tools.len() < original_count {
                log.info(
                    "ai_executor",
                    &format!(
                        "Deduplicated {} tools to {} (removed {} duplicates)",
                        original_count,
                        tools.len(),
                        original_count - tools.len()
                    ),
                )
                .await
                .ok();
            }

            log.info(
                "ai_executor",
                &format!(
                    "Processing with agentic executor using {} tools",
                    tools.len()
                ),
            )
            .await
            .ok();

            match ai_service
                .execute_agentic_loop(ai_messages, tools, &request_context)
                .await
            {
                Ok(result) => {
                    for call in &result.tool_calls {
                        tx.send(StreamEvent::ToolCallStarted(call.clone())).ok();
                    }
                    for (idx, tool_result) in result.tool_results.iter().enumerate() {
                        let call_id = result
                            .tool_calls
                            .get(idx)
                            .map(|c| c.ai_tool_call_id.as_ref().to_string())
                            .unwrap_or_else(|| format!("unknown_{}", idx));
                        tx.send(StreamEvent::ToolResult {
                            call_id,
                            result: tool_result.clone(),
                        })
                        .ok();
                    }

                    stream_text_chunks(&result.final_response, &tx).await;

                    Ok((
                        result.final_response,
                        result.tool_calls,
                        result.tool_results,
                        result.total_iterations,
                    ))
                },
                Err(e) => {
                    tx.send(StreamEvent::Error(e.to_string())).ok();
                    Err(())
                },
            }
        },
        _ => {
            log.warn("ai_executor", "No tools available for agentic execution")
                .await
                .ok();
            let (text, calls, results) =
                process_without_tools(ai_service, agent_runtime, ai_messages, tx, request_context)
                    .await?;
            Ok((text, calls, results, 1))
        },
    }
}

pub async fn process_with_tools(
    ai_service: Arc<AiService>,
    agent_name_str: &str,
    agent_runtime: &AgentRuntimeInfo,
    ai_messages: Vec<AiMessage>,
    tx: mpsc::UnboundedSender<StreamEvent>,
    log: LogService,
    request_context: RequestContext,
) -> Result<(String, Vec<ToolCall>, Vec<CallToolResult>, usize), ()> {
    let agent_name = AgentName::new(agent_name_str);
    match ai_service
        .list_available_tools_for_agent(&agent_name, &request_context)
        .await
    {
        Ok(tools) if !tools.is_empty() => {
            log.info(
                "ai_executor",
                &format!(
                    "Processing with {} tools for agent {}",
                    tools.len(),
                    agent_name_str
                ),
            )
            .await
            .ok();

            let tooled_request = TooledRequest {
                provider: agent_runtime.provider.clone(),
                model: agent_runtime.model.clone(),
                messages: ai_messages.clone(),
                tools,
                metadata: Some(SamplingMetadata::default()),
                response_format: None,
                structured_output: None,
                context_id: request_context.execution.context_id.clone(),
                task_id: request_context
                    .task_id()
                    .cloned()
                    .expect("task_id required"),
            };

            match ai_service
                .generate_with_tools(tooled_request, request_context.clone())
                .await
            {
                Ok(response) => {
                    let tool_calls = response.tool_calls.clone();
                    let tool_results = response.tool_results.clone();

                    for call in &tool_calls {
                        tx.send(StreamEvent::ToolCallStarted(call.clone())).ok();
                    }
                    for (idx, result) in tool_results.iter().enumerate() {
                        let call_id = tool_calls
                            .get(idx)
                            .map(|c| c.ai_tool_call_id.as_ref().to_string())
                            .unwrap_or_else(|| format!("unknown_{}", idx));
                        tx.send(StreamEvent::ToolResult {
                            call_id,
                            result: result.clone(),
                        })
                        .ok();
                    }

                    let executable_calls: Vec<_> = tool_calls
                        .iter()
                        .filter(|tc| tc.is_executable())
                        .cloned()
                        .collect();

                    if executable_calls.is_empty() || tool_results.is_empty() {
                        stream_text_chunks(&response.content, &tx).await;
                    }

                    Ok((response.content.clone(), tool_calls, tool_results, 1))
                },
                Err(e) => {
                    tx.send(StreamEvent::Error(e.to_string())).ok();
                    Err(())
                },
            }
        },
        _ => {
            let (text, calls, results) = process_without_tools(
                ai_service,
                agent_runtime,
                ai_messages,
                tx,
                request_context.clone(),
            )
            .await?;
            Ok((text, calls, results, 1))
        },
    }
}

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

    let synthesis_prompt = format!(
        r#"# Tool Execution Complete

You executed {} tool(s). Now synthesize the results into a clear, conversational response.

## Tool Results

{}

## Artifacts Created

{}

## Your Task

Synthesize these tool results into a natural response:

1. **Maintain your voice and personality** - This should sound like YOU, not a generic assistant
2. **Explain what was found/done** - Clear, concise summary of key findings
3. **Reference artifacts naturally** - Use the artifact reference format provided above
4. **Answer the user's question** - Connect results back to what they asked for
5. **Be conversational** - Natural language, not mechanical reporting

---

Provide your synthesized response now. Remember: maintain your personality, be clear and concise, reference artifacts naturally."#,
        tool_calls.len(),
        tool_results_context,
        artifact_references
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

    let synthesis_request = GenerateRequest {
        provider: agent_runtime.provider.clone(),
        model: agent_runtime.model.clone(),
        messages: synthesis_messages,
        metadata: Some(SamplingMetadata::default()),
        response_format: None,
        structured_output: None,
    };

    match ai_service
        .generate_stream(synthesis_request, request_context)
        .await
    {
        Ok(mut stream) => {
            let mut synthesized_text = String::new();
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(text) => {
                        synthesized_text.push_str(&text);
                        tx.send(StreamEvent::Text(text)).ok();
                    },
                    Err(e) => {
                        log.error("ai_executor", &format!("Synthesis stream error: {}", e))
                            .await
                            .ok();
                        return Err(());
                    },
                }
            }

            log.info(
                "ai_executor",
                &format!("Synthesis complete: {} chars", synthesized_text.len()),
            )
            .await
            .ok();

            Ok(synthesized_text)
        },
        Err(e) => {
            log.error("ai_executor", &format!("Synthesis failed: {}", e))
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
    let generate_request = GenerateRequest {
        provider: agent_runtime.provider.clone(),
        model: agent_runtime.model.clone(),
        messages: ai_messages,
        metadata: Some(SamplingMetadata::default()),
        response_format: None,
        structured_output: None,
    };

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

async fn stream_text_chunks(text: &str, tx: &mpsc::UnboundedSender<StreamEvent>) {
    let mut char_iter = text.chars().peekable();
    let mut chunk = String::new();
    while let Some(c) = char_iter.next() {
        chunk.push(c);
        if chunk.len() >= 20 || char_iter.peek().is_none() {
            tx.send(StreamEvent::Text(chunk.clone())).ok();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            chunk.clear();
        }
    }
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
