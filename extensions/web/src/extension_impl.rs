//! `Extension` implementation for `WebExtension`.
//!
//! The wiring seam: every schema, migration, seed, job, subject provider,
//! gateway guard, content provider, provided value and the route set the five
//! web sibling crates expose is contributed here, once, by `contribute`.

use std::sync::Arc;

use systemprompt::config::AppPaths;
use systemprompt::content::contract::ContentSlot;
use systemprompt::extension::artifact_ui::DynArtifactUiRenderer;
use systemprompt::extension::prelude::*;
use systemprompt::extension::reserved::InstallConferral;
use systemprompt::governance::authz::SubjectSourceCell;
use systemprompt::identifiers::SecretName;
use systemprompt::manifest::ConfigView;
use systemprompt::manifest::profile::ProfileView;
use systemprompt::manifest::services::InstallServicesView;
use systemprompt::models::auth::DynRequestAuthenticator;
use systemprompt_web_admin::env::{AdminEnv, AdminEnvParts};
use systemprompt_web_admin::gateway_entitlement::RouteEntitlementGuard;
use systemprompt_web_site::partials::{
    FooterPartialRenderer, HeadAssetsPartialRenderer, HeaderPartialRenderer, ScriptsPartialRenderer,
};
use systemprompt_web_site::partials_animations::{
    AgenticMeshAnimationPartialRenderer, ArchitectureDiagramPartialRenderer,
    CliRemoteAnimationPartialRenderer, MemoryLoopAnimationPartialRenderer,
    RustMeshAnimationPartialRenderer,
};

use crate::assets::web_assets;
use crate::extension::WebExtension;
use crate::homepage::{HomepagePageDataProvider, HomepagePrerenderer};
use crate::navigation::NavigationPageDataProvider;
use crate::router::{DbHandles, WebRuntime, route_set};
use crate::schemas::{migrations, schema_definitions};
use crate::{
    DocsContentDataProvider, DocsPageDataProvider, OrgUrlExtender, ReleaseVersionExtender,
    SkillsPagePrerenderer, WebSettings, config_loader,
};

// Why: a generic connector's OAuth client is read from a secret the
// operator names in its `connector.client_id_secret`; only names under this
// prefix are declared, so the profile grants them explicitly.
const CONNECTOR_SECRET_PREFIX: &str = "web_connector_";

const CROSS_TABLES: [&str; 6] = [
    "managed_installation_receipts",
    "managed_publications",
    "managed_resources",
    "markdown_content",
    "mcp_tool_executions",
    "users",
];

impl Extension for WebExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: Self::ID,
            name: "Web Content & Navigation",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    fn contribute(&self, c: &mut Contributions<'_>) -> Result<(), ContributionError> {
        for schema in schema_definitions() {
            c.schema(schema);
        }
        for table in CROSS_TABLES {
            c.cross_table(table);
        }
        c.migrations(migrations())
            .seed(Seed::new(
                "admin_oauth_client",
                include_str!("../schema/seeds/admin_oauth_client.sql"),
            ))
            .seed(Seed::new(
                "marketplace_plans",
                include_str!("../schema/seeds/marketplace_plans.sql"),
            ))
            .seed(Seed::new(
                "groups_projects",
                include_str!("../schema/seeds/groups_projects.sql"),
            ))
            .site_auth(SiteAuthConfig {
                login_path: "/admin/login",
                protected_prefixes: &["/admin", "/bridge-auth"],
                public_prefixes: &["/admin/login", "/admin/auth/adfs"],
                required_scope: "user",
            })
            .asset_with(|paths: &dyn AssetPaths| {
                let mut assets = web_assets(paths);
                assets.extend(crate::admin::assets::admin_assets(paths));
                assets
            })
            .conferral(InstallConferral::MemberCipher)
            .conferral(InstallConferral::ShareTokens)
            .conferral(InstallConferral::GitSourceSync)
            .secret(SecretName::new("github_mcp_client_id"))
            .secret(SecretName::new("github_mcp_client_secret"))
            .secret_prefix(CONNECTOR_SECRET_PREFIX)
            .provide::<Arc<WebSettings>, _>(|r: &Resolved<'_>| {
                let config = r.get::<Arc<ConfigView>>()?;
                let paths = r.get::<Arc<AppPaths>>()?;
                Ok(WebSettings::from_config(
                    &config,
                    paths.storage().files().join("admin"),
                ))
            })
            .provide_after_governance::<AdminEnv, _>(|r: &Resolved<'_>| {
                Ok(AdminEnv::new(AdminEnvParts {
                    config: r.get::<Arc<ConfigView>>()?,
                    profile: r.get::<Arc<ProfileView>>()?,
                    services: r.get::<InstallServicesView>()?,
                    authenticator: r.get::<DynRequestAuthenticator>()?,
                    subjects: r.get::<SubjectSourceCell>()?,
                    cipher: r.member_cipher().ok(),
                    share_tokens: r.share_tokens().ok(),
                    secrets: r.secrets()?,
                    paths: r.get::<Arc<AppPaths>>()?,
                    artifact_ui: r.optional::<DynArtifactUiRenderer>(),
                    git_sync: r.git_source_sync().ok(),
                }))
            })
            .provide_after_governance::<Arc<WebRuntime>, _>(|r: &Resolved<'_>| {
                let db = DbHandles::resolve(r)?;
                let settings = r.get::<Arc<WebSettings>>()?;
                let env = r.get::<AdminEnv>()?;
                Ok(Arc::new(WebRuntime::build(&db, &settings, &env)))
            })
            .gateway_guard(|r: &Resolved<'_>| {
                let guard: Arc<dyn GatewayRequestGuard> =
                    Arc::new(RouteEntitlementGuard::new(r.get::<SubjectSourceCell>()?));
                Ok(guard)
            })
            .routes(route_set());
        systemprompt_web_admin::gateway_safety::contribute_scanners(c);
        systemprompt_web_admin::marketplace_filter::contribute_marketplace_filter(c);
        crate::admin::authz::contribute_subject_attributes(c);
        crate::jobs::contribute_jobs(c);
        contribute_content(c);
        Ok(())
    }
}

fn services(r: &Resolved<'_>) -> Result<Arc<WebSettings>, FactoryError> {
    Ok(r.get::<Arc<WebSettings>>()?)
}

// Why: a content factory cannot decline (`ContentSlotBuilder` takes a
// factory that returns the provider or fails), so a page whose config file is
// missing refuses the slot rather than rendering from an invented default.
#[derive(Debug, thiserror::Error)]
#[error("{0} is not configured in the services tree")]
struct MissingSiteConfig(&'static str);

fn required<T>(config: Option<Arc<T>>, what: &'static str) -> Result<Arc<T>, FactoryError> {
    config.ok_or_else(|| FactoryError::Failed(Box::new(MissingSiteConfig(what))))
}

fn contribute_content(c: &mut Contributions<'_>) {
    let content = c.slot::<ContentSlot>();
    content
        .page_data_provider(
            |r: &Resolved<'_>| -> Result<Arc<dyn PageDataProvider>, FactoryError> {
                let settings = services(r)?;
                let nav = required(
                    config_loader::navigation_config(&settings.services_path),
                    "web/navigation.yaml",
                )?;
                let branding = config_loader::branding_config(&settings.services_path);
                Ok(Arc::new(
                    NavigationPageDataProvider::new(nav).with_branding(branding),
                ))
            },
        )
        .page_data_provider(
            |r: &Resolved<'_>| -> Result<Arc<dyn PageDataProvider>, FactoryError> {
                let settings = services(r)?;
                let homepage =
                    config_loader::homepage_config(&settings.services_path).unwrap_or_default();
                Ok(Arc::new(HomepagePageDataProvider::new(homepage)))
            },
        )
        .page_prerenderer(
            |r: &Resolved<'_>| -> Result<Arc<dyn PagePrerenderer>, FactoryError> {
                let settings = services(r)?;
                let homepage =
                    config_loader::homepage_config(&settings.services_path).unwrap_or_default();
                Ok(Arc::new(HomepagePrerenderer::new(homepage)))
            },
        )
        .page_prerenderer(
            |r: &Resolved<'_>| -> Result<Arc<dyn PagePrerenderer>, FactoryError> {
                let settings = services(r)?;
                let skills = required(
                    config_loader::skills_page_config(&settings.services_path),
                    "web/skills page config",
                )?;
                Ok(Arc::new(SkillsPagePrerenderer::new(skills)))
            },
        )
        .page_data_provider(
            |_r: &Resolved<'_>| -> Result<Arc<dyn PageDataProvider>, FactoryError> {
                Ok(Arc::new(DocsPageDataProvider::new()))
            },
        )
        .content_data_provider(
            |_r: &Resolved<'_>| -> Result<Arc<dyn ContentDataProvider>, FactoryError> {
                Ok(Arc::new(DocsContentDataProvider::new()))
            },
        )
        .template_data_extender(
            |r: &Resolved<'_>| -> Result<Arc<dyn TemplateDataExtender>, FactoryError> {
                let settings = services(r)?;
                Ok(Arc::new(OrgUrlExtender::new(
                    settings.api_external_url.clone(),
                )))
            },
        )
        .template_data_extender(
            |_r: &Resolved<'_>| -> Result<Arc<dyn TemplateDataExtender>, FactoryError> {
                Ok(Arc::new(ReleaseVersionExtender::new()))
            },
        );
    contribute_partials(content);
}

macro_rules! component {
    ($content:expr, $renderer:expr) => {
        $content.component_renderer(
            |_r: &Resolved<'_>| -> Result<Arc<dyn ComponentRenderer>, FactoryError> {
                Ok(Arc::new($renderer))
            },
        )
    };
}

fn contribute_partials(content: &mut systemprompt::content::contract::ContentSlotBuilder) {
    component!(content, HeadAssetsPartialRenderer);
    component!(content, HeaderPartialRenderer);
    component!(content, FooterPartialRenderer);
    component!(content, ScriptsPartialRenderer);
    component!(content, CliRemoteAnimationPartialRenderer);
    component!(content, RustMeshAnimationPartialRenderer);
    component!(content, MemoryLoopAnimationPartialRenderer);
    component!(content, AgenticMeshAnimationPartialRenderer);
    component!(content, ArchitectureDiagramPartialRenderer);
}
