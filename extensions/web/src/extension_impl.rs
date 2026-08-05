//! `Extension` trait implementation for `WebExtension`.
//!
//! The wiring seam: every provider, prerenderer, renderer, schema, migration
//! and job the five web sibling crates expose is advertised to the host runtime
//! here.

use std::sync::Arc;

use systemprompt::extension::prelude::*;
use systemprompt::traits::Job;

use crate::assets::web_assets;
use crate::features::FeaturePagePrerenderer;
use crate::homepage::{HomepagePageDataProvider, HomepagePrerenderer};
use crate::navigation::NavigationPageDataProvider;
use crate::schemas::{migrations, schema_definitions};
use systemprompt_web_site::config_loader;

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
            let branding = config_loader::branding_config();
            providers.push(Arc::new(
                NavigationPageDataProvider::new(nav_config).with_branding(branding),
            ));
        }

        if let Some(homepage_config) = Self::homepage_config() {
            providers.push(Arc::new(HomepagePageDataProvider::new(homepage_config)));
        }

        providers.extend(crate::shared::registry::page_data_providers());
        providers
    }

    fn content_data_providers(&self) -> Vec<Arc<dyn ContentDataProvider>> {
        crate::shared::registry::content_data_providers()
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
        crate::shared::registry::component_renderers()
    }

    fn template_data_extenders(&self) -> Vec<Arc<dyn TemplateDataExtender>> {
        crate::shared::registry::template_data_extenders()
    }

    fn schemas(&self) -> Vec<SchemaDefinition> {
        schema_definitions()
    }

    fn migrations(&self) -> Vec<Migration> {
        migrations()
    }

    fn seeds(&self) -> Vec<Seed> {
        vec![Seed::new(
            "admin_oauth_client",
            include_str!("../schema/seeds/admin_oauth_client.sql"),
        )]
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["content", "users", "authz"]
    }

    fn cross_extension_tables(&self) -> Vec<&'static str> {
        vec![
            "markdown_content",
            "users",
            "eval_runs",
            "eval_cases",
            "eval_results",
            "eval_pairs",
            "eval_judge_calls",
            "eval_rubrics",
        ]
    }

    fn router(&self, ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
        crate::router::build(ctx)
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
        crate::jobs::extension_jobs()
    }

    fn priority(&self) -> u32 {
        100
    }

    fn config_prefix(&self) -> Option<&str> {
        Some(Self::PREFIX)
    }

    fn declares_assets(&self) -> bool {
        true
    }

    fn required_assets(&self, paths: &dyn AssetPaths) -> Vec<AssetDefinition> {
        let mut assets = web_assets(paths);
        assets.extend(crate::admin::assets::admin_assets(paths));
        assets
    }
}
