//! Metadata API deploys over the REST resource.
//!
//! Split from [`client`](super::client) to keep each file under the extension
//! line ceiling; this half owns packaging and the deploy status poll, the other
//! half owns queries and sObject writes.
//!
//! The REST resource accepts JWT-format access tokens where the SOAP Metadata
//! API rejects them, which is the whole reason the External Client App can be
//! configured headlessly at all — those four sObjects are `createable: false`,
//! so metadata deploy is the only write path that exists for them.

use std::time::Duration;

use serde::Deserialize;

use super::client::{API_VERSION, Connection};
use crate::handlers::salesforce_auth::SalesforceError;

const DEPLOY_TIMEOUT: Duration = Duration::from_secs(300);
const DEPLOY_POLL_INTERVAL: Duration = Duration::from_secs(3);

impl Connection {
    /// Submit a metadata package and wait for it to finish.
    ///
    /// `check_only` runs Salesforce's full validation and writes nothing, which
    /// is what backs `apply --dry-run`.
    ///
    /// # Errors
    /// [`SalesforceError::TokenEndpoint`] if the submit is rejected,
    /// [`SalesforceError::Internal`] on timeout,
    /// [`SalesforceError::DeployResult`] on an unreadable result.
    pub async fn deploy(
        &self,
        files: &[(String, String)],
        check_only: bool,
    ) -> Result<DeployResult, SalesforceError> {
        let zip = build_zip(files)?;
        let id = self.submit_deploy(&zip, check_only).await?;
        self.await_deploy(&id).await
    }

    async fn submit_deploy(&self, zip: &[u8], check_only: bool) -> Result<String, SalesforceError> {
        let boundary = format!("----sp{:016x}", rand::random::<u64>());
        let options = serde_json::json!({
            "deployOptions": {
                "checkOnly": check_only,
                "singlePackage": true,
                "rollbackOnError": true,
            }
        });

        // Why: reqwest is built without the `multipart` feature, so the body is
        // assembled by hand.
        let mut body: Vec<u8> = Vec::new();
        body.extend_from_slice(
            format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"json\"\r\n\
                 Content-Type: application/json\r\n\r\n{options}\r\n"
            )
            .as_bytes(),
        );
        body.extend_from_slice(
            format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; \
                 filename=\"package.zip\"\r\nContent-Type: application/zip\r\n\r\n"
            )
            .as_bytes(),
        );
        body.extend_from_slice(zip);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

        let text = self
            .post_multipart(
                &format!("/services/data/v{API_VERSION}/metadata/deployRequest"),
                &boundary,
                body,
            )
            .await?;

        // JSON: protocol boundary — only the `id` field of Salesforce's deploy
        // response is needed; the rest of the envelope is opaque.
        serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| v.get("id").and_then(|i| i.as_str()).map(str::to_owned))
            .ok_or_else(|| {
                SalesforceError::Internal(format!("deploy submit returned no id: {text}"))
            })
    }

    async fn await_deploy(&self, id: &str) -> Result<DeployResult, SalesforceError> {
        let deadline = std::time::Instant::now() + DEPLOY_TIMEOUT;
        loop {
            let value = self
                .get_json_public(&format!(
                    "/services/data/v{API_VERSION}/metadata/deployRequest/{id}?includeDetails=true"
                ))
                .await?;
            let result = value.get("deployResult").cloned().unwrap_or(value);
            let parsed: DeployResult =
                serde_json::from_value(result).map_err(SalesforceError::DeployResult)?;
            if parsed.done {
                return Ok(parsed);
            }
            if std::time::Instant::now() > deadline {
                return Err(SalesforceError::Internal(format!(
                    "deploy {id} did not finish within {}s",
                    DEPLOY_TIMEOUT.as_secs()
                )));
            }
            tokio::time::sleep(DEPLOY_POLL_INTERVAL).await;
        }
    }
}

fn build_zip(files: &[(String, String)]) -> Result<Vec<u8>, SalesforceError> {
    use std::io::Write as _;

    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for (path, contents) in files {
            zip.start_file(path.clone(), options)
                .map_err(|e| SalesforceError::Zip {
                    path: path.clone(),
                    source: e,
                })?;
            zip.write_all(contents.as_bytes())
                .map_err(|e| SalesforceError::Zip {
                    path: path.clone(),
                    source: zip::result::ZipError::Io(e),
                })?;
        }
        zip.finish().map_err(|e| SalesforceError::Zip {
            path: "(finish)".to_owned(),
            source: e,
        })?;
    }
    Ok(cursor.into_inner())
}

/// The subset of Salesforce's `deployResult` this tool acts on.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployResult {
    pub id: String,
    pub done: bool,
    pub success: bool,
    pub status: String,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub details: Option<DeployDetails>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployDetails {
    #[serde(default, deserialize_with = "one_or_many")]
    pub component_failures: Vec<ComponentMessage>,
    #[serde(default, deserialize_with = "one_or_many")]
    pub component_successes: Vec<ComponentMessage>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentMessage {
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub component_type: Option<String>,
    #[serde(default)]
    pub problem: Option<String>,
}

// Why: Salesforce collapses a single-element list into a bare object, so both
// shapes have to decode.
fn one_or_many<'de, D>(deserializer: D) -> Result<Vec<ComponentMessage>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        Many(Vec<ComponentMessage>),
        One(Box<ComponentMessage>),
        Null,
    }
    match OneOrMany::deserialize(deserializer)? {
        OneOrMany::Many(v) => Ok(v),
        OneOrMany::One(v) => Ok(vec![*v]),
        OneOrMany::Null => Ok(Vec::new()),
    }
}

impl DeployResult {
    /// Every component-level failure, formatted one per line.
    #[must_use]
    pub fn failure_lines(&self) -> Vec<String> {
        self.details
            .iter()
            .flat_map(|d| d.component_failures.iter())
            .map(|f| {
                format!(
                    "{} [{}]: {}",
                    f.full_name.as_deref().unwrap_or("?"),
                    f.component_type.as_deref().unwrap_or("?"),
                    f.problem.as_deref().unwrap_or("unknown problem")
                )
            })
            .collect()
    }
}
