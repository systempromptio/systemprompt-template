use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{McpExecutionId, UserId};
use systemprompt::mcp::McpError;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::{Column, ColumnType, TableArtifact};
use systemprompt::models::execution::context::RequestContext;

#[derive(Deserialize, JsonSchema)]
pub struct ListSkillsInput {}

pub struct ListSkillsHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for ListSkillsHandler {
    type Input = ListSkillsInput;
    type Output = TableArtifact;

    fn tool_name(&self) -> &'static str {
        "list_skills"
    }

    fn description(&self) -> &'static str {
        "List all skills for the authenticated user. Returns skill metadata \
         including usage counts and ratings."
    }

    async fn handle(
        &self,
        _input: Self::Input,
        ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let pool = self.db_pool.pool().ok_or_else(|| {
            McpError::internal_error("Database pool not available".to_string(), None)
        })?;
        let user_id = UserId::new(ctx.user_id().to_string());

        let skills =
            systemprompt_web_extension::admin::repositories::user_skills::list_user_skills(
                &pool, &user_id,
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to list skills: {e}"), None))?;

        let skill_ids: Vec<systemprompt::identifiers::SkillId> =
            skills.iter().map(|s| s.skill_id.clone()).collect();

        let usage_counts =
            systemprompt_web_extension::admin::repositories::user_skills::fetch_skill_usage_counts(
                &pool, &skill_ids,
            )
            .await;

        let avg_ratings =
            systemprompt_web_extension::admin::repositories::user_skills::fetch_skill_avg_ratings(
                &pool, &skill_ids,
            )
            .await;

        let columns = vec![
            Column::new("skill_id", ColumnType::String),
            Column::new("name", ColumnType::String),
            Column::new("description", ColumnType::String),
            Column::new("version", ColumnType::String),
            Column::new("tags", ColumnType::String),
            Column::new("base_skill_id", ColumnType::String),
            Column::new("usage_count", ColumnType::Integer),
            Column::new("avg_rating", ColumnType::Number),
            Column::new("created_at", ColumnType::Date),
            Column::new("updated_at", ColumnType::Date),
        ];

        let rows: Vec<serde_json::Value> = skills
            .iter()
            .map(|s| {
                let usage = usage_counts.get(s.skill_id.as_ref()).copied().unwrap_or(0);
                let (avg_rating, _rating_count) =
                    avg_ratings.get(s.skill_id.as_ref()).copied().unwrap_or((0.0, 0));

                serde_json::json!({
                    "skill_id": s.skill_id,
                    "name": s.name,
                    "description": s.description,
                    "version": s.version,
                    "tags": s.tags,
                    "base_skill_id": s.base_skill_id,
                    "usage_count": usage,
                    "avg_rating": avg_rating,
                    "created_at": s.created_at.to_rfc3339(),
                    "updated_at": s.updated_at.to_rfc3339(),
                })
            })
            .collect();

        let total = rows.len();
        let summary = format!("Found {total} skill(s) for user '{user_id}'");
        let artifact = TableArtifact::new(columns, ctx).with_rows(rows);

        Ok((artifact, summary))
    }
}
