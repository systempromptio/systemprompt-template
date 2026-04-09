use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;
use systemprompt::agent::services::SkillService;
use systemprompt::ai::{AiMessage, AiRequest, AiService};
use systemprompt::database::DbPool;
use systemprompt::identifiers::{McpExecutionId, UserId};
use systemprompt::mcp::McpError;
use systemprompt::mcp::{McpToolHandler, ProgressCallback};
use systemprompt::models::artifacts::TextArtifact;
use systemprompt::models::execution::context::RequestContext;

use crate::tools::shared;

#[derive(Deserialize, JsonSchema)]
pub struct AnalyzeSkillInput {
    pub skill_id: String,
}

pub struct AnalyzeSkillHandler {
    pub db_pool: DbPool,
    pub ai_service: Arc<AiService>,
    pub skill_loader: Arc<SkillService>,
    pub progress: Option<ProgressCallback>,
}

#[async_trait]
impl McpToolHandler for AnalyzeSkillHandler {
    type Input = AnalyzeSkillInput;
    type Output = TextArtifact;

    fn tool_name(&self) -> &'static str {
        "analyze_skill"
    }

    fn description(&self) -> &'static str {
        "Analyze a skill using AI to assess quality, clarity, and best practices. \
         Returns improvement suggestions and a quality assessment. Requires skill_id."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        if let Some(ref notify) = self.progress {
            notify(0.0, Some(100.0), Some("Loading skill...".to_string())).await;
        }

        let user_id = UserId::new(ctx.user_id().to_string());
        let pool = shared::require_pool(&self.db_pool)?;

        let skill_content =
            match systemprompt_web_extension::admin::repositories::user_skills::list_user_skills(
                &pool, &user_id,
            )
            .await
            {
                Ok(skills) => skills
                    .into_iter()
                    .find(|s| s.skill_id.as_ref() == input.skill_id)
                    .map(|s| {
                        format!(
                            "Name: {}\nDescription: {}\nContent:\n{}",
                            s.name, s.description, s.content
                        )
                    }),
                Err(_) => None,
            };

        let skill_content = match skill_content {
            Some(content) => content,
            None => self
                .skill_loader
                .load_skill(&input.skill_id, ctx)
                .await
                .map_err(|e| {
                    McpError::internal_error(
                        format!(
                            "Skill '{}' not found in user skills or skill registry: {e}",
                            input.skill_id
                        ),
                        None,
                    )
                })?,
        };

        if let Some(ref notify) = self.progress {
            notify(20.0, Some(100.0), Some("Analyzing with AI...".to_string())).await;
        }

        let system_prompt = "You are an expert skill analyst for AI system prompts. \
            Analyze the provided skill and return a detailed assessment covering:\n\
            1. **Quality Score**: Rate the skill from 1-10 with justification.\n\
            2. **Clarity Analysis**: How clear and unambiguous are the instructions?\n\
            3. **Best Practices Check**: Does it follow prompt engineering best practices?\n\
            4. **Improvement Suggestions**: Specific, actionable improvements.\n\n\
            Format your response as a structured analysis with clear sections.";

        let user_prompt = format!(
            "Analyze the following skill:\n\n---\n{skill_content}\n---\n\n\
             Provide a comprehensive quality assessment, clarity analysis, \
             best practices evaluation, and improvement suggestions."
        );

        let messages = vec![
            AiMessage::system(system_prompt),
            AiMessage::user(&user_prompt),
        ];

        let request = AiRequest::builder(
            messages,
            self.ai_service.default_provider(),
            self.ai_service.default_model(),
            4096,
            ctx.clone(),
        )
        .build();

        let response = self.ai_service.generate(&request).await.map_err(|e| {
            McpError::internal_error(format!("Failed to analyze skill with AI: {e}"), None)
        })?;

        let analysis_text = response.content;

        if let Some(ref notify) = self.progress {
            notify(80.0, Some(100.0), Some("Building results...".to_string())).await;
        }

        let artifact = TextArtifact::new(&analysis_text, ctx)
            .with_title(format!("Skill Analysis: {}", input.skill_id));

        if let Some(ref notify) = self.progress {
            notify(100.0, Some(100.0), Some("Analysis complete".to_string())).await;
        }

        let summary = format!("Analysis complete for skill '{}'.", input.skill_id);
        let content = format!("{summary}\n\n{analysis_text}");

        Ok((artifact, content))
    }
}
