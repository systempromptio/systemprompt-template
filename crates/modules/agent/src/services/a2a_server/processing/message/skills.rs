use std::sync::Arc;

use systemprompt_core_ai::{AiMessage, MessageRole};
use systemprompt_core_logging::LogService;
use systemprompt_core_system::RequestContext;

use crate::models::AgentRuntimeInfo;
use crate::services::SkillService;

pub async fn load_skills_into_messages(
    skill_service: &Arc<SkillService>,
    agent_runtime: &AgentRuntimeInfo,
    request_ctx: &RequestContext,
    log: &LogService,
) -> Vec<AiMessage> {
    let mut ai_messages = Vec::new();

    if agent_runtime.skills.is_empty() {
        return ai_messages;
    }

    log.info(
        "message_processor",
        &format!(
            "Loading {} skills for agent: {:?}",
            agent_runtime.skills.len(),
            agent_runtime.skills
        ),
    )
    .await
    .ok();

    let mut skills_prompt = String::from(
        "# Your Skills\n\nYou have the following skills that define your capabilities and writing \
         style:\n\n",
    );

    for skill_id in &agent_runtime.skills {
        match skill_service.load_skill(skill_id, request_ctx).await {
            Ok(skill_content) => {
                log.info(
                    "message_processor",
                    &format!("Loaded skill '{}' ({} chars)", skill_id, skill_content.len()),
                )
                .await
                .ok();
                skills_prompt
                    .push_str(&format!("## {} Skill\n\n{}\n\n---\n\n", skill_id, skill_content));
            },
            Err(e) => {
                log.warn(
                    "message_processor",
                    &format!("Failed to load skill '{skill_id}': {e}"),
                )
                .await
                .ok();
            },
        }
    }

    ai_messages.push(AiMessage {
        role: MessageRole::System,
        content: skills_prompt,
    });

    log.info("message_processor", "Skills injected into agent context")
        .await
        .ok();

    ai_messages
}

pub fn build_conversation_messages(
    skill_messages: Vec<AiMessage>,
    system_prompt: Option<&String>,
    conversation_history: Vec<AiMessage>,
    user_text: String,
) -> Vec<AiMessage> {
    let mut ai_messages = skill_messages;

    if let Some(prompt) = system_prompt {
        ai_messages.push(AiMessage {
            role: MessageRole::System,
            content: prompt.clone(),
        });
    }

    ai_messages.extend(conversation_history);

    ai_messages.push(AiMessage {
        role: MessageRole::User,
        content: user_text,
    });

    ai_messages
}
