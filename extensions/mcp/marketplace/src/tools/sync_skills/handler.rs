use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{McpExecutionId, UserId};
use systemprompt::mcp::McpError;
use systemprompt::mcp::{McpToolHandler, ProgressCallback};
use systemprompt::models::artifacts::TextArtifact;
use systemprompt::models::execution::context::RequestContext;

use crate::tools::shared;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SyncSkillsInput {
    #[serde(default)]
    pub skill_ids: Vec<String>,
}

pub struct SyncSkillsHandler {
    pub db_pool: DbPool,
    pub progress: Option<ProgressCallback>,
}

impl std::fmt::Debug for SyncSkillsHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncSkillsHandler")
            .field("db_pool", &self.db_pool)
            .field("progress", &self.progress.as_ref().map(|_| "..."))
            .finish()
    }
}

#[async_trait]
impl McpToolHandler for SyncSkillsHandler {
    type Input = SyncSkillsInput;
    type Output = TextArtifact;

    fn tool_name(&self) -> &'static str {
        "sync_skills"
    }

    fn description(&self) -> &'static str {
        "Sync user skills to the server. Optionally provide specific skill_ids \
         to sync, or sync all skills if none specified."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        if let Some(ref notify) = self.progress {
            notify(0.0, Some(100.0), Some("Starting skill sync...".to_string())).await;
        }

        let user_id = UserId::new(ctx.user_id().to_string());
        let pool = shared::require_write_pool(&self.db_pool)?;

        let skills =
            systemprompt_web_extension::admin::repositories::user_skills::list_user_skills(
                &pool, &user_id,
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to list skills: {e}"), None))?;

        if let Some(ref notify) = self.progress {
            notify(30.0, Some(100.0), Some("Computing changes...".to_string())).await;
        }

        let skills_to_sync = if input.skill_ids.is_empty() {
            skills
        } else {
            skills
                .into_iter()
                .filter(|s| input.skill_ids.iter().any(|id| id == s.skill_id.as_ref()))
                .collect()
        };

        shared::invalidate_marketplace_cache(&pool, &user_id).await;

        if let Some(ref notify) = self.progress {
            notify(70.0, Some(100.0), Some("Applying sync...".to_string())).await;
        }

        let skills_synced = skills_to_sync.len();
        let timestamp = chrono::Utc::now().to_rfc3339();

        let synced_skill_names: Vec<String> = skills_to_sync
            .iter()
            .map(|s| format!("{} ({})", s.name, s.skill_id))
            .collect();

        let summary_detail = if synced_skill_names.is_empty() {
            "No skills matched for sync.".to_string()
        } else {
            format!("Synced skills:\n- {}", synced_skill_names.join("\n- "))
        };

        if let Some(ref notify) = self.progress {
            notify(100.0, Some(100.0), None).await;
        }

        let result_json = serde_json::to_string_pretty(&serde_json::json!({
            "_display": { "type": "confirmation", "action": "synced" },
            "status": "success",
            "skills_synced": skills_synced,
            "timestamp": timestamp,
            "details": summary_detail,
        }))
        .map_err(|e| McpError::internal_error(format!("Failed to serialize result: {e}"), None))?;

        let summary = format!("Synced {skills_synced} skill(s) for user '{user_id}'");
        let content = format!("{summary}\n\n{result_json}");
        let artifact = TextArtifact::new(&result_json, ctx).with_title("Skill Sync Result");

        Ok((artifact, content))
    }
}
