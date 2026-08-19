//! sObject writes over the REST and Tooling APIs.
//!
//! Create, update and delete for both API surfaces, selected by the `tooling`
//! flag rather than by separate methods — the paths differ only in a prefix.

use super::{API_VERSION, Connection};
use crate::handlers::salesforce_auth::SalesforceError;

// JSON: protocol boundary — sObject request bodies carry per-object field sets
// assembled by callers; there is no fixed shape to type here.
impl Connection {
    /// Create an sObject record, returning its new id.
    ///
    /// # Errors
    /// [`SalesforceError::TokenEndpoint`] carrying Salesforce's error body on a
    /// non-2xx — which is where validation failures surface.
    pub async fn create_sobject(
        &self,
        sobject: &str,
        body: &serde_json::Value,
        tooling: bool,
    ) -> Result<String, SalesforceError> {
        let prefix = if tooling { "tooling/" } else { "" };
        let resp = self
            .http
            .post(format!(
                "{}/services/data/v{API_VERSION}/{prefix}sobjects/{sobject}",
                self.instance_url
            ))
            .bearer_auth(&self.access_token)
            .json(body)
            .send()
            .await?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(SalesforceError::TokenEndpoint { status, body: text });
        }
        // JSON: only the `id` field of the create response is needed.
        serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| v.get("id").and_then(|i| i.as_str()).map(str::to_owned))
            .ok_or_else(|| {
                SalesforceError::Internal(format!("create {sobject} returned no id: {text}"))
            })
    }

    /// Update an sObject record in place.
    ///
    /// Salesforce answers a successful PATCH with 204 and an empty body, so
    /// there is nothing to return.
    ///
    /// # Errors
    /// [`SalesforceError::TokenEndpoint`] carrying Salesforce's error body on a
    /// non-2xx.
    // JSON: sObject bodies carry per-object field sets assembled by callers.
    pub async fn update_sobject(
        &self,
        sobject: &str,
        id: &str,
        body: &serde_json::Value,
        tooling: bool,
    ) -> Result<(), SalesforceError> {
        let prefix = if tooling { "tooling/" } else { "" };
        let resp = self
            .http
            .patch(format!(
                "{}/services/data/v{API_VERSION}/{prefix}sobjects/{sobject}/{id}",
                self.instance_url
            ))
            .bearer_auth(&self.access_token)
            .json(body)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(SalesforceError::TokenEndpoint { status, body });
        }
        Ok(())
    }

    /// Delete an sObject record.
    ///
    /// # Errors
    /// [`SalesforceError::TokenEndpoint`] on a non-2xx.
    pub async fn delete_sobject(
        &self,
        sobject: &str,
        id: &str,
        tooling: bool,
    ) -> Result<(), SalesforceError> {
        let prefix = if tooling { "tooling/" } else { "" };
        let resp = self
            .http
            .delete(format!(
                "{}/services/data/v{API_VERSION}/{prefix}sobjects/{sobject}/{id}",
                self.instance_url
            ))
            .bearer_auth(&self.access_token)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(SalesforceError::TokenEndpoint { status, body });
        }
        Ok(())
    }
}
