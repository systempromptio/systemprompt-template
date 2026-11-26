//! The deployment facts and member-bound handles the admin surface reads.
//!
//! [`AdminEnv`] is built once, in the web member's provide fn, from the install
//! views (`ConfigView`, `ProfileView`, `InstallServicesView`), the open request
//! authenticator, the subject source cell and the member's conferred cipher
//! and share-token signer. The routers carry it as an `Extension` and the jobs
//! resolve it from their context; nothing in the crate reads a process global.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use systemprompt::config::AppPaths;
use systemprompt::extension::artifact_ui::DynArtifactUiRenderer;
use systemprompt::governance::authz::SubjectSourceCell;
use systemprompt::manifest::ConfigView;
use systemprompt::manifest::profile::ProfileView;
use systemprompt::manifest::services::{InstallServicesView, ServicesConfig};
use systemprompt::models::auth::DynRequestAuthenticator;
use systemprompt::traits::{DeclaredSecrets, GitSourceSync, MemberCipher, ShareTokenSigner};

/// What the admin crate needs of the runtime, resolved once.
#[derive(Clone)]
pub struct AdminEnv(Arc<Inner>);

struct Inner {
    config: Arc<ConfigView>,
    profile: Arc<ProfileView>,
    services: InstallServicesView,
    authenticator: DynRequestAuthenticator,
    subjects: SubjectSourceCell,
    cipher: Arc<dyn MemberCipher>,
    share_tokens: Arc<dyn ShareTokenSigner>,
    secrets: DeclaredSecrets,
    paths: Arc<AppPaths>,
    artifact_ui: Option<DynArtifactUiRenderer>,
    git_sync: Arc<dyn GitSourceSync>,
}

pub struct AdminEnvParts {
    pub config: Arc<ConfigView>,
    pub profile: Arc<ProfileView>,
    pub services: InstallServicesView,
    pub authenticator: DynRequestAuthenticator,
    pub subjects: SubjectSourceCell,
    pub cipher: Arc<dyn MemberCipher>,
    pub share_tokens: Arc<dyn ShareTokenSigner>,
    pub secrets: DeclaredSecrets,
    pub paths: Arc<AppPaths>,
    pub artifact_ui: Option<DynArtifactUiRenderer>,
    pub git_sync: Arc<dyn GitSourceSync>,
}

impl std::fmt::Debug for AdminEnvParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdminEnvParts")
            .field("profile", &self.profile.name)
            .finish_non_exhaustive()
    }
}

impl std::fmt::Debug for AdminEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdminEnv")
            .field("profile", &self.0.profile.name)
            .finish_non_exhaustive()
    }
}

impl AdminEnv {
    #[must_use]
    pub fn new(parts: AdminEnvParts) -> Self {
        Self(Arc::new(Inner {
            config: parts.config,
            profile: parts.profile,
            services: parts.services,
            authenticator: parts.authenticator,
            subjects: parts.subjects,
            cipher: parts.cipher,
            share_tokens: parts.share_tokens,
            secrets: parts.secrets,
            paths: parts.paths,
            artifact_ui: parts.artifact_ui,
            git_sync: parts.git_sync,
        }))
    }

    #[must_use]
    pub fn config(&self) -> &ConfigView {
        &self.0.config
    }

    #[must_use]
    pub fn profile(&self) -> &ProfileView {
        &self.0.profile
    }

    #[must_use]
    pub fn services_view(&self) -> &InstallServicesView {
        &self.0.services
    }

    #[must_use]
    pub fn services(&self) -> Arc<ServicesConfig> {
        self.0.services.config()
    }

    #[must_use]
    pub fn active_services_root(&self) -> PathBuf {
        self.0.services.root().to_path_buf()
    }

    #[must_use]
    pub fn baked_services_root(&self) -> PathBuf {
        PathBuf::from(&self.0.config.services_path)
    }

    #[must_use]
    pub fn system_path(&self) -> &Path {
        Path::new(&self.0.config.system_path)
    }

    #[must_use]
    pub fn authenticator(&self) -> &DynRequestAuthenticator {
        &self.0.authenticator
    }

    #[must_use]
    pub fn subjects(&self) -> &SubjectSourceCell {
        &self.0.subjects
    }

    #[must_use]
    pub fn cipher(&self) -> &dyn MemberCipher {
        self.0.cipher.as_ref()
    }

    #[must_use]
    pub fn share_tokens(&self) -> &dyn ShareTokenSigner {
        self.0.share_tokens.as_ref()
    }

    #[must_use]
    pub fn secrets(&self) -> &DeclaredSecrets {
        &self.0.secrets
    }

    #[must_use]
    pub fn paths(&self) -> &AppPaths {
        &self.0.paths
    }

    #[must_use]
    pub fn artifact_ui(&self) -> Option<&DynArtifactUiRenderer> {
        self.0.artifact_ui.as_ref()
    }

    #[must_use]
    pub fn git_sync(&self) -> &dyn GitSourceSync {
        self.0.git_sync.as_ref()
    }
}
