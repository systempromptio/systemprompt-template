//! OAuth scope vocabulary.
//!
//! Salesforce names the same scope three different ways and this module is the
//! single place that reconciles them:
//!
//! | surface | example |
//! |---|---|
//! | `ExtlClntAppOauthSettings` sObject field | `OauthScopesMCP_API` |
//! | Metadata API `commaSeparatedOauthScopes` token | `MCP` |
//! | this repository's YAML | `mcp` |
//!
//! The metadata tokens were read back from a live org: submitting an invalid
//! scope makes Salesforce enumerate the valid set in its error, which is where
//! this list comes from. Getting one wrong fails the deploy loudly rather than
//! silently granting the wrong access, so the mapping is exhaustive and typed.

use serde::{Deserialize, Serialize};

/// A single OAuth scope grantable to an External Client App.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OauthScope {
    // Why: Identity URL access. Salesforce calls this `Basic`, the sObject calls it
    // `SSO`, and the Setup UI calls it "Access the identity URL service".
    Basic,
    Api,
    Web,
    Full,
    RefreshToken,
    OfflineAccess,
    OpenId,
    Profile,
    Email,
    Address,
    Phone,
    CustomPermissions,
    CustomApplications,
    Content,
    Lightning,
    Chatter,
    Wave,
    Eclair,
    Pardot,
    Interaction,
    ForgotPassword,
    UserRegistration,
    PwdlessLogin,
    EinsteinGpt,
    SfApiPlatform,
    Scrt,
    Chatbot,
    // Why: Salesforce Hosted MCP. This is the scope the platform's JWT-bearer mint
    // depends on; dropping it breaks every MCP tool call.
    Mcp,
    Cdp,
    CdpQuery,
    CdpProfile,
    CdpIngest,
    CdpSegment,
    CdpIdentityResolution,
    CdpCalculatedInsight,
    DataCloudUserClaims,
}

impl OauthScope {
    #[must_use]
    pub const fn metadata_token(self) -> &'static str {
        match self {
            Self::Basic => "Basic",
            Self::Api => "Api",
            Self::Web => "Web",
            Self::Full => "Full",
            Self::RefreshToken => "RefreshToken",
            Self::OfflineAccess => "OfflineAccess",
            Self::OpenId => "OpenID",
            Self::Profile => "Profile",
            Self::Email => "Email",
            Self::Address => "Address",
            Self::Phone => "Phone",
            Self::CustomPermissions => "CustomPermissions",
            Self::CustomApplications => "CustomApplications",
            Self::Content => "Content",
            Self::Lightning => "Lightning",
            Self::Chatter => "Chatter",
            Self::Wave => "Wave",
            Self::Eclair => "Eclair",
            Self::Pardot => "Pardot",
            Self::Interaction => "Interaction",
            Self::ForgotPassword => "ForgotPassword",
            Self::UserRegistration => "UserRegistration",
            Self::PwdlessLogin => "PwdlessLogin",
            Self::EinsteinGpt => "EinsteinGPT",
            Self::SfApiPlatform => "SFApiPlatform",
            Self::Scrt => "SCRT",
            Self::Chatbot => "Chatbot",
            Self::Mcp => "MCP",
            Self::Cdp => "CDP",
            Self::CdpQuery => "CDPQuery",
            Self::CdpProfile => "CDPProfile",
            Self::CdpIngest => "CDPIngest",
            Self::CdpSegment => "CDPSegment",
            Self::CdpIdentityResolution => "CDPIdentityResolution",
            Self::CdpCalculatedInsight => "CDPCalculatedInsight",
            Self::DataCloudUserClaims => "DataCloudUserClaims",
        }
    }

    #[must_use]
    pub const fn sobject_field(self) -> &'static str {
        match self {
            Self::Basic => "OauthScopesSSO",
            Self::Api => "OauthScopesAPI",
            Self::Web => "OauthScopesWEB",
            Self::Full => "OauthScopesFULL",
            Self::RefreshToken => "OauthScopesREFRESH_TOKEN",
            Self::OfflineAccess => "OauthScopesOFFLINE_ACCESS",
            Self::OpenId => "OauthScopesOPENID",
            Self::Profile => "OauthScopesPROFILE",
            Self::Email => "OauthScopesEMAIL",
            Self::Address => "OauthScopesADDRESS",
            Self::Phone => "OauthScopesPHONE",
            Self::CustomPermissions => "OauthScopesCUSTOM_PERMISSIONS",
            Self::CustomApplications => "OauthScopesVF",
            Self::Content => "OauthScopesCONTENT",
            Self::Lightning => "OauthScopesLIGHTNING",
            Self::Chatter => "OauthScopesCHATTER_REST_API",
            Self::Wave => "OauthScopesWAVE_REST_API",
            Self::Eclair => "OauthScopesECLAIR_REST_API",
            Self::Pardot => "OauthScopesPARDOT_API",
            Self::Interaction => "OauthScopesINTERACTION_API",
            Self::ForgotPassword => "OauthScopesFORGOT_PASSWORD",
            Self::UserRegistration => "OauthScopesUSER_REGISTRATION_API",
            Self::PwdlessLogin => "OauthScopesPWDLESS_LOGIN_API",
            Self::EinsteinGpt => "OauthScopesEINSTEIN_GPT_API",
            Self::SfApiPlatform => "OauthScopesSFAP_API",
            Self::Scrt => "OauthScopesSCRT_API",
            Self::Chatbot => "OauthScopesCHATBOT_API",
            Self::Mcp => "OauthScopesMCP_API",
            Self::Cdp => "OauthScopesCDP_API",
            Self::CdpQuery => "OauthScopesCDP_QUERY_API",
            Self::CdpProfile => "OauthScopesCDP_PROFILE_API",
            Self::CdpIngest => "OauthScopesCDP_INGEST_API",
            Self::CdpSegment => "OauthScopesCDP_SEGMENT_API",
            Self::CdpIdentityResolution => "OauthScopesCDP_IDENTITYRESOLUTION_API",
            Self::CdpCalculatedInsight => "OauthScopesCDP_CALCULATED_INSIGHT_API",
            Self::DataCloudUserClaims => "OauthScopesDATA_CLOUD_USER_CLAIMS",
        }
    }

    // Why: Every scope, in a stable order. Export iterates this to turn the
    // sObject's boolean columns back into a scope set.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Basic,
            Self::Api,
            Self::Web,
            Self::Full,
            Self::RefreshToken,
            Self::OfflineAccess,
            Self::OpenId,
            Self::Profile,
            Self::Email,
            Self::Address,
            Self::Phone,
            Self::CustomPermissions,
            Self::CustomApplications,
            Self::Content,
            Self::Lightning,
            Self::Chatter,
            Self::Wave,
            Self::Eclair,
            Self::Pardot,
            Self::Interaction,
            Self::ForgotPassword,
            Self::UserRegistration,
            Self::PwdlessLogin,
            Self::EinsteinGpt,
            Self::SfApiPlatform,
            Self::Scrt,
            Self::Chatbot,
            Self::Mcp,
            Self::Cdp,
            Self::CdpQuery,
            Self::CdpProfile,
            Self::CdpIngest,
            Self::CdpSegment,
            Self::CdpIdentityResolution,
            Self::CdpCalculatedInsight,
            Self::DataCloudUserClaims,
        ]
    }

    #[must_use]
    pub fn soql_projection() -> String {
        Self::all()
            .iter()
            .map(|s| s.sobject_field())
            .collect::<Vec<_>>()
            .join(",")
    }
}

// Why: `OauthScopesHUB_API` exists on the sObject but has no counterpart in the
// metadata token list Salesforce returns, so it cannot round-trip. Export
// warns rather than dropping it silently.
pub const UNMAPPED_SCOPE_FIELDS: &[&str] = &["OauthScopesHUB_API"];
