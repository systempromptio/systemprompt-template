//! ADFS SSO connection config, loaded from `services/web/config/adfs.yaml`.

use std::collections::BTreeMap;

use serde::Deserialize;


// Why: The claim URIs AD FS emits by default for the standard issuance
// templates. An installation whose issuance rules name them differently
// overrides them in `adfs.yaml`.
pub(super) fn default_email_attribute() -> String {
    "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress".to_owned()
}

pub(super) fn default_name_attribute() -> String {
    "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/name".to_owned()
}

pub(super) fn default_groups_attribute() -> String {
    "http://schemas.xmlsoap.org/claims/Group".to_owned()
}

// Why: ADFS SSO is the only way a non-operator account comes into existence,
// so this list is one half of the provisioning gate; the group map is the
// other. Operators are created out-of-band with `admin users create` and
// enrol a passkey.
pub(super) fn default_allowed_domains() -> Vec<String> {
    vec![
        "systempromptdigital.com".to_owned(),
        "systempromptcommerce.com".to_owned(),
    ]
}

// Why: Default off. An allow-listed domain says an address *could* belong to
// someone who should have access; it does not say anyone approved them. With
// ADFS the approval is the AD group — a login carrying a mapped group is one
// IT has already authorised — so an installation that trusts its group map
// turns this on. Off, SSO only signs in *existing* accounts.
pub(super) const fn default_auto_provision() -> bool {
    false
}

pub(super) const fn default_deny_without_group() -> bool {
    true
}

// Why: IdP-initiated sign-in — the user picks this app on the AD FS portal —
// is how most enterprise staff first reach an application. The assertion is
// verified exactly as a solicited one; only the request-id correlation is
// absent.
pub(super) const fn default_allow_idp_initiated() -> bool {
    true
}

pub(super) const fn default_clock_skew_seconds() -> u64 {
    120
}

// Why: The two SAML endpoints this instance serves. The loader appends them to
// the profile's external URL when the YAML names neither, so the registered
// relying-party trust and the values we send AD FS are derived from one fact.
pub const METADATA_PATH: &str = "/saml/metadata";
pub const ACS_PATH: &str = "/admin/auth/adfs/acs";

/// ADFS SSO connection config (SAML 2.0 relying party).
///
/// There is no client secret: trust runs on the `IdP`'s published signing
/// certificate, read from the federation metadata file named by
/// `idp_metadata_path` and pinned in the repository.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdfsConfig {
    #[serde(default)]
    pub enabled: bool,
    // Why: Our relying-party identifier — the `<Audience>` the IdP writes into
    // every assertion and the `<Issuer>` of our AuthnRequests. Also the
    // federated-identity `issuer`-side key, together with the IdP entity id.
    // Derived by the loader from the profile's external URL when the YAML
    // leaves it out, so one committed config is correct on every host.
    #[serde(default)]
    pub entity_id: String,
    // Why: Where AD FS posts the assertion; must match the ACS registered on
    // the relying-party trust character for character. Derived from the same
    // external URL as `entity_id`, so the two can never name different hosts.
    #[serde(default)]
    pub acs_url: String,
    // Why: The IdP federation metadata, relative to `services/web/config/`.
    // Committed so the signing certificate is pinned and rotation is a
    // reviewable diff, not a network fetch at sign-in time.
    pub idp_metadata_path: String,
    // Why: Filled by the loader from `idp_metadata_path`; never authored in
    // YAML.
    #[serde(skip)]
    pub idp_metadata_xml: String,
    #[serde(default = "default_allowed_domains")]
    pub allowed_email_domains: Vec<String>,
    #[serde(default = "default_auto_provision")]
    pub auto_provision: bool,
    #[serde(default = "default_email_attribute")]
    pub email_attribute: String,
    #[serde(default = "default_name_attribute")]
    pub name_attribute: String,
    #[serde(default = "default_groups_attribute")]
    pub groups_attribute: String,
    // Why: A login whose assertion carries no group AT ALL gets no session.
    // The AD group is the entitlement; without one there is nothing to grant.
    // A group this file's role map does not name is not a refusal — the DB
    // mapping decides membership, and the member signs in as a plain `user`.
    // Off only for evaluation instances where every domain user is welcome.
    #[serde(default = "default_deny_without_group")]
    pub deny_without_group: bool,
    #[serde(default = "default_allow_idp_initiated")]
    pub allow_idp_initiated: bool,
    #[serde(default = "default_clock_skew_seconds")]
    pub clock_skew_seconds: u64,
    // Why: AD group name → systemprompt roles. Re-projected onto the directory
    // half of `users.roles` on every login, so removing someone from the AD
    // group revokes the role at their next sign-in; roles an admin granted by
    // hand are a separate half and survive.
    #[serde(default)]
    pub group_roles: BTreeMap<String, Vec<String>>,
    // Why: AD group name *glob* → systemprompt roles, for entitlements the
    // directory spreads across many regional groups. Project managers arrive
    // as `Systemprompt-ProjectManagers-UK`, `-India`, `-Europe` and so on;
    // enumerating them here would go stale every time IT adds a region.
    // Applied on top of `group_roles`, never instead of it.
    #[serde(default)]
    pub group_role_patterns: BTreeMap<String, Vec<String>>,
}

// Why: one `*` wildcard is the whole grammar. AD group names are flat labels
// with a stable prefix, so a prefix/suffix match is all the directory shape
// asks for, and a full glob engine would be a dependency bought for nothing.
#[must_use]
pub fn group_matches_pattern(pattern: &str, group: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == group,
        Some((prefix, suffix)) => {
            group.len() >= prefix.len() + suffix.len()
                && group.starts_with(prefix)
                && group.ends_with(suffix)
        },
    }
}

impl AdfsConfig {
    // Why: A disabled placeholder used when no `adfs.yaml` is present, so the
    // routes can still be registered and report "unavailable" cleanly.
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            entity_id: String::new(),
            acs_url: String::new(),
            idp_metadata_path: String::new(),
            idp_metadata_xml: String::new(),
            allowed_email_domains: default_allowed_domains(),
            auto_provision: default_auto_provision(),
            email_attribute: default_email_attribute(),
            name_attribute: default_name_attribute(),
            groups_attribute: default_groups_attribute(),
            deny_without_group: default_deny_without_group(),
            allow_idp_initiated: default_allow_idp_initiated(),
            clock_skew_seconds: default_clock_skew_seconds(),
            group_roles: BTreeMap::new(),
            group_role_patterns: BTreeMap::new(),
        }
    }

    #[must_use]
    pub const fn is_usable(&self) -> bool {
        self.enabled
            && !self.entity_id.is_empty()
            && !self.acs_url.is_empty()
            && !self.idp_metadata_xml.is_empty()
    }

    #[must_use]
    pub fn email_allowed(&self, email: &str) -> bool {
        email
            .rsplit('@')
            .next()
            .is_some_and(|domain| self.allowed_email_domains.iter().any(|d| d == domain))
    }

    // Why: The roles an assertion's group list maps to, deduplicated and in a
    // stable order. Empty when no group is mapped.
    #[must_use]
    pub fn roles_for_groups(&self, groups: &[String]) -> Vec<String> {
        let mut roles: Vec<String> = groups
            .iter()
            .filter_map(|g| self.group_roles.get(g))
            .flatten()
            .cloned()
            .collect();
        roles.extend(
            self.group_role_patterns
                .iter()
                .filter(|(pattern, _)| groups.iter().any(|g| group_matches_pattern(pattern, g)))
                .flat_map(|(_, granted)| granted.iter().cloned()),
        );
        roles.sort();
        roles.dedup();
        roles
    }

    // Why: every group the assertion carried, in assertion order. Which DB
    // group or project a name resolves to is decided by `group_ad_mappings`,
    // and a name that resolves to none leaves the member in the derived
    // `unassigned` group. Filtering here would hide memberships the database
    // is able to place.
    #[must_use]
    #[expect(
        clippy::unused_self,
        reason = "the receiver is the seam: which groups survive is a config \
                  decision, and every call site already holds the config"
    )]
    pub fn mapped_groups(&self, groups: &[String]) -> Vec<String> {
        groups.to_vec()
    }
}
