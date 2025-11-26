use crate::models::cimd::CimdMetadata;
use anyhow::{anyhow, Result};
use reqwest::Client;
use std::time::Duration;

#[derive(Debug)]
pub struct CimdFetcher {
    client: Client,
}

impl CimdFetcher {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("SystemPrompt-OS/2.0")
            .redirect(reqwest::redirect::Policy::limited(3))
            .build()
            .expect("Failed to build HTTP client");

        Self { client }
    }

    pub async fn fetch_metadata(&self, client_id: &str) -> Result<CimdMetadata> {
        if !client_id.starts_with("https://") {
            return Err(anyhow!("CIMD client_id must be HTTPS URL"));
        }

        let response = self
            .client
            .get(client_id)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch CIMD metadata from {client_id}: {e}"))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch CIMD metadata: HTTP {} from {}",
                response.status(),
                client_id
            ));
        }

        let metadata: CimdMetadata = response
            .json()
            .await
            .map_err(|e| anyhow!("Invalid CIMD metadata JSON from {client_id}: {e}"))?;

        if metadata.client_id != client_id {
            return Err(anyhow!(
                "CIMD metadata client_id mismatch: expected '{}', got '{}'",
                client_id,
                metadata.client_id
            ));
        }

        metadata.validate()?;

        Ok(metadata)
    }
}

impl Default for CimdFetcher {
    fn default() -> Self {
        Self::new()
    }
}
