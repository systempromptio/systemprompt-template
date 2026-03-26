use std::sync::Arc;

use axum::Router;

use systemprompt::database::Database;
use systemprompt::extension::prelude::*;
use systemprompt::traits::Job;

use crate::blog::{BlogListPageDataProvider, BlogPostPageDataProvider};
use crate::config_loader;
use crate::docs::{DocsContentDataProvider, DocsPageDataProvider};
use crate::extenders::OrgUrlExtender;
use crate::features::FeaturePagePrerenderer;
use crate::homepage::{HomepagePageDataProvider, HomepagePrerenderer};
use crate::navigation::NavigationPageDataProvider;
use crate::partials::{
    AgenticMeshAnimationPartialRenderer, ArchitectureDiagramPartialRenderer,
    CliRemoteAnimationPartialRenderer, FooterPartialRenderer, HeadAssetsPartialRenderer,
    HeaderPartialRenderer, MemoryLoopAnimationPartialRenderer, RustMeshAnimationPartialRenderer,
    ScriptsPartialRenderer,
};
use crate::playbooks::{
    PlaybookPageDataProvider, PlaybooksContentDataProvider, PlaybooksListPageDataProvider,
};
use crate::{
    admin, api,
    assets::web_assets,
    jobs::{
        BundleAdminCssJob, BundleAdminJsJob, CompileAdminTemplatesJob,
        ContentAnalyticsAggregationJob, ContentIngestionJob, ContentPrerenderJob,
        CopyExtensionAssetsJob, GitHubMarketplaceSyncJob, LlmsTxtGenerationJob, MarketplaceSyncJob,
        PublishPipelineJob, RecalculateGamificationJob, RobotsTxtGenerationJob,
        SitemapGenerationJob,
    },
    schemas::schema_definitions,
};

use crate::extension::WebExtension;

impl Extension for WebExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "web",
            name: "Web Content & Navigation",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    fn page_data_providers(&self) -> Vec<Arc<dyn PageDataProvider>> {
        let mut providers: Vec<Arc<dyn PageDataProvider>> = vec![];

        if let Some(nav_config) = Self::navigation_config() {
            let branding = match config_loader::load_branding_config() {
                Ok(val) => val,
                Err(e) => {
                    tracing::error!(error = %e, "Branding config error");
                    None
                }
            };
            providers.push(Arc::new(
                NavigationPageDataProvider::new(nav_config).with_branding(branding),
            ));
        }

        if let Some(homepage_config) = Self::homepage_config() {
            providers.push(Arc::new(HomepagePageDataProvider::new(homepage_config)));
        }

        providers.extend([
            Arc::new(DocsPageDataProvider::new()) as Arc<dyn PageDataProvider>,
            Arc::new(BlogListPageDataProvider::new()),
            Arc::new(BlogPostPageDataProvider::new()),
            Arc::new(PlaybooksListPageDataProvider::new()),
            Arc::new(PlaybookPageDataProvider::new()),
        ]);
        providers
    }

    fn content_data_providers(&self) -> Vec<Arc<dyn ContentDataProvider>> {
        vec![
            Arc::new(DocsContentDataProvider::new()),
            Arc::new(PlaybooksContentDataProvider::new()),
        ]
    }

    fn page_prerenderers(&self) -> Vec<Arc<dyn PagePrerenderer>> {
        let mut prerenderers: Vec<Arc<dyn PagePrerenderer>> = vec![];

        if let Some(config) = Self::homepage_config() {
            prerenderers.push(Arc::new(HomepagePrerenderer::new(config)));
        }

        if let Some(config) = Self::features_config() {
            for page in &config.pages {
                prerenderers.push(Arc::new(FeaturePagePrerenderer::new(page.clone())));
            }
        }

        prerenderers
    }

    fn component_renderers(&self) -> Vec<Arc<dyn ComponentRenderer>> {
        vec![
            Arc::new(HeadAssetsPartialRenderer),
            Arc::new(HeaderPartialRenderer),
            Arc::new(FooterPartialRenderer),
            Arc::new(ScriptsPartialRenderer),
            Arc::new(CliRemoteAnimationPartialRenderer),
            Arc::new(RustMeshAnimationPartialRenderer),
            Arc::new(MemoryLoopAnimationPartialRenderer),
            Arc::new(AgenticMeshAnimationPartialRenderer),
            Arc::new(ArchitectureDiagramPartialRenderer),
        ]
    }

    fn template_data_extenders(&self) -> Vec<Arc<dyn TemplateDataExtender>> {
        vec![Arc::new(OrgUrlExtender::new())]
    }

    fn schemas(&self) -> Vec<SchemaDefinition> {
        schema_definitions()
    }

    fn router(&self, ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
        use axum::routing::post;

        let db_handle = ctx.database();
        let db = db_handle.as_any().downcast_ref::<Database>()?;
        let pool = db.pool()?;
        let write_pool = db.write_pool_arc().unwrap_or_else(|_| pool.clone());

        let admin_api = admin::admin_router(&pool, write_pool.clone());
        let webhook_api = admin::hooks_webhook_router(write_pool.clone());
        let secrets_api = admin::secrets_router(write_pool);
        let marketplace_git = admin::marketplace_git_router(pool.clone());
        let links_router = api::router(pool.clone(), self.validated_config.clone());

        let api_router = Router::new()
            .route(
                "/auth/session",
                post(api::auth::set_session).delete(api::auth::clear_session),
            )
            .merge(links_router)
            .merge(webhook_api)
            .merge(secrets_api)
            .merge(marketplace_git)
            .nest("/admin", admin_api);

        let admin_dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("storage")
            .join("files")
            .join("admin");
        let engine = match admin::templates::AdminTemplateEngine::new(&admin_dir) {
            Ok(engine) => engine,
            Err(e) => {
                tracing::error!(error = %e, "Failed to initialize admin template engine");
                return Some(ExtensionRouter::public(api_router, "/api/public"));
            }
        };
        let ssr_router = admin::admin_ssr_router(pool, engine);

        let combined = Router::new()
            .nest_service("/admin", ssr_router)
            .nest("/api/public", api_router);

        Some(ExtensionRouter::public(combined, "/"))
    }

    fn site_auth(&self) -> Option<SiteAuthConfig> {
        Some(SiteAuthConfig {
            login_path: "/admin/login",
            protected_prefixes: &["/admin"],
            public_prefixes: &["/admin/login", "/admin/add-passkey"],
            required_scope: "user",
        })
    }

    fn jobs(&self) -> Vec<Arc<dyn Job>> {
        vec![
            Arc::new(ContentIngestionJob),
            Arc::new(CopyExtensionAssetsJob),
            Arc::new(ContentPrerenderJob),
            Arc::new(SitemapGenerationJob),
            Arc::new(LlmsTxtGenerationJob),
            Arc::new(RobotsTxtGenerationJob),
            Arc::new(PublishPipelineJob),
            Arc::new(ContentAnalyticsAggregationJob),
            Arc::new(CompileAdminTemplatesJob),
            Arc::new(BundleAdminCssJob),
            Arc::new(BundleAdminJsJob),
            Arc::new(RecalculateGamificationJob),
            Arc::new(GitHubMarketplaceSyncJob),
            Arc::new(MarketplaceSyncJob),
        ]
    }

    fn priority(&self) -> u32 {
        100
    }

    fn migration_weight(&self) -> u32 {
        100
    }

    fn config_prefix(&self) -> Option<&str> {
        Some(Self::PREFIX)
    }

    fn declares_assets(&self) -> bool {
        true
    }

    fn required_assets(
        &self,
        paths: &dyn systemprompt::extension::AssetPaths,
    ) -> Vec<systemprompt::extension::AssetDefinition> {
        web_assets(paths)
    }
}
