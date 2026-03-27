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

#[derive(Deserialize, JsonSchema)]
pub struct UpdateSkillInput {
    pub skill_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
}

pub struct UpdateSkillHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for UpdateSkillHandler {
    type Input = UpdateSkillInput;
    type Output = TextArtifact;

    fn tool_name(&self) -> &'static str {
        "update_skill"
    }

    fn description(&self) -> &'static str {
        "Update an existing user skill. Requires skill_id. All other fields \
         (name, description, content, tags) are optional - only provided \
         fields will be updated."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let pool = self.db_pool.write_pool().ok_or_else(|| {
            McpError::internal_error("Database pool not available".to_string(), None)
        })?;
        let update_req = systemprompt_web_extension::admin::types::UpdateUserSkillRequest {
            name: input.name,
            description: input.description,
            content: input.content,
            tags: input.tags,
        };

        let user_id = UserId::new(ctx.user_id().to_string());
        let skill_id = SkillId::new(&input.skill_id);
        let skill =
            systemprompt_web_extension::admin::repositories::user_skills::update_user_skill(
                &pool,
                &user_id,
                &skill_id,
                &update_req,
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to update skill: {e}"), None))?
            .ok_or_else(|| {
                McpError::invalid_params(
                    format!(
                        "Skill '{}' not found or does not belong to you",
                        input.skill_id
                    ),
                    None,
                )
            })?;

        shared::invalidate_marketplace_cache(&pool, &user_id).await;

        let skill_json = serde_json::to_string_pretty(&serde_json::json!({
            "_display": { "type": "card", "entity": "skill", "action": "updated" },
            "skill_id": skill.skill_id,
            "name": skill.name,
            "description": skill.description,
            "content": skill.content,
            "version": skill.version,
            "tags": skill.tags,
            "base_skill_id": skill.base_skill_id,
            "created_at": skill.created_at.to_rfc3339(),
            "updated_at": skill.updated_at.to_rfc3339(),
        }))
        .unwrap_or_default();

        let summary = format!("Updated skill '{}' ({})", skill.name, skill.skill_id);
        let content = format!("{summary}\n\n{skill_json}");
        let artifact =
            TextArtifact::new(&skill_json, ctx).with_title(format!("Skill: {}", skill.name));

        Ok((artifact, content))
    }
}
