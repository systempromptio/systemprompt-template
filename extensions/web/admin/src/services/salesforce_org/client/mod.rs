//! Authenticated Salesforce client: REST, Tooling and Metadata deploy.
//!
//! Auth is the RFC 7523 JWT-bearer grant, reusing
//! `crate::services::salesforce_jwt_bearer`. No
//! browser, no refresh token, nothing to rotate but the certificate.
//!
//! The Metadata *deploy* REST resource accepts JWT-format access tokens, which
//! the SOAP Metadata API does not — SOAP rejects them with "SOAP API does not
//! support JWT-based access tokens". That single fact is why this module exists
//! instead of shelling out to the `sf` CLI: deploy over REST keeps the whole
//! loop headless with the credentials the platform already holds.
//!
//! [`TargetOrg`] and its credentials live in the private `target` module; the
//! sObject write methods live in the private `sobject` module.

mod sobject;
mod target;

pub use target::TargetOrg;

use crate::handlers::salesforce_auth::SalesforceError;
use crate::services::salesforce_jwt_bearer;

/// Salesforce API version for REST and Tooling *resource paths*.
///
/// Independent of [`METADATA_VERSION`] despite holding the same value today.
/// This one only decides which `/services/data/vNN.0/` URLs are called, so it
/// governs which sObjects exist — `McpServerAccess`, for one, appears at 67.0.
pub const API_VERSION: &str = "67.0";

/// Metadata API *schema* version, emitted as `<version>` in `package.xml`.
///
/// Separate from [`API_VERSION`] because this one selects a schema, not a URL:
/// it decides which elements a deployed component may carry. Bumping it is a
/// deliberate act — the deploy is declarative, so an element that comes newly
/// into scope and is then omitted takes its default rather than being left
/// alone. See `deploy/salesforce/README.md` for the probe method that
/// establishes the accepted element set for a version.
pub const METADATA_VERSION: &str = "67.0";

/// A live, authenticated connection to one org.
pub struct Connection {
    access_token: String,
    instance_url: String,
    http: reqwest::Client,
}

// Why: same reason as TargetOrg — this one holds a live bearer token.
impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection")
            .field("instance_url", &self.instance_url)
            .field("access_token", &"<redacted>")
            .field("http", &"reqwest::Client")
            .finish()
    }
}

impl Connection {
    /// Mint a token and open a connection.
    ///
    /// # Errors
    /// Propagates signing and token-endpoint failures from
    /// `salesforce_jwt_bearer::get_token_with_key`.
    pub async fn connect(target: &TargetOrg) -> Result<Self, SalesforceError> {
        let token = salesforce_jwt_bearer::get_token_with_key(
            &target.consumer_key,
            &target.jwt_subject,
            &target.my_domain,
            &target.token_url(),
            &target.private_key_pem,
        )
        .await?;
        Ok(Self {
            access_token: token.access_token,
            instance_url: token.instance_url,
            http: reqwest::Client::new(),
        })
    }

    /// The instance the token is scoped to.
    #[must_use]
    pub fn instance_url(&self) -> &str {
        &self.instance_url
    }

    // JSON: protocol boundary — the JSON-returning methods below hand back raw
    // Salesforce REST/Tooling responses; the projections vary per call, so
    // there is no fixed shape to deserialize here.
    // Why: A raw authenticated GET returning JSON, exposed to the sibling
    // `deploy` module so it can poll the deploy-status resource. Errors with
    // TokenEndpoint on a non-2xx.
    pub(in crate::services::salesforce_org) async fn get_json_public(
        &self,
        path: &str,
    ) -> Result<serde_json::Value, SalesforceError> {
        self.get_json(path).await
    }

    // Why: POST a pre-assembled multipart body and return the raw response
    // text. reqwest is built here without the `multipart` feature, so callers
    // assemble the body and this only attaches the boundary header. Errors with
    // TokenEndpoint on a non-2xx.
    pub(in crate::services::salesforce_org) async fn post_multipart(
        &self,
        path: &str,
        boundary: &str,
        body: Vec<u8>,
    ) -> Result<String, SalesforceError> {
        let resp = self
            .http
            .post(format!("{}{path}", self.instance_url))
            .bearer_auth(&self.access_token)
            .header(
                "Content-Type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(body)
            .send()
            .await?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if status.is_success() {
            Ok(text)
        } else {
            Err(SalesforceError::TokenEndpoint { status, body: text })
        }
    }

    // JSON: raw Salesforce response envelope — shape varies per resource.
    async fn get_json(&self, path: &str) -> Result<serde_json::Value, SalesforceError> {
        let resp = self
            .http
            .get(format!("{}{path}", self.instance_url))
            .bearer_auth(&self.access_token)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(SalesforceError::TokenEndpoint { status, body });
        }
        resp.json().await.map_err(SalesforceError::Http)
    }

    /// Run a SOQL query against the REST API, following `nextRecordsUrl` so the
    /// caller always sees the whole result set rather than the first page.
    ///
    /// # Errors
    /// [`SalesforceError::TokenEndpoint`] on a non-2xx,
    /// [`SalesforceError::Http`] on transport or decode failure.
    // JSON: query rows carry only the fields the SOQL projection named.
    pub async fn soql(&self, query: &str) -> Result<Vec<serde_json::Value>, SalesforceError> {
        self.query_paged(query, false).await
    }

    /// As [`soql`](Self::soql), against the Tooling API.
    ///
    /// # Errors
    /// Same as [`soql`](Self::soql).
    pub async fn tooling_soql(
        &self,
        query: &str,
    ) -> Result<Vec<serde_json::Value>, SalesforceError> {
        self.query_paged(query, true).await
    }

    // JSON: query rows carry only the fields the SOQL projection named.
    async fn query_paged(
        &self,
        query: &str,
        tooling: bool,
    ) -> Result<Vec<serde_json::Value>, SalesforceError> {
        let prefix = if tooling { "tooling/" } else { "" };
        let mut path = format!(
            "/services/data/v{API_VERSION}/{prefix}query/?q={}",
            urlencoding::encode(query)
        );
        let mut out = Vec::new();
        loop {
            let page = self.get_json(&path).await?;
            if let Some(records) = page.get("records").and_then(|r| r.as_array()) {
                out.extend(records.iter().cloned());
            }
            match page.get("nextRecordsUrl").and_then(|u| u.as_str()) {
                Some(next) => next.clone_into(&mut path),
                None => return Ok(out),
            }
        }
    }
}
