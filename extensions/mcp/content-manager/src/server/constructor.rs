use anyhow::{Context, Result};
use std::sync::Arc;
use systemprompt::agent::repository::content::ArtifactRepository;
use systemprompt::agent::services::SkillService;
use systemprompt::ai::services::providers::ImageProviderFactory;
use systemprompt::ai::{AiService, ImageService, NoopToolProvider, StorageConfig};
use systemprompt::database::DbPool;
use systemprompt::files::{FilesAiPersistenceProvider, FilesConfig};
use systemprompt::identifiers::McpServerId;
use systemprompt::loader::EnhancedConfigLoader;
use systemprompt::mcp::repository::ToolUsageRepository;
use systemprompt::system::AppContext;

#[derive(Clone)]
pub struct ContentManagerServer {
    pub db_pool: DbPool,
    pub service_id: McpServerId,
    pub ai_service: Arc<AiService>,
    pub image_service: Arc<ImageService>,
    pub skill_loader: Arc<SkillService>,
    pub artifact_repo: ArtifactRepository,
    pub tool_usage_repo: Arc<ToolUsageRepository>,
}

impl ContentManagerServer {
    #[allow(clippy::unused_async)]
    pub async fn new(
        db_pool: DbPool,
        service_id: McpServerId,
        _ctx: Arc<AppContext>,
    ) -> Result<Self> {
        let config_loader = EnhancedConfigLoader::from_env()?;
        let services_config = config_loader.load()?;

        let tool_provider = Arc::new(NoopToolProvider::new());
        let ai_service = Arc::new(
            AiService::new(db_pool.clone(), &services_config.ai, tool_provider, None)
                .context("Failed to initialize AiService")?,
        );

        // Initialize ImageService for image generation
        // Use FilesConfig for validated storage paths
        FilesConfig::init().context("Failed to initialize FilesConfig")?;
        let files_config = FilesConfig::get().context("Failed to get FilesConfig")?;

        let file_provider = Arc::new(
            FilesAiPersistenceProvider::new(&db_pool)
                .context("Failed to initialize FilesAiPersistenceProvider")?,
        );
        let storage_config = StorageConfig::new(
            files_config.generated_images(),
            format!("{}/images/generated", files_config.url_prefix()),
        );

        // Create image providers from all enabled configs that support image generation
        let image_providers = ImageProviderFactory::create_all(&services_config.ai.providers)
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "No image providers available");
                std::collections::HashMap::new()
            });

        // Determine default image provider:
        // 1. If default_provider supports images and was created, use it
        // 2. Otherwise use first available image provider
        let default_image_provider = {
            let default = &services_config.ai.default_provider;
            if ImageProviderFactory::supports_image_generation(default)
                && image_providers.contains_key(default)
            {
                Some(default.clone())
            } else {
                image_providers.keys().next().cloned()
            }
        };

        if image_providers.is_empty() {
            tracing::warn!("No image providers available - image generation will fail");
        } else {
            tracing::info!(
                providers = ?image_providers.keys().collect::<Vec<_>>(),
                default = ?default_image_provider,
                "Image providers initialized"
            );
        }

        let image_service = Arc::new(
            ImageService::with_providers(
                &db_pool,
                storage_config,
                file_provider,
                image_providers,
                default_image_provider,
            )
            .context("Failed to initialize ImageService")?,
        );

        let skill_loader = Arc::new(SkillService::new(db_pool.clone()));
        let artifact_repo = ArtifactRepository::new(db_pool.clone());
        let tool_usage_repo = Arc::new(
            ToolUsageRepository::new(&db_pool)
                .context("Failed to initialize ToolUsageRepository")?,
        );

        Ok(Self {
            db_pool,
            service_id,
            ai_service,
            image_service,
            skill_loader,
            artifact_repo,
            tool_usage_repo,
        })
    }
}
