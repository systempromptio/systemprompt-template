use anyhow::{anyhow, Result};
use std::sync::Arc;
use systemprompt_core_agent::models::a2a::{Message, Part};
use systemprompt_core_agent::repository::TaskRepository;
use systemprompt_core_ai::AiService;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::AppContext;
use systemprompt_identifiers::{AgentName, ContextId, SessionId, TraceId, UserId};
use systemprompt_models::ai::request::{AiMessage, AiRequest};
use systemprompt_models::ai::response_format::StructuredOutputOptions;
use systemprompt_models::execution::context::RequestContext;

use crate::models::{AiEvaluationResponse, ConversationEvaluation};
use crate::repository::SchedulerRepository;

const EVALUATION_PROMPT: &str = r#"You are an expert conversation evaluator analyzing an AI agent conversation with a user. Your task is to provide a detailed, structured evaluation of the conversation based on the actual messages exchanged.

## Evaluation Criteria

1. **Goal Achievement**: Analyze what the user was trying to accomplish and whether the agent successfully helped them achieve it. Consider the agent's capabilities and purpose when assessing success.

2. **User Satisfaction** (0-100 scale): Infer from the conversation tone, user responses, and outcomes how satisfied the user was:
   - 80-100: Very satisfied, positive tone, goals achieved
   - 60-79: Satisfied, generally positive outcome
   - 40-59: Neutral or mixed, some frustration
   - 20-39: Dissatisfied, negative indicators
   - 0-19: Very dissatisfied, clear frustration or failure

3. **Conversation Quality** (0-100 scale): Assess the overall quality:
   - 80-100: Excellent - technically correct, efficient, professional, well-structured
   - 60-79: Good - correct and helpful with minor issues
   - 40-59: Acceptable - completed task but with notable issues
   - 20-39: Poor - significant problems or inefficiencies
   - 0-19: Very poor - major failures or errors

4. **Issues Encountered**: Identify any problems during the conversation:
   - Errors or exceptions
   - Misunderstandings
   - Inefficient approaches
   - Missing capabilities
   - Performance problems
   - Communication issues

5. **Categorization**: Determine the primary topic/category and extract relevant keywords.

## Output Format

Provide your evaluation as a JSON object with the following structure:

```json
{
  "agent_goal": "Brief description of what the user was trying to accomplish",
  "goal_achieved": "yes" | "no" | "partial",
  "goal_achievement_confidence": 0.0-1.0,
  "goal_achievement_notes": "Optional explanation of goal achievement",

  "primary_category": "Main category (e.g., 'development', 'programming', 'content', 'system_administration')",
  "topics_discussed": "Comma-separated list of topics",
  "keywords": "Comma-separated relevant keywords",

  "user_satisfied": 0-100,
  "conversation_quality": 0-100,
  "quality_notes": "Optional explanation of quality rating",
  "issues_encountered": "Comma-separated list of issues, or null if none",

  "completion_status": "completed" | "abandoned" | "error",
  "overall_score": 0.0-1.0,
  "evaluation_summary": "2-3 sentence summary of the conversation and evaluation"
}
```

## Scoring Guidelines

- **User Satisfaction & Quality**: Provide numeric scores 0-100 based on the scales above
- **Goal Achievement Confidence**: How confident are you in your assessment (0.0-1.0)?
- **Overall Score**: Composite score considering all factors (0.0-1.0):
  - 0.9-1.0: Excellent conversation, goals achieved, user very satisfied
  - 0.7-0.89: Good conversation, goals mostly achieved, user satisfied
  - 0.5-0.69: Acceptable conversation, goals partially achieved, some issues
  - 0.3-0.49: Poor conversation, significant issues, user likely unsatisfied
  - 0.0-0.29: Very poor conversation, failed to achieve goals, major problems

Analyze the conversation carefully based on the actual messages provided. Provide an honest, accurate evaluation.
"#;

pub async fn evaluate_conversations(
    db_pool: DbPool,
    logger: LogService,
    app_context: Arc<AppContext>,
) -> Result<()> {
    let start_time = std::time::Instant::now();

    logger
        .info(
            "scheduler",
            "Job started | job=evaluate_conversations, batch_size=50",
        )
        .await
        .ok();

    let repository = SchedulerRepository::new(db_pool.clone());
    let conversations = repository.get_unevaluated_conversations(50).await?;

    if conversations.is_empty() {
        logger
            .debug("scheduler", "No unevaluated conversations found")
            .await
            .ok();
        return Ok(());
    }

    logger
        .debug(
            "scheduler",
            &format!("Conversations found | count={}", conversations.len()),
        )
        .await
        .ok();

    let ai_service = AiService::new(app_context.clone()).await?;

    let mut success_count = 0;
    let mut error_count = 0;

    for conversation in conversations.iter() {
        let context_id = conversation
            .get("context_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing context_id"))?;

        match evaluate_single_conversation(
            context_id,
            conversation,
            &db_pool,
            &ai_service,
            &repository,
            &logger,
        )
        .await
        {
            Ok(_) => {
                success_count += 1;
            },
            Err(e) => {
                error_count += 1;
                logger
                    .error(
                        "scheduler",
                        &format!("Evaluation failed | context_id={context_id}, error={e}"),
                    )
                    .await
                    .ok();
            },
        }
    }

    logger
        .log(
            systemprompt_core_logging::LogLevel::Info,
            "scheduler",
            &format!(
                "Job completed | job=evaluate_conversations, succeeded={}, failed={}",
                success_count, error_count
            ),
            Some(serde_json::json!({
                "job_name": "evaluate_conversations",
                "succeeded": success_count,
                "failed": error_count,
                "total_evaluated": success_count + error_count,
                "duration_ms": start_time.elapsed().as_millis(),
            })),
        )
        .await
        .ok();

    Ok(())
}

async fn evaluate_single_conversation(
    context_id: &str,
    conversation: &serde_json::Value,
    db_pool: &DbPool,
    ai_service: &AiService,
    repository: &SchedulerRepository,
    logger: &LogService,
) -> Result<()> {
    let agent_name = conversation
        .get("agent_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let started_at = conversation
        .get("started_at")
        .and_then(systemprompt_core_database::parse_database_datetime);
    let completed_at = conversation
        .get("completed_at")
        .and_then(systemprompt_core_database::parse_database_datetime);

    let conversation_duration_seconds = if let (Some(start), Some(end)) = (started_at, completed_at)
    {
        let duration = (end - start).num_seconds();
        if duration < 0 {
            0
        } else {
            duration as i32
        }
    } else {
        0
    };

    let messages = get_context_messages_with_content(db_pool, conversation).await?;

    if messages.is_empty() {
        return Err(anyhow!("No messages found for context"));
    }

    let conversation_text = reconstruct_conversation(&messages)?;
    let total_turns = messages.len() as i32;

    let req_context = create_evaluation_request_context(context_id);
    let evaluation_json =
        call_ai_evaluator(ai_service, &conversation_text, &req_context, logger).await?;

    logger
        .debug(
            "evaluations",
            &format!(
                "AI response JSON: {}",
                &evaluation_json[..std::cmp::min(500, evaluation_json.len())]
            ),
        )
        .await
        .ok();

    let ai_response: AiEvaluationResponse =
        serde_json::from_str(&evaluation_json).map_err(|e| {
            let log_msg = format!(
                "Failed to parse AI evaluation response: {} | JSON: {}",
                e,
                &evaluation_json[..std::cmp::min(500, evaluation_json.len())]
            );
            anyhow!("{}", log_msg)
        })?;

    let context_id = conversation
        .get("context_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing context_id"))?;

    let eval = ConversationEvaluation::from_ai_response(
        ai_response,
        context_id.to_string(),
        agent_name,
        total_turns,
        conversation_duration_seconds,
    );

    if eval.user_satisfied == 0 && eval.conversation_quality == 0 {
        logger
            .warn(
                "evaluations",
                &format!(
                    "Zero-score conversation detected {}: quality={}, user_satisfied={}, \
                     score={:.2}",
                    context_id, eval.conversation_quality, eval.user_satisfied, eval.overall_score
                ),
            )
            .await
            .ok();
    }

    repository.create_evaluation(&eval).await?;

    logger
        .debug(
            "evaluations",
            &format!(
                "Evaluated conversation {}: quality={}, user_satisfied={}, score={:.2}, \
                 goal_achieved={}",
                context_id,
                eval.conversation_quality,
                eval.user_satisfied,
                eval.overall_score,
                eval.goal_achieved
            ),
        )
        .await
        .ok();

    Ok(())
}

fn create_evaluation_request_context(_task_id: &str) -> RequestContext {
    RequestContext::new(
        SessionId::new("evaluation-job".to_string()),
        TraceId::new(format!("eval-{}", uuid::Uuid::new_v4())),
        ContextId::new("".to_string()),
        AgentName::system(),
    )
    .with_user_id(UserId::new("system".to_string()))
}

async fn get_context_messages_with_content(
    db_pool: &DbPool,
    conversation: &serde_json::Value,
) -> Result<Vec<Message>> {
    let context_id = conversation
        .get("context_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing context_id"))?;

    let task_repo = TaskRepository::new(db_pool.clone());
    task_repo
        .get_messages_by_context(context_id)
        .await
        .map_err(|e| {
            anyhow!(
                "Failed to retrieve messages for context {}: {}",
                context_id,
                e
            )
        })
}

fn reconstruct_conversation(messages: &[Message]) -> Result<String> {
    let mut conversation_text = String::new();

    for msg in messages {
        let role_label = match msg.role.as_str() {
            "user" => "User",
            "agent" => "Agent",
            _ => "Unknown",
        };

        conversation_text.push_str(&format!("\n{}: ", role_label));

        let mut has_content = false;
        for part in &msg.parts {
            match part {
                Part::Text(text_part) => {
                    conversation_text.push_str(&text_part.text);
                    has_content = true;
                },
                Part::File(file_part) => {
                    conversation_text.push_str(&format!(
                        "[File: {}] ",
                        file_part.file.name.as_deref().unwrap_or("unnamed")
                    ));
                    has_content = true;
                },
                Part::Data(_) => {
                    conversation_text.push_str("[Data attached] ");
                    has_content = true;
                },
            }
        }

        if !has_content {
            conversation_text.push_str("[Empty message]");
        }
    }

    if conversation_text.is_empty() {
        return Err(anyhow!("Empty conversation - no messages to evaluate"));
    }

    Ok(conversation_text)
}

async fn call_ai_evaluator(
    ai_service: &AiService,
    conversation_text: &str,
    req_context: &RequestContext,
    logger: &LogService,
) -> Result<String> {
    let mut request = AiRequest::new(vec![
        AiMessage::system(EVALUATION_PROMPT),
        AiMessage::user(format!(
            "Evaluate this conversation:\n\n{}",
            conversation_text
        )),
    ]);
    request.structured_output = Some(StructuredOutputOptions::with_json_object());

    logger
        .debug(
            "evaluations",
            "Calling AI service for evaluation (using default provider/model)...",
        )
        .await
        .ok();

    let response = ai_service.generate(request, req_context.clone()).await?;

    logger
        .debug(
            "evaluations",
            &format!(
                "AI evaluation completed (tokens: {})",
                response.tokens_used.unwrap_or(0)
            ),
        )
        .await
        .ok();

    Ok(response.content)
}
