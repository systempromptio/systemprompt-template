use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{McpExecutionId, UserId};
use systemprompt::mcp::McpError;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::TextArtifact;
use systemprompt::models::execution::context::RequestContext;

use crate::tools::shared;

#[derive(Deserialize, JsonSchema)]
pub struct CreateSkillInput {
    pub name: String,
    pub description: String,
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub base_skill_id: Option<String>,
    pub target_plugin_id: Option<String>,
}

pub struct CreateSkillHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for CreateSkillHandler {
    type Input = CreateSkillInput;
    type Output = TextArtifact;

    fn tool_name(&self) -> &'static str {
        "create_skill"
    }

    fn description(&self) -> &'static str {
        "Create a new user skill. Requires name, description, and content. \
         Optionally provide tags and base_skill_id to fork from an existing skill."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let skill_id = shared::generate_slug(&input.name);

        let pool = self.db_pool.write_pool().ok_or_else(|| {
            McpError::internal_error("Database pool not available".to_string(), None)
        })?;
        let create_req = systemprompt_web_extension::admin::types::CreateSkillRequest {
            skill_id: systemprompt::identifiers::SkillId::new(skill_id.clone()),
            name: input.name.clone(),
            description: input.description.clone(),
            content: input.content.clone(),
            tags: input.tags,
            base_skill_id: input.base_skill_id.map(systemprompt::identifiers::SkillId::new),
        };

        let user_id = UserId::new(ctx.user_id().to_string());
        let skill =
            systemprompt_web_extension::admin::repositories::user_skills::create_user_skill(
                &pool,
                &user_id,
                &create_req,
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to create skill: {e}"), None))?;

        let added_to_plugin = shared::add_to_plugin(
            &self.db_pool,
            &user_id,
            &skill.id,
            "skill",
            input.target_plugin_id.as_deref(),
        )
        .await;

        shared::invalidate_marketplace_cache(&pool, &user_id).await;

        let skill_json = serde_json::to_string_pretty(&serde_json::json!({
            "_display": { "type": "card", "entity": "skill", "action": "created" },
            "skill_id": skill.skill_id,
            "name": skill.name,
            "description": skill.description,
            "content": skill.content,
            "enabled": skill.enabled,
            "version": skill.version,
            "tags": skill.tags,
            "base_skill_id": skill.base_skill_id,
            "added_to_plugin": added_to_plugin,
            "created_at": skill.created_at.to_rfc3339(),
            "updated_at": skill.updated_at.to_rfc3339(),
        }))
        .map_err(|e| McpError::internal_error(format!("Failed to serialize skill: {e}"), None))?;

        let summary = if let Some(ref plugin_id) = added_to_plugin {
            format!(
                "Created skill '{}' ({}) and added to plugin '{}'",
                skill.name, skill.skill_id, plugin_id
            )
        } else {
            format!("Created skill '{}' ({})", skill.name, skill.skill_id)
        };
        let content = format!("{summary}\n\n{skill_json}");
        let artifact =
            TextArtifact::new(&skill_json, ctx).with_title(format!("Skill: {}", skill.name));

        Ok((artifact, content))
    }
}
