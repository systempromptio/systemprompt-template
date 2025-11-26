use crate::repository::SkillRepository;
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_core_system::CONTEXT_BROADCASTER;
use systemprompt_identifiers::SkillId;
use systemprompt_models::execution::context::RequestContext;
use systemprompt_models::execution::events::BroadcastEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub skill_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillExecutionContext {
    context_id: String,
    task_id: Option<String>,
    agent_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillRequestContext {
    execution: SkillExecutionContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillLoadedData {
    skill_id: String,
    skill_name: String,
    description: String,
    instructions: String,
    task_id: Option<String>,
    request_context: SkillRequestContext,
}

#[derive(Debug, Clone)]
pub struct SkillService {
    skill_repo: Arc<SkillRepository>,
    db_pool: DbPool,
}

impl SkillService {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            skill_repo: Arc::new(SkillRepository::new(db_pool.clone())),
            db_pool,
        }
    }

    pub async fn load_skill(&self, skill_id: &str, ctx: &RequestContext) -> Result<String> {
        let logger = LogService::new(self.db_pool.clone(), ctx.log_context());
        let skill_id_typed = SkillId::new(skill_id);

        let skill = self.skill_repo
            .get_by_skill_id(&skill_id_typed)
            .await?
            .ok_or_else(|| {
                anyhow!(
                    "Skill not found in database: {} (ensure skill is synced via SkillIngestionService)",
                    skill_id
                )
            })?;

        logger
            .log(
                LogLevel::Info,
                "skill_service",
                &format!("Loaded skill: {}", skill.skill_id),
                Some(json!({
                    "skill_id": skill.skill_id,
                    "skill_name": skill.name,
                    "context_id": ctx.context_id().as_str(),
                    "task_id": ctx.task_id().map(|t| t.as_str()),
                    "user_id": ctx.user_id().as_str(),
                    "agent_name": ctx.agent_name().as_str()
                })),
            )
            .await
            .ok();

        let skill_data = SkillLoadedData {
            skill_id: skill.skill_id.as_str().to_string(),
            skill_name: skill.name.clone(),
            description: skill.description.clone(),
            instructions: skill.instructions.clone(),
            task_id: ctx.task_id().map(|t| t.as_str().to_string()),
            request_context: SkillRequestContext {
                execution: SkillExecutionContext {
                    context_id: ctx.context_id().as_str().to_string(),
                    task_id: ctx.task_id().map(|t| t.as_str().to_string()),
                    agent_name: ctx.agent_name().as_str().to_string(),
                },
            },
        };

        let event = BroadcastEvent {
            event_type: "skill_loaded".to_string(),
            context_id: ctx.context_id().as_str().to_string(),
            user_id: ctx.user_id().as_str().to_string(),
            data: serde_json::to_value(&skill_data).unwrap_or_default(),
            timestamp: Utc::now(),
        };

        logger
            .log(
                LogLevel::Info,
                "skill_service",
                &format!("Broadcasting skill_loaded event: {}", skill.skill_id),
                Some(json!({
                    "event_type": "skill_loaded",
                    "skill_id": skill.skill_id,
                    "context_id": ctx.context_id().as_str(),
                    "task_id": ctx.task_id().map(|t| t.as_str()),
                    "user_id": ctx.user_id().as_str()
                })),
            )
            .await
            .ok();

        CONTEXT_BROADCASTER
            .broadcast_to_user(ctx.user_id().as_str(), event)
            .await;

        Ok(skill.instructions)
    }

    pub async fn load_skill_metadata(
        &self,
        skill_id: &str,
        ctx: &RequestContext,
    ) -> Result<SkillMetadata> {
        let logger = LogService::new(self.db_pool.clone(), ctx.log_context());
        let skill_id_typed = SkillId::new(skill_id);

        let skill = self.skill_repo
            .get_by_skill_id(&skill_id_typed)
            .await?
            .ok_or_else(|| {
                anyhow!(
                    "Skill not found in database: {} (ensure skill is synced via SkillIngestionService)",
                    skill_id
                )
            })?;

        logger
            .log(
                LogLevel::Info,
                "skill_service",
                &format!("Loaded skill metadata: {}", skill.skill_id),
                Some(json!({
                    "skill_id": skill.skill_id,
                    "skill_name": skill.name,
                    "context_id": ctx.context_id().as_str(),
                    "task_id": ctx.task_id().map(|t| t.as_str()),
                })),
            )
            .await
            .ok();

        Ok(SkillMetadata {
            skill_id: skill.skill_id.as_str().to_string(),
            name: skill.name,
        })
    }
}
