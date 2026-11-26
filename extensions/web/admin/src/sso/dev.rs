//! The development login link, contributed to core's `SsoSlot` as the `dev`
//! provider.
//!
//! `dev-login issue <user>` stores a one-shot code and prints
//! `/api/v1/core/sso/dev/callback?code=…`; the provider redeems the code and
//! asserts that account's identity, and core's driver resolves the account and
//! mints the session like any other sign-in. The provider exists only when
//! `services/web/config/dev-login.yaml` enables it **and** the profile is a
//! development, non-cloud one; enabled on any other profile, the boot is
//! refused.

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;
use systemprompt::manifest::profile::{Environment, ProfileType};
use systemprompt::traits::sso::{
    CookieSameSite, ExternalIdentity, SsoError, SsoIdentityPolicy, SsoPaths, SsoProtocol,
    SsoProvider, SsoRedirect, SsoReturn, SsoStart, SsoStateContext,
};

use crate::repositories::dev_login::consume_dev_login_code;

pub const PROVIDER_ID: &str = "dev";
pub const DEV_LOGIN_PATH: &str = "/api/v1/core/sso/dev/callback";
const START_PATH: &str = "/api/v1/core/sso/dev/start";
const ISSUER: &str = "dev-login";
const LOGIN_PATH: &str = "/admin/login";
const DEFAULT_RETURN_TO: &str = "/admin";

// Why: development alone is not enough — a cloud profile can carry any
// environment label, and a link that signs someone in without a directory
// must never exist off a developer's own machine.
#[must_use]
pub const fn dev_login_allowed(environment: Environment, target: ProfileType) -> bool {
    environment.is_development() && !target.is_cloud()
}

#[must_use]
pub fn dev_login_url(api_external_url: &str, code: &str) -> String {
    format!(
        "{}{DEV_LOGIN_PATH}?code={code}",
        api_external_url.trim_end_matches('/')
    )
}

/// The profile does not allow the development login link.
#[derive(Debug, thiserror::Error)]
#[error(
    "dev-login.yaml enables the development login link, but profile {profile} is {environment:?} / \
     {target:?}; it is allowed only on a development, non-cloud profile"
)]
pub struct DevLoginRefused {
    pub profile: String,
    pub environment: Environment,
    pub target: ProfileType,
}

/// Redeems a one-shot code for the identity of the account it was issued for.
#[derive(Debug)]
pub struct DevLoginProvider {
    pool: Arc<PgPool>,
    policy: SsoIdentityPolicy,
}

impl DevLoginProvider {
    // Why: the code names an existing account, so the identity links to it by
    // its verified email and never provisions one.
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            pool,
            policy: SsoIdentityPolicy {
                link_by_email: true,
                auto_provision: false,
                ..SsoIdentityPolicy::default()
            },
        }
    }
}

#[async_trait]
impl SsoProvider for DevLoginProvider {
    fn id(&self) -> &str {
        PROVIDER_ID
    }

    fn issuer(&self) -> &str {
        ISSUER
    }

    // Why: the slot's protocol enum names the two browser protocols only; the
    // link is a same-origin GET return like an OIDC code.
    fn protocol(&self) -> SsoProtocol {
        SsoProtocol::Oidc
    }

    fn policy(&self) -> &SsoIdentityPolicy {
        &self.policy
    }

    fn paths(&self) -> SsoPaths {
        SsoPaths {
            start: START_PATH,
            callback: DEV_LOGIN_PATH,
            metadata: None,
        }
    }

    fn login_path(&self) -> &str {
        LOGIN_PATH
    }

    fn default_return_to(&self) -> &str {
        DEFAULT_RETURN_TO
    }

    // Why: the link the CLI prints is the whole sign-in; there is no start
    // step whose state it could echo.
    fn allow_unsolicited(&self) -> bool {
        true
    }

    fn return_cookie_same_site(&self) -> CookieSameSite {
        CookieSameSite::Lax
    }

    async fn start(&self, _input: SsoStart<'_>) -> Result<SsoRedirect, SsoError> {
        Err(SsoError::InvalidAssertion {
            reason: "the development login starts from a link the CLI issues",
        })
    }

    async fn finish(
        &self,
        ret: SsoReturn,
        _state: Option<SsoStateContext>,
    ) -> Result<ExternalIdentity, SsoError> {
        let code = ret.param("code").ok_or(SsoError::InvalidAssertion {
            reason: "no login code",
        })?;
        let user = consume_dev_login_code(&self.pool, code)
            .await
            .map_err(SsoError::internal)?
            .ok_or(SsoError::InvalidAssertion {
                reason: "the login code is unknown, spent or expired",
            })?;
        tracing::info!(user_id = %user.user_id, "dev login code redeemed");
        Ok(ExternalIdentity {
            subject: user.user_id.as_str().to_owned(),
            email: Some(user.email),
            email_verified: true,
            display_name: Some(user.display_name),
            groups: Vec::new(),
            groups_overage: false,
            tenant_id: None,
            assertion_id: None,
            assertion_expires_at: None,
            authenticated_at: None,
            raw_claims: serde_json::Value::Null,
        })
    }
}
