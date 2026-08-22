//! Salesforce SSO connection config, loaded from
//! `services/web/config/salesforce.yaml`.

use serde::Deserialize;

// Why: Resolve the Salesforce Connected App secret, env var first then the
// encrypted secrets store, mirroring
// [`crate::repositories::secrets::secret_crypto::load_master_key`]. The secret
// is never persisted in `salesforce.yaml`.
pub(crate) fn client_secret() -> Option<String> {
    // Why: env::var().ok() and SecretsBootstrap::get().ok() are both
    // missing-is-normal carve-outs encoding the priority chain (env var
    // first, then bootstrap).
    std::env::var("SALESFORCE_CLIENT_SECRET").ok().or_else(|| {
        systemprompt::config::SecretsBootstrap::get()
            .ok()
            .and_then(|s| s.get("salesforce_client_secret").cloned())
    })
}

// Why: Resolve the Salesforce Connected App private key (PEM) used to sign the
// RFC 7523 JWT-bearer assertion. Env var first, then the encrypted secrets
// store, mirroring [`client_secret`]. Never persisted in `salesforce.yaml`.
pub(crate) fn salesforce_private_key() -> Option<String> {
    std::env::var("SALESFORCE_PRIVATE_KEY").ok().or_else(|| {
        systemprompt::config::SecretsBootstrap::get()
            .ok()
            .and_then(|s| s.get("salesforce_private_key").cloned())
    })
}

// Why: Resolve the public certificate paired with [`salesforce_private_key`].
// Env var first, then the encrypted secrets store, mirroring the two above.
//
// Only `salesforce_org::apply` needs it, and it needs it badly: a metadata
// deploy is declarative, so a package omitting `<certificate>` clears the app's
// digital signature and the JWT-bearer grant that deployed it stops working.
// Public material, so storing it beside the private key is about keeping the
// pair together, not about secrecy.
pub(crate) fn salesforce_certificate() -> Option<String> {
    std::env::var("SALESFORCE_CERTIFICATE").ok().or_else(|| {
        systemprompt::config::SecretsBootstrap::get()
            .ok()
            .and_then(|s| s.get("salesforce_certificate").cloned())
    })
}

pub(super) fn default_scopes() -> String {
    "openid email profile api".to_owned()
}

// Why: Salesforce SSO is the only way a non-operator account comes into
// existence, so this list is the whole provisioning gate. Operators are created
// out-of-band with `admin users create` and enrol a passkey.
pub(super) fn default_allowed_domains() -> Vec<String> {
    vec![
        "astounddigital.com".to_owned(),
        "astoundcommerce.com".to_owned(),
    ]
}

// Why: Default off. An allow-listed domain says an address *could* belong to
// someone who should have access; it does not say anyone approved them. Closed
// enrolment is the enterprise posture (REQ-002), so provisioning is an
// admin act — via an invite, or `admin users create` — and both SSO first
// login and the passkey door refuse to create an account on their own.
pub(super) const fn default_auto_provision() -> bool {
    false
}

// Why: Default off, for the same reason as `auto_provision`. With this off the
// passkey door stops being a self-service front door: it refuses to create an
// account, and an unapproved address gets nowhere. Existing account holders
// still sign in normally; new arrivals come through an invite, which carries
// its own authorization and so bypasses the domain list entirely.
pub(super) const fn default_allow_self_registration() -> bool {
    false
}

/// Salesforce SSO connection config.
///
/// The client *secret* is never stored here — it is read from the
/// `SALESFORCE_CLIENT_SECRET` environment variable at callback/refresh time.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SalesforceConfig {
    #[serde(default)]
    pub enabled: bool,
    // Why: The org's My Domain base URL, e.g. `https://astound.my.salesforce.com`.
    // Doubles as the federated-identity `issuer` key.
    pub my_domain: String,
    #[serde(alias = "client_id")]
    pub consumer_key: String,
    pub redirect_uri: String,
    #[serde(default = "default_scopes")]
    pub scopes: String,
    #[serde(default = "default_allowed_domains")]
    pub allowed_email_domains: Vec<String>,
    // Why: Whether a verified, allow-listed first-time login auto-creates a local
    // account. When `false`, SSO only logs in / links *existing* users and an
    // unknown user is rejected (admin must pre-create the account).
    #[serde(default = "default_auto_provision")]
    pub auto_provision: bool,
    // Why: Whether `POST /admin/auth/passkey/register` may create a new account.
    // When `false` the endpoint refuses outright and an invite is the only
    // route in — see `handlers::passkey_auth`.
    #[serde(default = "default_allow_self_registration")]
    pub allow_self_registration: bool,
}

impl SalesforceConfig {
    // Why: A disabled placeholder used when no `salesforce.yaml` is present, so the
    // routes can still be registered and report "unavailable" cleanly.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            my_domain: String::new(),
            consumer_key: String::new(),
            redirect_uri: String::new(),
            scopes: default_scopes(),
            allowed_email_domains: default_allowed_domains(),
            auto_provision: default_auto_provision(),
            allow_self_registration: default_allow_self_registration(),
        }
    }

    pub(super) const fn is_usable(&self) -> bool {
        self.enabled
            && !self.my_domain.is_empty()
            && !self.consumer_key.is_empty()
            && !self.redirect_uri.is_empty()
    }

    fn base(&self) -> &str {
        self.my_domain.trim_end_matches('/')
    }

    pub(super) fn authorize_url(&self) -> String {
        format!("{}/services/oauth2/authorize", self.base())
    }

    pub fn token_url(&self) -> String {
        format!("{}/services/oauth2/token", self.base())
    }

    pub fn jwt_bearer_audience(&self) -> &str {
        self.base()
    }

    pub(super) fn userinfo_url(&self) -> String {
        format!("{}/services/oauth2/userinfo", self.base())
    }

    // Why: The `issuer` value recorded in `federated_identities`.
    pub(super) fn issuer(&self) -> &str {
        self.base()
    }

    pub fn email_allowed(&self, email: &str) -> bool {
        email
            .rsplit('@')
            .next()
            .is_some_and(|domain| self.allowed_email_domains.iter().any(|d| d == domain))
    }
}
