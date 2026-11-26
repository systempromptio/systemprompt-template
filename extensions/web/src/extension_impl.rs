//! `Extension` implementation for `WebExtension`.
//!
//! The wiring seam: every schema, migration, seed, job, subject provider,
//! gateway guard, content provider, provided value and the route set the five
//! web sibling crates expose is contributed here, once, by `contribute`.

use std::sync::Arc;

use systemprompt::config::AppPaths;
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
use systemprompt_web_admin::dashboard::ConsoleDashboard;
use systemprompt_web_admin::env::{AdminEnv, AdminEnvParts};
#[cfg(feature = "gateway")]
use systemprompt_web_admin::gateway_entitlement::RouteEntitlementGuard;

use crate::WebSettings;
#[cfg(feature = "content")]
use crate::assets::web_assets;
use crate::extension::WebExtension;
use crate::router::{DbHandles, WebRuntime, route_set};
use crate::schemas::{migrations, schema_definitions};

// Why: a generic connector's OAuth client is read from a secret the
// operator names in its `connector.client_id_secret`; only names under this
// prefix are declared, so the profile grants them explicitly.
const CONNECTOR_SECRET_PREFIX: &str = "web_connector_";

const CROSS_TABLES: &[&str] = &[
    "users",
    #[cfg(feature = "mcp")]
    "managed_installation_receipts",
    #[cfg(feature = "mcp")]
    "managed_publications",
    #[cfg(feature = "mcp")]
    "managed_resources",
    #[cfg(feature = "mcp")]
    "mcp_tool_executions",
    #[cfg(feature = "content")]
    "markdown_content",
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
        for &table in CROSS_TABLES {
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
            .asset_with(declared_assets)
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
            .conferral(InstallConferral::BrokerCalls)
            .secret(SecretName::new("github_mcp_client_id"))
            .secret(SecretName::new("github_mcp_client_secret"))
            .secret_prefix(CONNECTOR_SECRET_PREFIX)
            .routes(route_set());
        contribute_values(c);
        crate::admin::authz::contribute_subject_attributes(c);
        crate::admin::jobs::contribute_jobs(c);
        crate::sign_in::contribute(c);
        c.job(systemprompt_web_shared::asset_jobs::BundleAdminCssJob)
            .job(systemprompt_web_shared::asset_jobs::CopyExtensionAssetsJob::new(declared_assets));
        #[cfg(feature = "gateway")]
        contribute_gateway(c);
        #[cfg(feature = "mcp")]
        systemprompt_web_admin::marketplace_filter::contribute_marketplace_filter(c);
        #[cfg(feature = "content")]
        {
            crate::jobs::contribute_jobs(c, declared_assets);
            crate::extension_content::contribute_content(c);
        }
        Ok(())
    }
}

fn contribute_values(c: &mut Contributions<'_>) {
    c.provide::<Arc<WebSettings>, _>(|r: &Resolved<'_>| {
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
            broker_calls: r.broker_calls()?,
        }))
    })
    .provide::<Arc<ConsoleDashboard>, _>(|r: &Resolved<'_>| {
        let settings = r.get::<Arc<WebSettings>>()?;
        ConsoleDashboard::load(&settings.services_path)
            .map(Arc::new)
            .map_err(|e| ProvideError::Failed(Box::new(e)))
    })
    .provide_after_governance::<Arc<WebRuntime>, _>(|r: &Resolved<'_>| {
        let db = DbHandles::resolve(r)?;
        let settings = r.get::<Arc<WebSettings>>()?;
        let env = r.get::<AdminEnv>()?;
        let dashboard = r.get::<Arc<ConsoleDashboard>>()?;
        let sign_in = r.get::<Arc<crate::sign_in::SignInSetup>>()?;
        Ok(Arc::new(WebRuntime::build(
            &db,
            &settings,
            &env,
            dashboard,
            sign_in.options(),
        )))
    });
}

#[cfg(feature = "gateway")]
fn contribute_gateway(c: &mut Contributions<'_>) {
    c.gateway_guard(|r: &Resolved<'_>| {
        let guard: Arc<dyn GatewayRequestGuard> =
            Arc::new(RouteEntitlementGuard::new(r.get::<SubjectSourceCell>()?));
        Ok(guard)
    });
    systemprompt_web_admin::gateway_safety::contribute_scanners(c);
}

fn declared_assets(paths: &dyn AssetPaths) -> Vec<AssetDefinition> {
    let mut assets = crate::admin::assets::admin_assets(paths);
    #[cfg(feature = "content")]
    assets.extend(web_assets(paths));
    // Why: the kernel reserves `/` exactly and answers it with
    // `web/dist/index.html`, so a build without a public site sends visitors to
    // the console through that page rather than a route of its own.
    #[cfg(not(feature = "content"))]
    assets.push(AssetDefinition::html(
        paths.storage_files().join("console-home.html"),
        "index.html",
    ));
    assets
}
