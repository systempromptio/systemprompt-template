use anyhow::{Context, Result};
use std::sync::Arc;
use systemprompt::agent::services::SkillService;
use systemprompt::ai::{AiService, NoopToolProvider};
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpServerId;
use systemprompt::loader::ConfigLoader;
use systemprompt::mcp::repository::ToolUsageRepository;
use systemprompt::mcp::{McpArtifactRepository, McpToolExecutor};
use systemprompt::system::AppContext;

#[derive(Clone, Debug)]
pub struct MarketplaceServer {
    pub db_pool: DbPool,
    pub service_id: McpServerId,
    pub ai_service: Arc<AiService>,
    pub skill_loader: Arc<SkillService>,
    pub executor: McpToolExecutor,
}

impl MarketplaceServer {
    pub fn new(db_pool: DbPool, service_id: McpServerId, _ctx: Arc<AppContext>) -> Result<Self> {
        let services_config = ConfigLoader::load()?;

        let tool_provider = Arc::new(NoopToolProvider::new());
        let ai_service = Arc::new(
            AiService::new(
                DbPool::clone(&db_pool),
                &services_config.ai,
                tool_provider,
                None,
            )
            .context("Failed to initialize AiService")?,
        );

        let skill_loader = Arc::new(SkillService::new(&db_pool)?);
        let tool_usage_repo = Arc::new(
            ToolUsageRepository::new(&db_pool)
                .context("Failed to initialize ToolUsageRepository")?,
        );
        let artifact_repo = Arc::new(
            McpArtifactRepository::new(&db_pool)
                .context("Failed to initialize McpArtifactRepository")?,
        );
        let executor = McpToolExecutor::new(tool_usage_repo, artifact_repo, "skill-manager");

        Ok(Self {
            db_pool,
            service_id,
            ai_service,
            skill_loader,
            executor,
        })
    }
}
