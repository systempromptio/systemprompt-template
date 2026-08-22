//! Make an org match an [`OrgSpec`].
//!
//! Work splits by what Salesforce actually permits, which is not uniform, and
//! each half has its own sub-module:
//!
//! - **Metadata deploy** ([`package`]) for the External Client App and its
//!   OAuth settings and policies. Those four sObjects report `createable:
//!   false`, so there is no REST write path — the Metadata API is the only way
//!   in.
//! - **REST writes** ([`permissions`]) for permission sets, the
//!   `SetupEntityAccess` grants that pre-authorize the app, and the
//!   `PermissionSetAssignment` rows that put users inside it. These are
//!   ordinary createable sObjects.
//! - **Tooling writes** ([`hosted_mcp`]) for the standard hosted MCP servers.
//!   `McpServerAccess` is `updateable: true` from API version 67.0, so
//!   activation is a PATCH rather than the Setup click it used to be.
//!
//! # Ordering is load-bearing
//!
//! Permission sets, grants and assignments all run *before* the metadata
//! deploy. The deploy is what flips `permittedUsersPolicyType` to
//! `AdminApprovedPreAuthorized`, and from that moment only holders of the
//! permission set can authenticate. Deploying first opens a window in which
//! nobody — including the operator running this command — holds it yet.

pub mod hosted_mcp;
pub mod lookup;
pub mod package;
pub mod permissions;

pub use hosted_mcp::apply_hosted_mcp_servers;
pub use package::build_package;
pub use permissions::{apply_assignments, apply_permission_sets};

use super::client::Connection;
use super::deploy::DeployResult;
use super::spec::OrgSpec;
use crate::handlers::salesforce_auth::SalesforceError;

/// What an apply did, or would do.
#[derive(Debug, Default)]
pub struct ApplyReport {
    pub deploy: Option<DeployResult>,
    pub permission_sets_created: Vec<String>,
    pub app_grants_created: Vec<String>,
    pub assignments_created: Vec<String>,
    pub servers_activated: Vec<String>,
    pub manual_followups: Vec<String>,
}

// Why: With `check_only` the deploy is validated in full and nothing is
// written, which is what `--dry-run` uses. Salesforce reports component-level
// failures either way.
//
// Propagates deploy failures. A deploy that runs but reports component errors
// returns `Ok` with an unsuccessful [`DeployResult`] — inspect
// [`DeployResult::failure_lines`].
pub async fn apply_metadata(
    conn: &Connection,
    spec: &OrgSpec,
    certificate: Option<&str>,
    check_only: bool,
) -> Result<DeployResult, SalesforceError> {
    check_certificate_present(certificate)?;
    let package = build_package(spec, certificate);
    conn.deploy(&package, check_only).await
}

// Why: A metadata deploy is declarative: `certificate` is in schema on
// `ExtlClntAppGlobalOauthSettings`, so a package that omits it clears the
// digital signature — and the JWT-bearer grant this whole tool authenticates
// with then fails with `invalid_grant: invalid assertion`. The certificate is
// not readable back through any API, so apply cannot preserve it by round-trip
// and must be given it.
//
// This is a guard, not a fix: it converts a silent, self-inflicted lockout
// into a refusal before anything is sent.
//
// [`SalesforceError::Internal`] naming the variable to set.
pub fn check_certificate_present(certificate: Option<&str>) -> Result<(), SalesforceError> {
    if certificate.is_some_and(|c| !c.trim().is_empty()) {
        return Ok(());
    }
    Err(SalesforceError::Internal(
        "refusing to deploy: SF_TARGET_CERTIFICATE is not set. A metadata deploy is \
         declarative, so a package without <certificate> clears the External Client App's \
         digital signature and the JWT-bearer grant stops working (invalid_grant: invalid \
         assertion). Set SF_TARGET_CERTIFICATE to the PEM certificate matching \
         SF_TARGET_PRIVATE_KEY."
            .to_owned(),
    ))
}
