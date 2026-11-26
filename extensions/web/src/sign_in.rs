//! The browser sign-in providers the web member contributes to core's
//! `SsoSlot`, decided once from the services tree and the profile.
//!
//! AD FS is contributed when `services/web/config/adfs.yaml` enables it. The
//! development login link is contributed when `dev-login.yaml` enables it on
//! a development, non-cloud profile; enabled on any other profile it refuses
//! the boot. Core serves both under its SSO base and mints every session.

use std::sync::Arc;

use systemprompt::database::DbPool;
use systemprompt::extension::prelude::*;
use systemprompt::manifest::profile::ProfileView;
use systemprompt::oauth_issuance::contract::SsoSlot;
use systemprompt::oauth_issuance::sso::RoleResolver;
use systemprompt::traits::sso::{AuthEventSink, SsoProvider};
use systemprompt_web_admin::sso::dev::DevLoginRefused;
use systemprompt_web_admin::sso::events::{DirectoryRoleResolver, DirectorySignInSink};
use systemprompt_web_admin::{AdfsConfig, AdfsProvider, DevLoginProvider, SignInOptions};

use crate::{WebSettings, config_loader};

/// What the services tree and the profile ask of sign-in.
#[derive(Debug, Clone)]
pub(crate) struct SignInSetup {
    adfs: Option<Arc<AdfsConfig>>,
    dev: bool,
}

/// Why the sign-in setup could not be read.
#[derive(Debug, thiserror::Error)]
pub(crate) enum SignInSetupError {
    #[error("the sign-in configuration does not load")]
    Config(#[source] config_loader::ConfigError),
    #[error(transparent)]
    DevLogin(#[from] DevLoginRefused),
}

impl SignInSetup {
    pub(crate) fn load(
        settings: &WebSettings,
        profile: &ProfileView,
    ) -> Result<Self, SignInSetupError> {
        let adfs = config_loader::adfs_config(&settings.services_path, &settings.api_external_url)
            .map_err(SignInSetupError::Config)?
            .filter(|config| config.enabled);
        let dev = config_loader::dev_login_requested(&settings.services_path)
            .map_err(SignInSetupError::Config)?;
        if dev && !systemprompt_web_admin::dev_login_allowed(profile.environment, profile.target) {
            return Err(DevLoginRefused {
                profile: profile.name.clone(),
                environment: profile.environment,
                target: profile.target,
            }
            .into());
        }
        Ok(Self { adfs, dev })
    }

    #[must_use]
    pub(crate) const fn options(&self) -> SignInOptions {
        SignInOptions {
            adfs: self.adfs.is_some(),
            dev: self.dev,
        }
    }
}

fn setup(r: &Resolved<'_>) -> Result<Arc<SignInSetup>, FactoryError> {
    Ok(r.get::<Arc<SignInSetup>>()?)
}

pub(crate) fn contribute(c: &mut Contributions<'_>) {
    c.provide::<Arc<SignInSetup>, _>(|r: &Resolved<'_>| {
        let settings = r.get::<Arc<WebSettings>>()?;
        let profile = r.get::<Arc<ProfileView>>()?;
        SignInSetup::load(&settings, &profile)
            .map(Arc::new)
            .map_err(|e| ProvideError::Failed(Box::new(e)))
    });
    c.slot::<SsoSlot>()
        .provider(
            |r: &Resolved<'_>| -> Result<Option<Arc<dyn SsoProvider>>, FactoryError> {
                let Some(config) = setup(r)?.adfs.clone() else {
                    return Ok(None);
                };
                let provider =
                    AdfsProvider::new(&config).map_err(|e| FactoryError::Failed(Box::new(e)))?;
                Ok(Some(Arc::new(provider)))
            },
        )
        .provider(
            |r: &Resolved<'_>| -> Result<Option<Arc<dyn SsoProvider>>, FactoryError> {
                if !setup(r)?.dev {
                    return Ok(None);
                }
                let pool = r.get::<DbPool>()?.write_pool();
                Ok(Some(Arc::new(DevLoginProvider::new(pool))))
            },
        )
        .event_sink(
            |r: &Resolved<'_>| -> Result<Arc<dyn AuthEventSink>, FactoryError> {
                Ok(Arc::new(DirectorySignInSink::new(
                    r.get::<DbPool>()?.write_pool(),
                )))
            },
        )
        .role_resolver(
            |r: &Resolved<'_>| -> Result<Arc<dyn RoleResolver>, FactoryError> {
                Ok(Arc::new(DirectoryRoleResolver::new(
                    r.get::<DbPool>()?.write_pool(),
                )))
            },
        );
}
