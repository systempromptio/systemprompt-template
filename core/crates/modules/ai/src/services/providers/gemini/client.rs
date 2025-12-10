use anyhow::{anyhow, Result};
use reqwest::Client;

use super::constants::timeout;

pub fn build_client() -> Result<Client> {
    Client::builder()
        .timeout(timeout::REQUEST_TIMEOUT)
        .connect_timeout(timeout::CONNECT_TIMEOUT)
        .build()
        .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))
}
