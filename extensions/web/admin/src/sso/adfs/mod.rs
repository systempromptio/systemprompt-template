//! "Sign in with Systemprompt SSO": SAML 2.0 Web Browser SSO against the
//! corporate AD FS farm, contributed to core's `SsoSlot` as the `adfs`
//! provider.
//!
//! The provider speaks the protocol only: [`AdfsProvider::start`] builds the
//! `AuthnRequest` (HTTP-Redirect binding) and [`AdfsProvider::finish`]
//! verifies the posted `SAMLResponse` (XML-DSig against the metadata-pinned
//! certificate, audience, destination, validity window, `InResponseTo`).
//! Core's SSO driver owns everything around it: the state and its browser
//! binder, assertion replay, the identity policy built from `adfs.yaml`,
//! account resolution and the session.
//!
//! Module layout: [`AdfsConfig`] (the loaded YAML, from the shared crate),
//! `descriptors` (the SP and `IdP` descriptors built from it), `claims`
//! (what an assertion asserted).

mod claims;
mod descriptors;

use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use rand::Rng;
use saml::sp::LoginTracker;
use saml::{
    Binding, ConsumeResponse, Dispatch, IdpDescriptor, ReplayMode, ServiceProvider,
    SsoResponseBinding, StartLogin,
};
use systemprompt::identifiers::{SsoAssertionId, SsoRequestId};
use systemprompt::traits::sso::{
    CookieSameSite, ExternalIdentity, SsoError, SsoIdentityPolicy, SsoMetadataDoc, SsoPaths,
    SsoProtocol, SsoProvider, SsoRedirect, SsoReturn, SsoStart, SsoStateContext, SsoStateSeed,
};

pub use claims::AssertionClaims;
pub use systemprompt_web_shared::adfs::{
    ACS_PATH, AdfsConfig, ENTITY_ID_PATH, METADATA_PATH, START_PATH, group_matches_pattern,
};

pub const PROVIDER_ID: &str = "adfs";
const LOGIN_PATH: &str = "/admin/login";
const DEFAULT_RETURN_TO: &str = "/admin";

const PATHS: SsoPaths = SsoPaths {
    start: START_PATH,
    callback: ACS_PATH,
    metadata: Some(METADATA_PATH),
};

/// Why an enabled `adfs.yaml` cannot become a provider, or an assertion was
/// refused.
#[derive(Debug, thiserror::Error)]
pub enum AdfsError {
    #[error("IdP federation metadata is unusable: {0}")]
    Metadata(String),
    #[error("service-provider configuration rejected: {0}")]
    ServiceProvider(String),
    #[error("adfs.yaml is enabled but {0}")]
    Incomplete(&'static str),
    #[error("assertion rejected")]
    Assertion(#[source] saml::Error),
    #[error("login tracker could not be carried across the redirect")]
    Tracker(#[source] serde_json::Error),
}

/// The AD FS relying party, built once from an enabled `adfs.yaml`.
#[derive(Debug)]
pub struct AdfsProvider {
    config: AdfsConfig,
    idp: IdpDescriptor,
    sp: ServiceProvider,
    policy: SsoIdentityPolicy,
}

impl AdfsProvider {
    // Why: core's policy admits every domain when the allow-list is empty,
    // while this config's contract is that no domain is admitted until the
    // operator names one, so an enabled config with none is refused here.
    pub fn new(config: &AdfsConfig) -> Result<Self, AdfsError> {
        if config.entity_id.is_empty() || config.acs_url.is_empty() {
            return Err(AdfsError::Incomplete(
                "no entity id or ACS URL could be derived",
            ));
        }
        if config.idp_metadata_xml.is_empty() {
            return Err(AdfsError::Incomplete("its IdP metadata is empty"));
        }
        if config.allowed_email_domains.is_empty() {
            return Err(AdfsError::Incomplete("it names no allowed email domain"));
        }
        let idp = descriptors::idp_descriptor(config)?;
        let sp = descriptors::service_provider(config)?;
        let policy = SsoIdentityPolicy {
            allowed_email_domains: config.allowed_email_domains.clone(),
            auto_provision: config.auto_provision,
            link_by_email: false,
            deny_without_group: config.deny_without_group,
            group_roles: config.group_roles.clone(),
            group_role_patterns: config.group_role_patterns.clone(),
            default_roles: vec!["user".to_owned()],
        };
        Ok(Self {
            config: config.clone(),
            idp,
            sp,
            policy,
        })
    }

    fn identity(&self, identity: &saml::Identity) -> ExternalIdentity {
        let claims = AssertionClaims::from_identity(identity);
        let email = claims.login_email(&self.config);
        let subject = email
            .as_deref()
            .map_or_else(|| claims.external_sub(""), |e| claims.external_sub(e));
        // JSON: `ExternalIdentity::raw_claims` keeps the assertion's attributes as
        // issued — an open set the IdP defines, not a shape this crate owns.
        let raw_claims = serde_json::Value::Object(
            claims
                .attributes
                .iter()
                .map(|(name, values)| (name.clone(), serde_json::json!(values)))
                .collect(),
        );
        ExternalIdentity {
            subject,
            email,
            email_verified: true,
            display_name: claims.display_name(&self.config),
            groups: claims.values(&self.config.groups_attribute),
            groups_overage: false,
            tenant_id: None,
            assertion_id: Some(SsoAssertionId::new(identity.assertion_id.clone())),
            assertion_expires_at: Some(DateTime::<Utc>::from(identity.not_on_or_after)),
            authenticated_at: Some(DateTime::<Utc>::from(identity.authn_instant)),
            raw_claims,
        }
    }
}

fn tracker(state: Option<&SsoStateContext>) -> Result<Option<LoginTracker>, SsoError> {
    let Some(carried) = state.and_then(|s| s.nonce.as_deref()) else {
        return Ok(None);
    };
    serde_json::from_str(carried)
        .map(Some)
        .map_err(|e| SsoError::internal(AdfsError::Tracker(e)))
}

fn random_url_safe() -> String {
    let bytes: [u8; 32] = rand::rng().random();
    URL_SAFE_NO_PAD.encode(bytes)
}

fn assertion_refused(error: saml::Error) -> SsoError {
    tracing::warn!(error = %error, "ADFS assertion rejected");
    match error {
        saml::Error::UnsolicitedNotAllowed => SsoError::UnsolicitedNotAllowed,
        saml::Error::XmlParse(m) if m.contains("NameID") => SsoError::NoSubject,
        other => {
            tracing::debug!(error = ?other, "ADFS assertion detail");
            SsoError::InvalidAssertion {
                reason: "the SAMLResponse did not verify",
            }
        },
    }
}

#[async_trait]
impl SsoProvider for AdfsProvider {
    fn id(&self) -> &str {
        PROVIDER_ID
    }

    fn issuer(&self) -> &str {
        &self.idp.entity_id
    }

    fn protocol(&self) -> SsoProtocol {
        SsoProtocol::Saml2
    }

    fn policy(&self) -> &SsoIdentityPolicy {
        &self.policy
    }

    fn paths(&self) -> SsoPaths {
        PATHS
    }

    fn login_path(&self) -> &str {
        LOGIN_PATH
    }

    fn default_return_to(&self) -> &str {
        DEFAULT_RETURN_TO
    }

    fn allow_unsolicited(&self) -> bool {
        self.config.allow_idp_initiated
    }

    // Why: AD FS returns the browser with a cross-site form POST, which a
    // `SameSite=Lax` binder cookie does not ride on.
    fn return_cookie_same_site(&self) -> CookieSameSite {
        CookieSameSite::None
    }

    async fn start(&self, input: SsoStart<'_>) -> Result<SsoRedirect, SsoError> {
        if input.callback_url != self.config.acs_url {
            return Err(SsoError::internal(format!(
                "the ADFS ACS {} is not core's callback {}",
                self.config.acs_url, input.callback_url
            )));
        }
        let state = random_url_safe();
        let started = self
            .sp
            .start_login(
                &self.idp,
                StartLogin {
                    relay_state: Some(&state),
                    binding: Binding::HttpRedirect,
                    force_authn: input.reauth.force,
                    is_passive: false,
                    requested_name_id_format: None,
                    requested_authn_context: None,
                    acs_index: None,
                    acs_url: None,
                    response_binding: None,
                },
            )
            .map_err(|e| SsoError::internal(AdfsError::Assertion(e)))?;
        let Dispatch::Redirect(target) = started.dispatch else {
            return Err(SsoError::internal(
                "ADFS start produced a POST dispatch for a redirect binding",
            ));
        };
        // Why: the response is verified against the AuthnRequest it answers,
        // so the whole tracker rides in the provider's remembered value.
        let tracker = serde_json::to_string(&started.tracker)
            .map_err(|e| SsoError::internal(AdfsError::Tracker(e)))?;
        Ok(SsoRedirect {
            url: target.to_string(),
            seed: SsoStateSeed {
                state,
                request_id: Some(SsoRequestId::new(started.tracker.request_id.clone())),
                nonce: Some(tracker),
                pkce_verifier: None,
            },
        })
    }

    async fn finish(
        &self,
        ret: SsoReturn,
        state: Option<SsoStateContext>,
    ) -> Result<ExternalIdentity, SsoError> {
        let encoded = ret
            .param("SAMLResponse")
            .ok_or(SsoError::InvalidAssertion {
                reason: "no SAMLResponse was posted",
            })?;
        let xml = STANDARD
            .decode(encoded.trim())
            .map_err(|_e| SsoError::InvalidAssertion {
                reason: "the SAMLResponse is not base64",
            })?;
        let tracker = tracker(state.as_ref())?;
        let identity = self
            .sp
            .consume_response(ConsumeResponse {
                idp: &self.idp,
                peer_crypto_policy: None,
                saml_response: &xml,
                binding: SsoResponseBinding::HttpPost,
                relay_state: ret.param("RelayState"),
                tracker: tracker.as_ref(),
                expected_destination: &self.config.acs_url,
                now: SystemTime::now(),
                clock_skew: Duration::from_secs(self.config.clock_skew_seconds),
                replay_cache: None,
                replay_mode: ReplayMode::All,
                holder_of_key_cert: None,
            })
            .map_err(assertion_refused)?;
        Ok(self.identity(&identity))
    }

    async fn metadata(&self) -> Option<Result<SsoMetadataDoc, SsoError>> {
        Some(Ok(SsoMetadataDoc {
            content_type: "application/samlmetadata+xml",
            body: sp_metadata(&self.config),
        }))
    }
}

fn sp_metadata(cfg: &AdfsConfig) -> String {
    format!(
        "<?xml version=\"1.0\"?>\n<md:EntityDescriptor xmlns:md=\"urn:oasis:names:tc:SAML:2.0:metadata\" entityID=\"{entity}\"><md:SPSSODescriptor AuthnRequestsSigned=\"false\" WantAssertionsSigned=\"true\" protocolSupportEnumeration=\"urn:oasis:names:tc:SAML:2.0:protocol\"><md:NameIDFormat>urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress</md:NameIDFormat><md:AssertionConsumerService Binding=\"urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST\" Location=\"{acs}\" index=\"0\" isDefault=\"true\"/></md:SPSSODescriptor></md:EntityDescriptor>\n",
        entity = escape_attr(&cfg.entity_id),
        acs = escape_attr(&cfg.acs_url),
    )
}

fn escape_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
