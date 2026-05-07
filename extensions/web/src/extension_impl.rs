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
use crate::{
    admin, api,
    assets::web_assets,
    jobs::{
        BundleAdminCssJob, BundleAdminJsJob, ContentAnalyticsAggregationJob, ContentIngestionJob,
        ContentPrerenderJob, CopyExtensionAssetsJob, LlmsTxtGenerationJob, PublishPipelineJob,
        RobotsTxtGenerationJob, SitemapGenerationJob,
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

        let docs_provider: Arc<dyn PageDataProvider> = Arc::new(DocsPageDataProvider::new());
        providers.extend([
            docs_provider,
            Arc::new(BlogListPageDataProvider::new()),
            Arc::new(BlogPostPageDataProvider::new()),
        ]);
        providers
    }

    fn content_data_providers(&self) -> Vec<Arc<dyn ContentDataProvider>> {
        vec![Arc::new(DocsContentDataProvider::new())]
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
        let write_pool = db.write_pool_arc().unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to get write pool, falling back to read pool");
            Arc::clone(&pool)
        });

        let admin_api = admin::admin_router(Arc::clone(&pool));
        let webhook_api = admin::hooks_webhook_router(Arc::clone(&write_pool));
        let secrets_api = admin::secrets_router(Arc::clone(&write_pool));
        let cowork_api = admin::cowork_router(Arc::clone(&pool));
        let share_api = admin::share_manifest_router(Arc::clone(&pool));
        let links_router = api::router(Arc::clone(&pool), self.validated_config.clone());

        let api_router = Router::new()
            .route(
                "/auth/session",
                post(api::auth::set_session).delete(api::auth::clear_session),
            )
            .merge(links_router)
            .merge(webhook_api)
            .merge(secrets_api)
            .nest("/admin", admin_api);

        let admin_dir = std::env::current_dir()
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Failed to get current directory, using fallback");
                std::path::PathBuf::from(".")
            })
            .join("storage")
            .join("files")
            .join("admin");
        let branding = match config_loader::load_branding_config() {
            Ok(val) => val,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to load branding config for admin");
                None
            }
        };
        let engine = match admin::templates::AdminTemplateEngine::new(&admin_dir) {
            Ok(engine) => engine.with_branding(branding),
            Err(e) => {
                tracing::error!(error = %e, "Failed to initialize admin template engine");
                return Some(ExtensionRouter::public(api_router, "/api/public"));
            }
        };
        let cowork_auth_router = admin::cowork_auth_ssr_router(Arc::clone(&pool), engine.clone());
        let ssr_router = admin::admin_ssr_router(pool, engine);

        let combined = Router::new()
            .nest_service("/admin", ssr_router)
            .nest_service("/cowork-auth", cowork_auth_router)
            .merge(cowork_api)
            .merge(share_api)
            .nest("/api/public", api_router);

        Some(ExtensionRouter::public(combined, "/"))
    }

    fn site_auth(&self) -> Option<SiteAuthConfig> {
        Some(SiteAuthConfig {
            login_path: "/admin/login",
            protected_prefixes: &["/admin", "/cowork-auth"],
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
            Arc::new(BundleAdminCssJob),
            Arc::new(BundleAdminJsJob),
        ]
    }

    fn priority(&self) -> u32 {
        100
    }

    fn migration_weight(&self) -> u32 {
        1000
    }

    fn config_prefix(&self) -> Option<&str> {
        Some(Self::PREFIX)
    }

    fn declares_assets(&self) -> bool {
        true
    }

    fn required_assets(&self, paths: &dyn AssetPaths) -> Vec<AssetDefinition> {
        web_assets(paths)
    }
}

