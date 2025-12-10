use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use systemprompt_models::execution::BroadcastEvent;

use super::broadcaster::CONTEXT_BROADCASTER;

#[async_trait]
pub trait BroadcastClient: Send + Sync {
    async fn broadcast(&self, user_id: &str, event: BroadcastEvent) -> usize;
}

#[derive(Debug, Clone, Copy)]
pub struct LocalBroadcaster;

impl LocalBroadcaster {
    pub const fn new() -> Self {
        Self
    }
}

impl Default for LocalBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BroadcastClient for LocalBroadcaster {
    async fn broadcast(&self, user_id: &str, event: BroadcastEvent) -> usize {
        CONTEXT_BROADCASTER.broadcast_to_user(user_id, event).await
    }
}

#[derive(Debug)]
pub struct WebhookBroadcaster {
    client: reqwest::Client,
    api_url: String,
    token: String,
}

impl WebhookBroadcaster {
    pub fn new(token: String) -> Self {
        let api_url = std::env::var("API_INTERNAL_URL")
            .or_else(|_| std::env::var("API_EXTERNAL_URL"))
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        Self {
            client: reqwest::Client::new(),
            api_url,
            token,
        }
    }

    pub fn with_url(token: String, api_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_url,
            token,
        }
    }
}

#[async_trait]
impl BroadcastClient for WebhookBroadcaster {
    async fn broadcast(&self, user_id: &str, event: BroadcastEvent) -> usize {
        let payload = json!({
            "event_type": event.event_type,
            "context_id": event.context_id,
            "user_id": user_id,
            "data": event.data,
            "timestamp": event.timestamp.to_rfc3339()
        });

        let result = self
            .client
            .post(format!("{}/api/v1/webhook/broadcast", self.api_url))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await;

        match result {
            Ok(response) if response.status().is_success() => 1,
            _ => 0,
        }
    }
}

pub fn create_webhook_broadcaster(token: &str) -> Arc<dyn BroadcastClient> {
    Arc::new(WebhookBroadcaster::new(token.to_string()))
}

pub fn create_local_broadcaster() -> Arc<dyn BroadcastClient> {
    Arc::new(LocalBroadcaster::new())
}
