//! The browser sign-in provider the web member contributes to core's
//! `SsoSlot`, decided once from the services tree.
//!
//! AD FS is contributed when `services/web/config/adfs.yaml` enables it; core
//! serves it under its SSO base and mints every session. With no provider
//! contributed, the console signs in with core's passkey flow alone.

use std::sync::Arc;

use systemprompt::database::DbPool;
use systemprompt::extension::prelude::*;
use systemprompt::oauth_issuance::contract::SsoSlot;
use systemprompt::oauth_issuance::sso::RoleResolver;
use systemprompt::traits::sso::{AuthEventSink, SsoProvider};
use systemprompt_web_admin::sso::events::{DirectoryRoleResolver, DirectorySignInSink};
use systemprompt_web_admin::{AdfsConfig, AdfsProvider, SignInOptions};
use systemprompt_web_shared::config_loader::{self, ConfigError};

use crate::WebSettings;

/// What the services tree asks of sign-in.
#[derive(Debug, Clone)]
pub(crate) struct SignInSetup {
    adfs: Option<Arc<AdfsConfig>>,
}

/// Why the sign-in setup could not be read.
#[derive(Debug, thiserror::Error)]
#[error("the sign-in configuration does not load")]
pub(crate) struct SignInSetupError(#[source] ConfigError);

impl SignInSetup {
    pub(crate) fn load(settings: &WebSettings) -> Result<Self, SignInSetupError> {
        let adfs = config_loader::adfs_config(&settings.services_path, &settings.api_external_url)
            .map_err(SignInSetupError)?
            .filter(|config| config.enabled);
        Ok(Self { adfs })
    }

    #[must_use]
    pub(crate) const fn options(&self) -> SignInOptions {
        SignInOptions {
            adfs: self.adfs.is_some(),
        }
    }
}

pub(crate) fn contribute(c: &mut Contributions<'_>) {
    c.provide::<Arc<SignInSetup>, _>(|r: &Resolved<'_>| {
        let settings = r.get::<Arc<WebSettings>>()?;
        SignInSetup::load(&settings)
            .map(Arc::new)
            .map_err(|e| ProvideError::Failed(Box::new(e)))
    });
    c.slot::<SsoSlot>()
        .provider(
            |r: &Resolved<'_>| -> Result<Option<Arc<dyn SsoProvider>>, FactoryError> {
                let Some(config) = r.get::<Arc<SignInSetup>>()?.adfs.clone() else {
                    return Ok(None);
                };
                let provider =
                    AdfsProvider::new(&config).map_err(|e| FactoryError::Failed(Box::new(e)))?;
                Ok(Some(Arc::new(provider)))
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
