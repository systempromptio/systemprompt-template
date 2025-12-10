use async_trait::async_trait;
use reqwest::header::HeaderMap;
use reqwest::{Client, Response};
use serde_json::Value;
use std::time::Duration;

use super::error::ClientResult;

#[async_trait]
pub trait Transport: Send + Sync {
    async fn send_request(&self, path: &str, body: Value) -> ClientResult<Response>;
}

#[derive(Debug)]
pub struct HttpTransport {
    client: Client,
    base_url: String,
    headers: HeaderMap,
}

impl HttpTransport {
    pub fn new(base_url: impl Into<String>) -> ClientResult<Self> {
        use reqwest::header::HeaderValue;

        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        Ok(Self {
            client,
            base_url: base_url.into(),
            headers,
        })
    }

    pub fn with_timeout(mut self, timeout: Duration) -> ClientResult<Self> {
        self.client = Client::builder().timeout(timeout).build()?;
        Ok(self)
    }

    pub fn with_auth_token(mut self, token: impl AsRef<str>) -> ClientResult<Self> {
        use reqwest::header::HeaderValue;

        let bearer_value = format!("Bearer {}", token.as_ref());
        let header_value = HeaderValue::from_str(&bearer_value).map_err(|e| {
            super::error::ClientError::invalid_response(format!("Invalid auth token header: {e}"))
        })?;
        self.headers.insert("Authorization", header_value);
        Ok(self)
    }

    fn build_url(&self, path: &str) -> String {
        if path.starts_with('/') {
            format!("{}{}", self.base_url, path)
        } else {
            format!("{}/{}", self.base_url, path)
        }
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn send_request(&self, path: &str, body: Value) -> ClientResult<Response> {
        let url = self.build_url(path);

        let response = self
            .client
            .post(&url)
            .headers(self.headers.clone())
            .json(&body)
            .send()
            .await?;

        Ok(response)
    }
}
