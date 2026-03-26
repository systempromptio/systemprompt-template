use std::sync::Arc;

use systemprompt::ai::{AiMessage, AiRequest, AiService, StructuredOutputOptions};
use systemprompt::identifiers::{AgentName, ContextId, SessionId, TraceId, UserId};
use systemprompt::models::auth::UserType;
use systemprompt::models::execution::context::RequestContext;

use crate::admin::repositories::{session_analyses, usage_aggregations};

pub use super::ai_summary_types::*;

use super::ai_context;
use super::session_summary;

const SYSTEM_PROMPT: &str = "\
You are analysing a Claude Code session to understand what the user wanted, whether they got it, how efficiently they worked, and how they can improve.

Focus on the USER MESSAGES to understand intent. These are the user's own words — they are the primary signal.

Return ONLY valid JSON:

{
  \"title\": \"What the user wanted to do, max 80 chars\",
  \"description\": \"The user's primary goal in one sentence, max 200 chars\",
  \"goal_summary\": \"What the user wanted to accomplish, 1-2 sentences, max 200 chars\",
  \"category\": \"feature|bugfix|refactoring|techdebt|documentation|discovery|testing|deployment|configuration|design|review|other\",
  \"goal_outcome_map\": [{\"goal\": \"Specific goal from user messages\", \"outcome\": \"What actually happened\", \"achieved\": true}, ...],
  \"outcomes\": [\"Outcome mapped to goal: Achieved/Not achieved\", ...],
  \"tags\": [\"coding\", ...],
  \"goal_achieved\": \"yes|partial|no\",
  \"quality_score\": 1-5,
  \"outcome\": \"End state in one sentence\",
  \"efficiency_metrics\": {
    \"total_turns\": 0,
    \"duration_minutes\": 0,
    \"corrections_count\": 0,
    \"avg_turns_per_goal\": 0.0,
    \"unnecessary_loops\": 0
  },
  \"best_practices_checklist\": [
    {\"practice\": \"Clear, specific instructions\", \"score\": \"yes|partial|no|n/a\", \"note\": \"Evidence from session\"},
    {\"practice\": \"Sufficient context provided\", \"score\": \"...\", \"note\": \"...\"},
    {\"practice\": \"Complex tasks broken into steps\", \"score\": \"...\", \"note\": \"...\"},
    {\"practice\": \"Effective use of skills and slash commands\", \"score\": \"...\", \"note\": \"...\"},
    {\"practice\": \"Used plan mode for complex tasks\", \"score\": \"...\", \"note\": \"...\"},
    {\"practice\": \"AI stayed on track without frequent correction\", \"score\": \"...\", \"note\": \"...\"},
    {\"practice\": \"Minimal unnecessary back-and-forth\", \"score\": \"...\", \"note\": \"...\"},
    {\"practice\": \"Unambiguous instructions\", \"score\": \"...\", \"note\": \"...\"},
    {\"practice\": \"Appropriate session scope\", \"score\": \"...\", \"note\": \"...\"},
    {\"practice\": \"Provided examples when needed\", \"score\": \"...\", \"note\": \"...\"}
  ],
  \"improvement_hints\": \"Specific actionable advice for THIS session. Reference actual prompts or patterns observed. null if the session was exemplary\",
  \"error_analysis\": \"Tool-level errors (infrastructure/transient issues only), or null\",
  \"skill_assessment\": \"Only if skills were used: did they help? null if no skills used\",
  \"skill_scores\": {\"skill-name\": 1-5, ...},
  \"recommendations\": \"One specific, actionable suggestion — or null if score is 5\",
  \"automation_ratio\": 0.0,
  \"plan_mode_used\": false,
  \"client_surface\": \"cli\"
}

CATEGORY rules — choose the SINGLE best category:
- feature = building something new that didn't exist before
- bugfix = fixing broken behavior or errors
- refactoring = restructuring code without changing behavior
- techdebt = addressing accumulated shortcuts, maintenance, or cleanup
- documentation = writing or updating documentation
- discovery = exploring, researching, prototyping, or investigating
- testing = writing or running tests
- deployment = CI/CD, deployment, infrastructure changes
- configuration = config changes, environment setup
- design = architecture or design planning
- review = code review, PR review, audit
- other = doesn't fit any of the above

Tags from: coding, research, debugging, shell, exploration, refactoring, documentation, deployment, testing, configuration, design, review

Quality scoring (goal-focused):
5 = All user goals achieved
4 = Primary goals achieved, minor goals incomplete
3 = Some goals met, some not
2 = Primary goals not achieved
1 = Session abandoned or no meaningful progress

Skill scoring (per-skill effectiveness):
5 = Skill was essential and performed perfectly
4 = Skill helped significantly, minor issues
3 = Skill contributed but wasn't decisive
2 = Skill was used but didn't help much
1 = Skill was counterproductive or irrelevant

EFFICIENCY METRICS rules:
- total_turns = number of user prompts in the session (from SESSION data)
- duration_minutes = session length (from SESSION TIMING data)
- corrections_count = prompts where user corrected, redirected, or undid AI work. Indicators: \"no\", \"wrong\", \"that's not\", \"actually\", \"I said\", \"try again\", \"undo\", \"revert\", or any re-statement of a prior instruction
- avg_turns_per_goal = total_turns divided by number of distinct goals
- unnecessary_loops = sequences where the same goal was attempted multiple times due to miscommunication

BEST PRACTICES scoring:
- yes = user clearly followed this practice (evidence in prompts)
- partial = some evidence but inconsistent
- no = clear evidence of not following this practice
- n/a = not applicable to this session (e.g. plan mode for a 2-prompt session)
- note MUST reference specific evidence from the session prompts. Never use generic notes

AUTOMATION & SURFACE rules:
- automation_ratio = automated_actions / (user_prompts + automated_actions). 0.0 means fully manual, 1.0 means fully automated. Use SESSION METADATA if available, otherwise estimate from event patterns
- plan_mode_used = true if the session used plan mode (Mode=plan in SESSION METADATA, or user messages reference /plan or plan mode)
- client_surface = the client that created the session (from SESSION METADATA Client field). One of: cli, vscode, jetbrains, desktop, or unknown

RULES:
- The title describes what the user WANTED, not what tools were used
- Never mention specific tool names (Read, Edit, Bash, etc.) in any field except error_analysis
- goal_outcome_map must map each distinct user goal to its specific outcome
- outcomes must have 3-5 items. Each maps to a specific goal and states whether it was achieved
- skill_assessment is ONLY about whether invoked skills helped achieve goals
- skill_scores must map each skill name from SKILLS USED to a 1-5 score. null if no skills used
- recommendations must be null or genuinely useful. Never say \"add tests\" or \"improve documentation\"
- improvement_hints must be specific to THIS session, referencing actual prompts or patterns. Never give generic advice. null if quality_score is 5";

pub fn build_request_context(
    user_id: &UserId,
    session_id: &SessionId,
    jwt_token: &str,
) -> RequestContext {
    RequestContext::new(
        SessionId::new(session_id.as_str()),
        TraceId::new(uuid::Uuid::new_v4().to_string()),
        ContextId::new(""),
        AgentName::new("hook-summary"),
    )
    .with_user_id(user_id.clone())
    .with_auth_token(jwt_token)
    .with_user_type(UserType::User)
}

pub async fn generate_session_analysis(
    ai_service: &Arc<AiService>,
    ctx: &RequestContext,
    last_message: &str,
    analysis_context: &str,
) -> Option<SessionAnalysis> {
    let user_prompt = if analysis_context.is_empty() {
        format!("Session final message:\n\n{last_message}")
    } else {
        format!("{analysis_context}\n\nSession final message:\n\n{last_message}")
    };

    let messages = vec![
        AiMessage::system(SYSTEM_PROMPT),
        AiMessage::user(&user_prompt),
    ];

    let request = AiRequest::builder(
        messages,
        ai_service.default_provider(),
        ai_service.default_model(),
        65536,
        ctx.clone(),
    )
    .with_structured_output(StructuredOutputOptions::with_schema(
        session_analysis_schema(),
    ))
    .build();

    let response = ai_service
        .generate(&request)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "Failed to generate AI session analysis");
        })
        .ok()?;

    serde_json::from_str::<SessionAnalysis>(&response.content)
        .map(validate_analysis)
        .map_err(|e| {
            tracing::warn!(error = %e, "Failed to parse AI session analysis response");
        })
        .ok()
}

pub async fn run_analysis_for_session(
    pool: &sqlx::PgPool,
    ai_service: &Arc<AiService>,
    user_id: &UserId,
    session_id: &SessionId,
    jwt_token: &str,
    direct_message: Option<&str>,
) -> Option<SessionAnalysis> {
    let ctx = build_request_context(user_id, session_id, jwt_token);
    let analysis_context = ai_context::gather_analysis_context(pool, user_id, session_id).await;
    let events_ctx = session_summary::generate_session_summary(pool, user_id, session_id).await;

    let full_context = ai_context::build_full_context(&analysis_context, events_ctx.as_ref());
    let last_msg =
        ai_context::resolve_last_message(pool, user_id, session_id, direct_message).await;

    let msg = if last_msg.is_empty() {
        "Session completed."
    } else {
        &last_msg
    };

    if let Some(analysis) = generate_session_analysis(ai_service, &ctx, msg, &full_context).await {
        session_analyses::insert_session_analysis(
            pool,
            session_id.as_str(),
            user_id.as_str(),
            &analysis,
        )
        .await;
        usage_aggregations::update_session_ai_summary_structured(pool, session_id, &analysis).await;
        Some(analysis)
    } else {
        tracing::warn!(
            session_id = session_id.as_str(),
            user_id = user_id.as_str(),
            context_len = full_context.len(),
            message_len = msg.len(),
            "AI session analysis returned None — generation or parsing failed"
        );
        None
    }
}
