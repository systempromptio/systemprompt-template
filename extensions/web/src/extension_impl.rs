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
use systemprompt::extension::{ResealColumn, SealedLayout};
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
                public_prefixes: &["/admin/login"],
                required_scope: "user",
            })
            .asset_with(|paths: &dyn AssetPaths| {
                let mut assets = web_assets(paths);
                assets.extend(crate::admin::assets::admin_assets(paths));
                assets
            })
            .conferral(InstallConferral::MemberCipher)
            .reseal_column(ResealColumn {
                table: "user_encryption_keys",
                keys: &["user_id"],
                ciphertext: "encrypted_dek",
                layout: SealedLayout::SplitBytea { nonce: "dek_nonce" },
            })
            .reseal_column(ResealColumn {
                table: "mcp_connector_credentials",
                keys: &["user_id", "provider"],
                ciphertext: "ciphertext",
                layout: SealedLayout::SplitBytea { nonce: "nonce" },
            })
            .reseal_column(ResealColumn {
                table: "mcp_connector_oauth_states",
                keys: &["state"],
                ciphertext: "ciphertext",
                layout: SealedLayout::SplitBytea { nonce: "nonce" },
            })
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
                    cipher: r.member_cipher()?,
                    share_tokens: r.share_tokens()?,
                    secrets: r.secrets()?,
                    paths: r.get::<Arc<AppPaths>>()?,
                    artifact_ui: r.optional::<DynArtifactUiRenderer>(),
                    git_sync: r.git_source_sync()?,
                }))
            })
            .provide_after_governance::<Arc<WebRuntime>, _>(|r: &Resolved<'_>| {
                // Why: the routers need AdminEnv, which a child host withholds
                // with the member cipher; asking for the cipher here withholds
                // the runtime on the same host for the same reason.
                r.member_cipher()?;
                let db = DbHandles::resolve(r)?;
                let settings = r.get::<Arc<WebSettings>>()?;
                let env = r.get::<AdminEnv>()?;
                let sign_in = r.get::<Arc<crate::sign_in::SignInSetup>>()?;
                Ok(Arc::new(WebRuntime::build(
                    &db,
                    &settings,
                    &env,
                    sign_in.options(),
                )))
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
        crate::sign_in::contribute(c);
        c.command(systemprompt::extension::cli::CommandDecl::group::<
            crate::dev_login_command::DevLoginCommands,
        >());
        contribute_content(c);
        Ok(())
    }
}

fn services(r: &Resolved<'_>) -> Result<Arc<WebSettings>, FactoryError> {
    Ok(r.get::<Arc<WebSettings>>()?)
}

type Built<T> = Result<Option<Arc<T>>, FactoryError>;

fn contribute_content(c: &mut Contributions<'_>) {
    let content = c.slot::<ContentSlot>();
    content
        .page_data_provider(|r: &Resolved<'_>| -> Built<dyn PageDataProvider> {
            let settings = services(r)?;
            let Some(nav) = config_loader::navigation_config(&settings.services_path) else {
                return Ok(None);
            };
            let branding = config_loader::branding_config(&settings.services_path);
            Ok(Some(Arc::new(
                NavigationPageDataProvider::new(nav).with_branding(branding),
            )))
        })
        .page_data_provider(|r: &Resolved<'_>| -> Built<dyn PageDataProvider> {
            let settings = services(r)?;
            let homepage =
                config_loader::homepage_config(&settings.services_path).unwrap_or_default();
            Ok(Some(Arc::new(HomepagePageDataProvider::new(homepage))))
        })
        .page_prerenderer(|r: &Resolved<'_>| -> Built<dyn PagePrerenderer> {
            let settings = services(r)?;
            let homepage =
                config_loader::homepage_config(&settings.services_path).unwrap_or_default();
            Ok(Some(Arc::new(HomepagePrerenderer::new(homepage))))
        })
        .page_prerenderer(|r: &Resolved<'_>| -> Built<dyn PagePrerenderer> {
            let settings = services(r)?;
            Ok(
                config_loader::skills_page_config(&settings.services_path).map(
                    |skills| -> Arc<dyn PagePrerenderer> {
                        Arc::new(SkillsPagePrerenderer::new(skills))
                    },
                ),
            )
        })
        .page_data_provider(|_r: &Resolved<'_>| -> Built<dyn PageDataProvider> {
            Ok(Some(Arc::new(DocsPageDataProvider::new())))
        })
        .content_data_provider(|_r: &Resolved<'_>| -> Built<dyn ContentDataProvider> {
            Ok(Some(Arc::new(DocsContentDataProvider::new())))
        })
        .template_data_extender(|r: &Resolved<'_>| -> Built<dyn TemplateDataExtender> {
            let settings = services(r)?;
            Ok(Some(Arc::new(OrgUrlExtender::new(
                settings.api_external_url.clone(),
            ))))
        })
        .template_data_extender(|_r: &Resolved<'_>| -> Built<dyn TemplateDataExtender> {
            Ok(Some(Arc::new(ReleaseVersionExtender::new())))
        });
    contribute_partials(content);
}

macro_rules! component {
    ($content:expr, $renderer:expr) => {
        $content.component_renderer(|_r: &Resolved<'_>| -> Built<dyn ComponentRenderer> {
            Ok(Some(Arc::new($renderer)))
        })
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
