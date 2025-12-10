use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::super::backend::ProxyError;

#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn send(&self, request: reqwest::RequestBuilder)
        -> Result<reqwest::Response, ProxyError>;
}

#[derive(Debug, Clone)]
pub struct ReqwestClient {
    #[allow(dead_code)] // Used implicitly by reqwest::RequestBuilder
    client: reqwest::Client,
}

impl ReqwestClient {
    pub const fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn send(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, ProxyError> {
        let response = request
            .send()
            .await
            .map_err(|e| ProxyError::ConnectionFailed {
                service: "unknown".to_string(),
                url: "unknown".to_string(),
                source: e,
            })?;

        Ok(response)
    }
}

#[derive(Debug)]
pub struct MockClient {
    responses: Arc<Mutex<VecDeque<Result<reqwest::Response, ProxyError>>>>,
}

impl MockClient {
    pub fn new(responses: Vec<Result<reqwest::Response, ProxyError>>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(VecDeque::from(responses))),
        }
    }

    pub fn empty() -> Self {
        Self::new(vec![])
    }
}

#[async_trait]
impl HttpClient for MockClient {
    async fn send(
        &self,
        _request: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, ProxyError> {
        let mut responses = self.responses.lock().await;
        responses.pop_front().unwrap_or_else(|| {
            Err(ProxyError::InvalidResponse {
                service: "mock".to_string(),
                reason: "No more mocked responses available".to_string(),
            })
        })
    }
}
