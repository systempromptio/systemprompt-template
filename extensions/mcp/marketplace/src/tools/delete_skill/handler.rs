use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{McpExecutionId, SkillId, UserId};
use systemprompt::mcp::McpError;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::TextArtifact;
use systemprompt::models::execution::context::RequestContext;

use crate::tools::shared;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteSkillInput {
    pub skill_id: String,
}

#[derive(Debug)]
pub struct DeleteSkillHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for DeleteSkillHandler {
    type Input = DeleteSkillInput;
    type Output = TextArtifact;

    fn tool_name(&self) -> &'static str {
        "delete_skill"
    }

    fn description(&self) -> &'static str {
        "Delete a user skill. Requires skill_id (the slug, e.g. 'my-skill-name'). \
         Returns whether the skill was successfully deleted."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let pool = shared::require_write_pool(&self.db_pool)?;

        let user_id = UserId::new(ctx.user_id().to_string());
        let skill_id = SkillId::new(input.skill_id.clone());
        let deleted =
            systemprompt_web_extension::admin::repositories::user_skills::delete_user_skill(
                &pool, &user_id, &skill_id,
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to delete skill: {e}"), None))?;

        if !deleted {
            return Err(McpError::invalid_params(
                format!(
                    "Skill '{}' not found or does not belong to you",
                    input.skill_id
                ),
                None,
            ));
        }

        shared::invalidate_marketplace_cache(&pool, &user_id).await;

        let result_json = serde_json::to_string_pretty(&serde_json::json!({
            "_display": { "type": "confirmation", "action": "deleted" },
            "deleted": true,
            "skill_id": input.skill_id,
        }))
        .map_err(|e| McpError::internal_error(format!("Failed to serialize result: {e}"), None))?;

        let summary = format!("Deleted skill '{}'", input.skill_id);
        let content = format!("{summary}\n\n{result_json}");
        let artifact = TextArtifact::new(&result_json, ctx).with_title("Skill Deleted");

        Ok((artifact, content))
    }
}
