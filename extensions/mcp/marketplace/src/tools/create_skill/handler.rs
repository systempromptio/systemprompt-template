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

const MAX_NAME_LEN: usize = 256;
const MAX_DESCRIPTION_LEN: usize = 4096;
const MAX_CONTENT_LEN: usize = 65536;
const MAX_TAG_LEN: usize = 128;
const MAX_TAGS_COUNT: usize = 50;

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
        if input.name.len() > MAX_NAME_LEN {
            return Err(McpError::invalid_params(
                format!("name exceeds maximum length of {MAX_NAME_LEN}"),
                None,
            ));
        }
        if input.description.len() > MAX_DESCRIPTION_LEN {
            return Err(McpError::invalid_params(
                format!("description exceeds maximum length of {MAX_DESCRIPTION_LEN}"),
                None,
            ));
        }
        if input.content.len() > MAX_CONTENT_LEN {
            return Err(McpError::invalid_params(
                format!("content exceeds maximum length of {MAX_CONTENT_LEN}"),
                None,
            ));
        }
        if input.tags.len() > MAX_TAGS_COUNT {
            return Err(McpError::invalid_params(
                format!("tags count exceeds maximum of {MAX_TAGS_COUNT}"),
                None,
            ));
        }
        for tag in &input.tags {
            if tag.len() > MAX_TAG_LEN {
                return Err(McpError::invalid_params(
                    format!("tag exceeds maximum length of {MAX_TAG_LEN}"),
                    None,
                ));
            }
        }

        let skill_id = shared::generate_slug(&input.name);

        let pool = shared::require_write_pool(&self.db_pool)?;
        let create_req = systemprompt_web_extension::admin::types::CreateSkillRequest {
            skill_id: systemprompt::identifiers::SkillId::new(skill_id.clone()),
            name: input.name.clone(),
            description: input.description.clone(),
            content: input.content.clone(),
            tags: input.tags,
            base_skill_id: input
                .base_skill_id
                .map(systemprompt::identifiers::SkillId::new),
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
