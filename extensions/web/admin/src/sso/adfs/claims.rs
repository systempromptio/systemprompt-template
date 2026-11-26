//! The claims a verified assertion carries, read with the claim URIs the
//! [`AdfsConfig`] names. Core's identity policy gates them and resolves the
//! account; this module only says what the assertion asserted.

use saml::{Identity, NameIdFormat};

use systemprompt_web_shared::adfs::AdfsConfig;

/// The subset of a verified assertion this login consumes: the subject and
/// the attribute statement, with the `IdP`'s claim URIs left as-is.
#[derive(Debug, Clone, Default)]
pub struct AssertionClaims {
    pub name_id: Option<String>,
    pub name_id_is_email: bool,
    pub attributes: Vec<(String, Vec<String>)>,
}

impl AssertionClaims {
    #[must_use]
    pub fn from_identity(identity: &Identity) -> Self {
        Self {
            name_id: Some(identity.name_id.value.clone()),
            name_id_is_email: identity.name_id.format == NameIdFormat::EmailAddress,
            attributes: identity
                .attributes
                .iter()
                .map(|a| (a.name.clone(), a.values.clone()))
                .collect(),
        }
    }

    // Why: Every value under the named attribute, in assertion order. AD FS
    // emits one `<AttributeValue>` per group.
    #[must_use]
    pub fn values(&self, attribute: &str) -> Vec<String> {
        self.attributes
            .iter()
            .filter(|(name, _)| name == attribute)
            .flat_map(|(_, values)| values.iter().cloned())
            .collect()
    }

    fn first(&self, attribute: &str) -> Option<String> {
        self.values(attribute)
            .into_iter()
            .map(|v| v.trim().to_owned())
            .find(|v| !v.is_empty())
    }

    // Why: The sign-in email: the configured email attribute, else an
    // email-format NameID. Lower-cased, since AD is case-insensitive and
    // `users.email` is stored lower.
    #[must_use]
    pub fn login_email(&self, cfg: &AdfsConfig) -> Option<String> {
        self.first(&cfg.email_attribute)
            .or_else(|| {
                self.name_id
                    .as_deref()
                    .filter(|_| self.name_id_is_email)
                    .map(ToOwned::to_owned)
            })
            .map(|e| e.trim().to_lowercase())
            .filter(|e| e.contains('@'))
    }

    #[must_use]
    pub fn display_name(&self, cfg: &AdfsConfig) -> Option<String> {
        self.first(&cfg.name_attribute)
    }

    // Why: The stable external subject: the NameID when the IdP sends one,
    // else the email — AD FS relying parties are routinely configured with
    // claims but no NameID, and the login email is the next-best stable key.
    #[must_use]
    pub fn external_sub(&self, email: &str) -> String {
        self.name_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map_or_else(|| email.to_owned(), ToOwned::to_owned)
    }
}
