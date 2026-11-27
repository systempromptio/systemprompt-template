//! An `AdminEnv` built the way the web member's provide fn builds it.
//!
//! The parts come from a fixture profile and a services tree the test states:
//! the install views are derived from them by core's own constructors
//! (`ProfileView::of`, `ServicesState::from_config`,
//! `InstallServicesView::of`), the subject source cell is bound to the same
//! providers the composition admits (core's `traffic` and role providers, the
//! web member's group, project, connector and organization providers), and the
//! member cipher is issued by core's `MemberCiphers` from a fixed test key. The
//! request authenticator refuses every request unless a suite supplies its own
//! through `EnvFixture::with_authenticator`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::config::AppPaths;
use systemprompt::extension::{LateCell, StandingTag};
use systemprompt::governance::authz::{
    SubjectAttributeProvider, SubjectProviderEntry, SubjectProviderSet, SubjectSourceCell,
    TrafficProvider,
};
use systemprompt::identifiers::{ExtensionId, JwtToken, UserId};
use systemprompt::manifest::ConfigView;
use systemprompt::manifest::paths::PathResolution;
use systemprompt::manifest::profile::{Profile, ProfileView};
use systemprompt::manifest::services::root::{ActiveServicesRoot, ServicesProvenance};
use systemprompt::manifest::services::{InstallServicesView, ServicesConfig};
use systemprompt::models::auth::{
    DynRequestAuthenticator, JwtAudience, JwtUserContext, RequestAuthenticator,
};
use systemprompt::models::execution::context::{ContextExtractionError, RequestContext};
use systemprompt::security::at_rest::MasterKey;
use systemprompt::security::conferred::{MemberCiphers, MemberShareTokens};
use systemprompt::traits::{
    AuthUser, BrokerCallRefused, BrokerCallVerifier, ConferralIssuer, DeclaredSecrets,
    ExtensionSecret, GitSourceSync, GitSourceSyncError, GitSourceSyncRequest, GitSourceSynced,
    MemberCipher, MemberGrant, ReloadOutcome, SecretReferences, SecretStore, SecretStoreError,
    SecretsStatus, ShareTokenSigner,
};
use systemprompt_web_admin::authz::connector::ConnectorAttributeProvider;
use systemprompt_web_admin::authz::group::GroupAttributeProvider;
use systemprompt_web_admin::authz::organization::OrganizationAttributeProvider;
use systemprompt_web_admin::authz::project::ProjectAttributeProvider;
use systemprompt_web_admin::env::{AdminEnv, AdminEnvParts};

use crate::paths::repo_root;

const PROFILE: &str = include_str!("../fixtures/profile.yaml");

pub const MEMBER_KEY: [u8; 32] = [7u8; 32];

#[must_use]
pub fn web_member() -> ExtensionId {
    ExtensionId::new("web")
}

#[must_use]
pub fn fixture_profile() -> Profile {
    let root = repo_root();
    let text = PROFILE.replace("__REPO__", &root.to_string_lossy());
    let mut profile: Profile = serde_yaml::from_str(&text).expect("the fixture profile parses");
    profile
        .derive_fields()
        .expect("the fixture profile derives its URLs");
    profile
}

#[must_use]
pub fn services_config(yaml: &str) -> ServicesConfig {
    serde_yaml::from_str(yaml).expect("the services fixture parses")
}

#[must_use]
pub fn install_services(
    profile: &Profile,
    services: ServicesConfig,
    root: &Path,
) -> InstallServicesView {
    let state = systemprompt::loader::ServicesState::from_config(profile, services)
        .expect("the services fixture resolves");
    let active = ActiveServicesRoot {
        path: root.to_path_buf(),
        base: root.to_path_buf(),
        provenance: ServicesProvenance::Bundled,
    };
    InstallServicesView::of(state.view(), &active, profile, root.join(".bundle-cache"))
}

#[must_use]
pub fn member_cipher() -> Arc<dyn MemberCipher> {
    let master = MasterKey::from_store(&FixtureMasterKey).expect("the fixture store holds a key");
    MemberCiphers::from_master_key(&master)
        .expect("the test member key is valid")
        .issue(&web_grant())
}

#[must_use]
pub fn share_tokens() -> Arc<dyn ShareTokenSigner> {
    MemberShareTokens::from_seed(&MEMBER_KEY)
        .expect("the test share seed is valid")
        .issue(&web_grant())
}

fn web_grant() -> MemberGrant {
    MemberGrant {
        member: web_member(),
        token_audiences: Arc::from(Vec::new()),
    }
}

#[derive(Debug)]
struct AbsentBroker;

impl BrokerCallVerifier for AbsentBroker {
    fn verify(&self, _headers: &http::HeaderMap) -> Result<(), BrokerCallRefused> {
        Err(BrokerCallRefused::Absent)
    }
}

#[derive(Debug)]
struct RefusingGitSync;

#[async_trait::async_trait]
impl GitSourceSync for RefusingGitSync {
    async fn sync(
        &self,
        _request: &GitSourceSyncRequest,
    ) -> Result<GitSourceSynced, GitSourceSyncError> {
        Err(GitSourceSyncError::NotGitSource)
    }
}

fn entry(
    owner: &str,
    standing: StandingTag,
    provider: Arc<dyn SubjectAttributeProvider>,
) -> SubjectProviderEntry {
    SubjectProviderEntry {
        owner: ExtensionId::new(owner),
        standing,
        provider,
    }
}

#[must_use]
pub fn subject_cell(pool: &Arc<PgPool>, services: &InstallServicesView) -> SubjectSourceCell {
    let set = SubjectProviderSet::new(vec![
        entry("authz", StandingTag::Core, Arc::new(TrafficProvider)),
        entry(
            "users",
            StandingTag::Core,
            Arc::new(systemprompt::users::UsersRoleSubjectProvider::new(
                Arc::clone(pool),
            )),
        ),
        entry(
            "web",
            StandingTag::Install,
            Arc::new(GroupAttributeProvider::new(Arc::clone(pool))),
        ),
        entry(
            "web",
            StandingTag::Install,
            Arc::new(ProjectAttributeProvider::new(Arc::clone(pool))),
        ),
        entry(
            "web",
            StandingTag::Install,
            Arc::new(ConnectorAttributeProvider::new(
                Arc::clone(pool),
                services.clone(),
            )),
        ),
        entry(
            "web",
            StandingTag::Install,
            Arc::new(OrganizationAttributeProvider),
        ),
    ])
    .expect("the composed subject providers form a set");
    let (cell, binder) = SubjectSourceCell::unbound();
    SubjectSourceCell::bind(binder, Arc::new(set)).expect("a fresh cell binds once");
    cell
}

#[derive(Debug)]
struct NoSecrets;

#[derive(Debug, thiserror::Error)]
#[error("the test environment holds no secrets")]
struct NoStore;

fn no_store() -> SecretStoreError {
    SecretStoreError::new("the test environment holds no secrets", NoStore)
}

#[async_trait::async_trait]
impl SecretStore for NoSecrets {
    fn secret(&self, _name: &str) -> Option<ExtensionSecret> {
        None
    }

    fn secret_exact(&self, _name: &str) -> Option<ExtensionSecret> {
        None
    }

    fn key_names(&self) -> Result<Vec<String>, SecretStoreError> {
        Ok(Vec::new())
    }

    fn manifest_signing_secret_seed(&self) -> Result<[u8; 32], SecretStoreError> {
        Err(no_store())
    }

    fn encryption_master_key(&self) -> Result<[u8; 32], SecretStoreError> {
        Err(no_store())
    }

    fn status(&self) -> SecretsStatus {
        SecretsStatus {
            source: "test".to_owned(),
            generation: 0,
            initialised_at: chrono::Utc::now(),
            last_reload_at: None,
            last_error: None,
        }
    }

    fn refresh_interval(&self) -> Option<std::time::Duration> {
        None
    }

    async fn reload(&self) -> Result<ReloadOutcome, SecretStoreError> {
        Err(no_store())
    }
}

#[async_trait::async_trait]
impl SecretStore for FixtureMasterKey {
    fn secret(&self, _name: &str) -> Option<ExtensionSecret> {
        None
    }

    fn secret_exact(&self, _name: &str) -> Option<ExtensionSecret> {
        None
    }

    fn key_names(&self) -> Result<Vec<String>, SecretStoreError> {
        Ok(Vec::new())
    }

    fn manifest_signing_secret_seed(&self) -> Result<[u8; 32], SecretStoreError> {
        Err(no_store())
    }

    fn encryption_master_key(&self) -> Result<[u8; 32], SecretStoreError> {
        Ok(MEMBER_KEY)
    }

    fn status(&self) -> SecretsStatus {
        SecretsStatus {
            source: "test".to_owned(),
            generation: 0,
            initialised_at: chrono::Utc::now(),
            last_reload_at: None,
            last_error: None,
        }
    }

    fn refresh_interval(&self) -> Option<std::time::Duration> {
        None
    }

    async fn reload(&self) -> Result<ReloadOutcome, SecretStoreError> {
        Err(no_store())
    }
}

#[derive(Debug)]
struct FixtureMasterKey;

#[derive(Debug)]
struct NoReferences;

impl SecretReferences for NoReferences {
    fn references(&self, _name: &str) -> bool {
        false
    }
}

#[derive(Debug)]
struct RefusingAuthenticator;

fn refused() -> ContextExtractionError {
    ContextExtractionError::MissingAuthHeader
}

#[async_trait::async_trait]
impl RequestAuthenticator for RefusingAuthenticator {
    async fn authenticate(
        &self,
        _headers: &http::HeaderMap,
        _audiences: &[JwtAudience],
    ) -> Result<RequestContext, ContextExtractionError> {
        Err(refused())
    }

    async fn decode_for_audiences(
        &self,
        _token: &JwtToken,
        _audiences: &[JwtAudience],
    ) -> Result<(JwtUserContext, AuthUser), ContextExtractionError> {
        Err(refused())
    }

    async fn decode_for_gateway(
        &self,
        _token: &JwtToken,
    ) -> Result<(JwtUserContext, AuthUser), ContextExtractionError> {
        Err(refused())
    }

    async fn load_user(
        &self,
        _user_id: &UserId,
        _route_context: &str,
    ) -> Result<AuthUser, ContextExtractionError> {
        Err(refused())
    }
}

fn config_view(profile: &Profile, services_root: &Path) -> ConfigView {
    let paths = &profile.paths;
    ConfigView {
        instance_id: profile
            .server
            .instance_id
            .clone()
            .expect("the fixture profile names its instance"),
        metrics_port: profile.server.metrics_port,
        max_concurrent_streams: profile.server.max_concurrent_streams,
        role: profile.server.role,
        advertise_url: profile.server.advertise_url.clone(),
        max_in_flight: profile.server.max_in_flight,
        shutdown: profile.server.shutdown,
        sitename: profile.site.name.clone(),
        database_type: profile.database.db_type.clone(),
        github_link: String::new(),
        system_path: paths.system.clone(),
        services_path: services_root.to_string_lossy().into_owned(),
        bin_path: paths.bin.clone(),
        skills_path: paths.skills(),
        settings_path: paths.config(),
        content_config_path: paths.content_config(),
        geoip_database_path: None,
        web_path: paths.web_path_resolved(),
        web_config_path: paths.web_config(),
        web_metadata_path: paths.web_metadata(),
        host: profile.server.host.clone(),
        port: profile.server.port,
        api_server_url: profile.server.api_server_url.clone(),
        api_internal_url: profile.server.api_internal_url.clone(),
        api_external_url: profile.server.api_external_url.clone(),
        jwt_issuer: profile.security.issuer.clone(),
        jwt_access_token_expiration: profile.security.access_token_expiration,
        jwt_refresh_token_expiration: profile.security.refresh_token_expiration,
        jwt_audiences: profile.security.audiences.clone(),
        allowed_resource_audiences: profile.security.allowed_resource_audiences.clone(),
        trusted_issuers: profile.security.trusted_issuers.clone(),
        id_jag_ttl_secs: profile.security.id_jag_ttl_secs,
        use_https: profile.server.use_https,
        rate_limits: profile.rate_limits,
        retention: profile.retention,
        planes: profile.config_planes.clone(),
        cors_allowed_origins: profile.server.cors_allowed_origins.clone(),
        trusted_proxies: profile.server.trusted_proxies.clone(),
        is_cloud: profile.target.is_cloud(),
        content_negotiation: profile.server.content_negotiation.clone(),
        security_headers: profile.server.security_headers.clone(),
        allow_registration: profile.security.allow_registration,
        allow_dynamic_client_registration: profile.security.allow_dynamic_client_registration,
        api_key_max_lifetime_days: profile.security.api_keys.max_lifetime_days,
        login_page_url: profile.security.login_page_url.clone(),
        ui_session: profile.security.ui_session,
        authorize: profile.security.authorize,
        system_admin_username: profile.system_admin.username.clone(),
        system_admin_email: profile.system_admin.email.clone(),
    }
}

#[derive(Debug)]
pub struct EnvFixture {
    pub env: AdminEnv,
    pub profile: Profile,
    pub services_root: PathBuf,
}

impl EnvFixture {
    #[must_use]
    pub fn new(pool: &Arc<PgPool>, services: ServicesConfig, services_root: &Path) -> Self {
        Self::build(
            pool,
            services,
            services_root,
            member_cipher(),
            Arc::new(RefusingAuthenticator),
        )
    }

    #[must_use]
    pub fn with_authenticator(pool: &Arc<PgPool>, authenticator: DynRequestAuthenticator) -> Self {
        Self::build(
            pool,
            ServicesConfig::default(),
            &repo_root().join("services"),
            member_cipher(),
            authenticator,
        )
    }

    #[must_use]
    pub fn with_services_and_authenticator(
        pool: &Arc<PgPool>,
        services: ServicesConfig,
        services_root: &Path,
        authenticator: DynRequestAuthenticator,
    ) -> Self {
        Self::build(
            pool,
            services,
            services_root,
            member_cipher(),
            authenticator,
        )
    }

    #[must_use]
    pub fn empty(pool: &Arc<PgPool>) -> Self {
        Self::new(
            pool,
            ServicesConfig::default(),
            &repo_root().join("services"),
        )
    }

    fn build(
        pool: &Arc<PgPool>,
        services: ServicesConfig,
        services_root: &Path,
        cipher: Arc<dyn MemberCipher>,
        authenticator: DynRequestAuthenticator,
    ) -> Self {
        let profile = fixture_profile();
        let view = install_services(&profile, services, services_root);
        let subjects = subject_cell(pool, &view);
        let grants = profile.extensions.secret_grants(&web_member()).to_vec();
        let paths =
            AppPaths::from_profile(&profile.paths, PathResolution::Lexical, Some(services_root))
                .expect("the fixture profile's paths resolve");
        let env = AdminEnv::new(AdminEnvParts {
            config: Arc::new(config_view(&profile, services_root)),
            profile: Arc::new(ProfileView::of(&profile)),
            services: view,
            authenticator,
            subjects,
            cipher,
            share_tokens: share_tokens(),
            secrets: DeclaredSecrets::over(
                Arc::new(NoSecrets),
                Arc::from(grants.clone()),
                Arc::from(grants),
                Arc::new(NoReferences),
            ),
            paths: Arc::new(paths),
            artifact_ui: None,
            git_sync: Arc::new(RefusingGitSync),
            broker_calls: Arc::new(AbsentBroker),
        });
        Self {
            env,
            profile,
            services_root: services_root.to_path_buf(),
        }
    }
}
