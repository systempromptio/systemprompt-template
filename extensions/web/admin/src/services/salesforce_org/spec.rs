//! The desired-state document: `services/salesforce/org.yaml`.
//!
//! This is the source of truth for what a Salesforce org should look like.
//! [`export`](super::export) produces one from a live org,
//! [`diff`](super::diff) compares two, and [`apply`](super::apply) makes an org
//! match one.
//!
//! Record ids, consumer keys and org ids are deliberately absent: they are
//! per-org and minted by Salesforce, so a spec carrying them could not be
//! applied to a second org.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::scope::OauthScope;

pub const SPEC_RELATIVE_PATH: &str = "salesforce/org.yaml";

#[derive(Debug, Clone, thiserror::Error)]
pub enum SpecError {
    #[error("salesforce org spec not found at {0}")]
    NotFound(PathBuf),
    #[error("failed to read {path}: {message}")]
    Read { path: PathBuf, message: String },
    #[error("failed to parse {path}: {message}")]
    Parse { path: PathBuf, message: String },
    #[error("failed to serialise org spec: {0}")]
    Serialise(String),
}

/// The whole desired state of an org.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrgSpec {
    pub external_client_app: ExternalClientApp,
    #[serde(default)]
    pub permission_sets: Vec<PermissionSetSpec>,
    // Why: Standard hosted MCP servers. Never *created* — an org either offers a
    // standard server or it does not — but their `Active` flag is read and
    // written through the Tooling `McpServerAccess` object.
    #[serde(default)]
    pub hosted_mcp_servers: Vec<HostedMcpServer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalClientApp {
    // Why: Immutable API name. Every dependent metadata record keys off this, so
    // renaming it creates a second app rather than renaming the first.
    pub developer_name: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub contact_email: String,
    // Why: Salesforce enum `ExtlClntAppDistState`. Salesforce does not publish the
    // value set and its parse error does not enumerate it; `Local` is verified
    // against a live org. Invalid values fail the deploy with a clear message.
    pub distribution_state: String,
    pub oauth: OauthSpec,
    pub policies: PolicySpec,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OauthSpec {
    // Why: Must match the platform's `redirect_uri` in `salesforce.yaml` exactly —
    // Salesforce compares them character for character.
    pub callback_url: String,
    pub scopes: Vec<OauthScope>,
    #[serde(default)]
    pub first_party_app_enabled: bool,
    #[serde(default = "default_true")]
    pub pkce_required: bool,
    #[serde(default)]
    pub consumer_secret_optional: bool,
    // Why: Declared explicitly rather than left implicit because the whole tool
    // depends on it: the SOAP Metadata API rejects JWT-format tokens but the
    // REST deploy resource accepts them, which is the only reason a headless
    // apply is possible at all. The element is out of schema at metadata
    // version 64.0 and in scope at 67.0, so from 67.0 a deploy that omitted it
    // would reset it and lock the tool out of the org it just configured.
    #[serde(default = "default_true")]
    pub named_user_jwt: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub single_logout_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySpec {
    // Why: Salesforce enum `PermittedUsersPolicyType`. `AdminApprovedPreAuthorized`
    // is verified live and is what gates access to a permission set.
    pub permitted_users: String,
    pub ip_relaxation: IpRelaxation,
    pub refresh_token_policy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token_validity: Option<Validity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_session_level: Option<String>,
}

/// Verified exhaustively: Salesforce's validation error names the full set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpRelaxation {
    Enforce,
    Bypass,
    #[serde(rename = "Bypass_2factor")]
    Bypass2Factor,
    #[serde(rename = "Enforce_relaxrefresh")]
    EnforceRelaxRefresh,
}

impl IpRelaxation {
    #[must_use]
    pub const fn metadata_token(self) -> &'static str {
        match self {
            Self::Enforce => "Enforce",
            Self::Bypass => "Bypass",
            Self::Bypass2Factor => "Bypass_2factor",
            Self::EnforceRelaxRefresh => "Enforce_relaxrefresh",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Validity {
    pub period: u32,
    pub unit: ValidityUnit,
}

/// Verified exhaustively: "Set the refresh token validity unit to Days, Hours,
/// Months."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidityUnit {
    Hours,
    Days,
    Months,
}

impl ValidityUnit {
    #[must_use]
    pub const fn metadata_token(self) -> &'static str {
        match self {
            Self::Hours => "Hours",
            Self::Days => "Days",
            Self::Months => "Months",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PermissionSetSpec {
    pub name: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    // Why: Developer name of the External Client App this permission set
    // pre-authorizes. This is the `SetupEntityAccess` grant, and it is what
    // makes `PermittedUsersPolicyType: AdminApprovedPreAuthorized` usable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grants_app: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostedMcpServer {
    pub name: String,
    // Why: `McpServerAccess.DeveloperName`, e.g. `platform_sobject_all`. The stable
    // key `apply` matches on — labels are translatable, developer names are
    // not.
    pub developer_name: String,
    pub endpoint: String,
    // Why: Desired `Active` state. Apply switches a server on; it never switches
    // one off, and a server absent from the org is an error rather than
    // something activation could fix.
    #[serde(default = "default_true")]
    pub active: bool,
}

const fn default_true() -> bool {
    true
}

impl OrgSpec {
    pub fn load(path: &Path) -> Result<Self, SpecError> {
        if !path.exists() {
            return Err(SpecError::NotFound(path.to_path_buf()));
        }
        let raw = std::fs::read_to_string(path).map_err(|e| SpecError::Read {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;
        serde_yaml::from_str(&raw).map_err(|e| SpecError::Parse {
            path: path.to_path_buf(),
            message: e.to_string(),
        })
    }

    pub fn to_yaml(&self) -> Result<String, SpecError> {
        serde_yaml::to_string(self).map_err(|e| SpecError::Serialise(e.to_string()))
    }
}
