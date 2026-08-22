//! The org being read or configured, and where its credentials come from.

use crate::handlers::salesforce_auth::SalesforceError;

/// Connection details for the org being read or configured.
///
/// Deliberately separate from `SalesforceConfig`: that describes *this*
/// deployment's SSO client, whereas this describes an arbitrary target org.
#[derive(Clone)]
pub struct TargetOrg {
    pub my_domain: String,
    pub consumer_key: String,
    // Why: Salesforce *Username* to act as. Not the email; the two differ and
    // Salesforce matches the assertion `sub` on the Username.
    pub jwt_subject: String,
    pub private_key_pem: String,
    // Why: Required to *apply*, unused to export or diff. A metadata deploy is
    // declarative and `certificate` is in schema on
    // `ExtlClntAppGlobalOauthSettings`, so a package that omits it clears the
    // app's digital signature — and with it the JWT-bearer grant this type
    // authenticates with.
    pub certificate_pem: Option<String>,
}

impl TargetOrg {
    pub fn from_env() -> Result<Self, SalesforceError> {
        fn var(name: &str) -> Result<String, SalesforceError> {
            std::env::var(name).map_err(|e| SalesforceError::Env {
                name: name.to_owned(),
                source: e,
            })
        }
        Ok(Self {
            my_domain: var("SF_TARGET_MY_DOMAIN")?.trim_end_matches('/').to_owned(),
            consumer_key: var("SF_TARGET_CONSUMER_KEY")?,
            jwt_subject: var("SF_TARGET_JWT_SUBJECT")?,
            private_key_pem: var("SF_TARGET_PRIVATE_KEY")?,
            // Why: optional here rather than required, so export and diff still
            // work without it. Apply checks for it and refuses.
            //
            // Falls back to the platform's own certificate — env var, then the
            // profile's secrets store — so configuring the org this deployment
            // already talks to needs no extra plumbing. SF_TARGET_CERTIFICATE
            // stays available for pointing at a *different* org, matching how
            // the other SF_TARGET_* values work.
            certificate_pem: std::env::var("SF_TARGET_CERTIFICATE")
                .ok()
                .or_else(crate::handlers::salesforce_auth::salesforce_certificate),
        })
    }

    pub(super) fn token_url(&self) -> String {
        format!("{}/services/oauth2/token", self.my_domain)
    }
}

// Why: hand-written rather than derived because the struct holds an RSA
// private key. A derived Debug would print it in full anywhere the value is
// formatted or attached to a tracing span.
impl std::fmt::Debug for TargetOrg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TargetOrg")
            .field("my_domain", &self.my_domain)
            .field("consumer_key", &"<redacted>")
            .field("jwt_subject", &self.jwt_subject)
            .field("private_key_pem", &"<redacted>")
            .field("certificate_pem", &self.certificate_pem.is_some())
            .finish()
    }
}
