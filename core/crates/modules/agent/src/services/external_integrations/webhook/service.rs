use hmac::{Hmac, Mac};
use reqwest::Client;
use serde_json::Value;
use sha2::Sha256;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::external_integrations::{
    IntegrationError, IntegrationResult, WebhookEndpoint, WebhookRequest, WebhookResponse,
};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug)]
pub struct WebhookService {
    endpoints: RwLock<HashMap<String, WebhookEndpoint>>,
    http_client: Client,
}

impl WebhookService {
    pub fn new() -> Self {
        Self {
            endpoints: RwLock::new(HashMap::new()),
            http_client: Client::new(),
        }
    }

    pub async fn register_endpoint(
        &self,
        mut endpoint: WebhookEndpoint,
    ) -> IntegrationResult<String> {
        if endpoint.id.is_empty() {
            endpoint.id = Uuid::new_v4().to_string();
        }

        let endpoint_id = endpoint.id.clone();

        {
            let mut endpoints = self.endpoints.write().await;
            endpoints.insert(endpoint_id.clone(), endpoint);
        }

        Ok(endpoint_id)
    }

    pub async fn update_endpoint(&self, endpoint: WebhookEndpoint) -> IntegrationResult<()> {
        {
            let mut endpoints = self.endpoints.write().await;
            endpoints.insert(endpoint.id.clone(), endpoint);
        }
        Ok(())
    }

    pub async fn get_endpoint(
        &self,
        endpoint_id: &str,
    ) -> IntegrationResult<Option<WebhookEndpoint>> {
        let endpoints = self.endpoints.read().await;
        Ok(endpoints.get(endpoint_id).cloned())
    }

    pub async fn list_endpoints(&self) -> IntegrationResult<Vec<WebhookEndpoint>> {
        let endpoints = self.endpoints.read().await;
        Ok(endpoints.values().cloned().collect())
    }

    pub async fn remove_endpoint(&self, endpoint_id: &str) -> IntegrationResult<bool> {
        let mut endpoints = self.endpoints.write().await;
        Ok(endpoints.remove(endpoint_id).is_some())
    }

    pub async fn handle_webhook(
        &self,
        endpoint_id: &str,
        request: WebhookRequest,
    ) -> IntegrationResult<WebhookResponse> {
        let endpoint = {
            let endpoints = self.endpoints.read().await;
            endpoints.get(endpoint_id).cloned().ok_or_else(|| {
                IntegrationError::Webhook(format!("Endpoint not found: {endpoint_id}"))
            })?
        };

        if !endpoint.active {
            return Ok(WebhookResponse {
                status: 404,
                body: Some(serde_json::json!({"error": "Endpoint is inactive"})),
            });
        }

        if let (Some(_secret), Some(signature)) = (&endpoint.secret, &request.signature) {
            if !self.verify_signature_internal(&endpoint, &request.body, signature)? {
                return Ok(WebhookResponse {
                    status: 401,
                    body: Some(serde_json::json!({"error": "Invalid signature"})),
                });
            }
        }

        let event_type = request
            .headers
            .get("x-webhook-event")
            .or_else(|| request.headers.get("x-event-type"))
            .or_else(|| request.headers.get("x-github-event"))
            .map(|s| s.clone())
            .unwrap_or_else(|| "unknown".to_string());

        if !endpoint.events.is_empty()
            && !endpoint.events.contains(&event_type)
            && !endpoint.events.contains(&"*".to_string())
        {
            return Ok(WebhookResponse {
                status: 200,
                body: Some(serde_json::json!({"message": "Event type not subscribed"})),
            });
        }

        Ok(WebhookResponse {
            status: 200,
            body: Some(serde_json::json!({
                "message": "Webhook processed successfully",
                "event_type": event_type,
                "endpoint_id": endpoint_id
            })),
        })
    }

    pub async fn send_webhook(
        &self,
        url: &str,
        payload: Value,
        config: Option<WebhookConfig>,
    ) -> IntegrationResult<WebhookDeliveryResult> {
        let config = config.unwrap_or_default();

        let mut request_builder = self
            .http_client
            .post(url)
            .json(&payload)
            .header("Content-Type", "application/json")
            .header("User-Agent", "SystemPrompt-Webhook/1.0");

        for (key, value) in &config.headers {
            request_builder = request_builder.header(key, value);
        }

        if let Some(secret) = &config.secret {
            let signature = self.generate_signature(secret, &payload)?;
            request_builder = request_builder.header("X-Webhook-Signature", signature);
        }

        if let Some(timeout) = config.timeout {
            request_builder = request_builder.timeout(timeout);
        }

        let start_time = std::time::Instant::now();

        match request_builder.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let headers: HashMap<String, String> = response
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect();

                let body = response.text().await.unwrap_or_default();
                let duration = start_time.elapsed();

                Ok(WebhookDeliveryResult {
                    success: status >= 200 && status < 300,
                    status_code: status,
                    response_body: body,
                    response_headers: headers,
                    duration_ms: duration.as_millis() as u64,
                    error: None,
                })
            },
            Err(e) => {
                let duration = start_time.elapsed();
                Ok(WebhookDeliveryResult {
                    success: false,
                    status_code: 0,
                    response_body: String::new(),
                    response_headers: HashMap::new(),
                    duration_ms: duration.as_millis() as u64,
                    error: Some(e.to_string()),
                })
            },
        }
    }

    pub async fn verify_signature(
        &self,
        endpoint_id: &str,
        payload: &Value,
        signature: &str,
    ) -> IntegrationResult<bool> {
        let endpoint = {
            let endpoints = self.endpoints.read().await;
            endpoints.get(endpoint_id).cloned().ok_or_else(|| {
                IntegrationError::Webhook(format!("Endpoint not found: {endpoint_id}"))
            })?
        };

        self.verify_signature_internal(&endpoint, payload, signature)
    }

    fn verify_signature_internal(
        &self,
        endpoint: &WebhookEndpoint,
        payload: &Value,
        signature: &str,
    ) -> IntegrationResult<bool> {
        let secret = endpoint.secret.as_ref().ok_or_else(|| {
            IntegrationError::Webhook("No secret configured for endpoint".to_string())
        })?;

        let expected_signature = self.generate_signature(secret, payload)?;

        Ok(self.secure_compare(&expected_signature, signature))
    }

    fn generate_signature(&self, secret: &str, payload: &Value) -> IntegrationResult<String> {
        let payload_bytes = serde_json::to_vec(payload)?;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| IntegrationError::Webhook(format!("Invalid secret: {e}")))?;

        mac.update(&payload_bytes);
        let result = mac.finalize();
        let hex_result = hex::encode(result.into_bytes());

        Ok(format!("sha256={hex_result}"))
    }

    fn secure_compare(&self, a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let mut result = 0u8;
        for (byte_a, byte_b) in a.bytes().zip(b.bytes()) {
            result |= byte_a ^ byte_b;
        }

        result == 0
    }

    pub async fn get_endpoint_stats(&self, endpoint_id: &str) -> IntegrationResult<WebhookStats> {
        let endpoint = {
            let endpoints = self.endpoints.read().await;
            endpoints.get(endpoint_id).cloned().ok_or_else(|| {
                IntegrationError::Webhook(format!("Endpoint not found: {endpoint_id}"))
            })?
        };

        Ok(WebhookStats {
            endpoint_id: endpoint.id,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            last_request_at: None,
            average_response_time_ms: 0,
        })
    }

    pub async fn test_endpoint(&self, endpoint_id: &str) -> IntegrationResult<WebhookTestResult> {
        let endpoint = {
            let endpoints = self.endpoints.read().await;
            endpoints.get(endpoint_id).cloned().ok_or_else(|| {
                IntegrationError::Webhook(format!("Endpoint not found: {endpoint_id}"))
            })?
        };

        let test_payload = serde_json::json!({
            "test": true,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "endpoint_id": endpoint_id
        });

        let config = WebhookConfig {
            secret: endpoint.secret.clone(),
            headers: endpoint.headers.clone(),
            timeout: Some(std::time::Duration::from_secs(10)),
        };

        let result = self
            .send_webhook(&endpoint.url, test_payload, Some(config))
            .await?;

        Ok(WebhookTestResult {
            endpoint_id: endpoint.id,
            success: result.success,
            status_code: result.status_code,
            response_time_ms: result.duration_ms,
            error: result.error,
        })
    }
}

impl Default for WebhookService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct WebhookConfig {
    pub secret: Option<String>,
    pub headers: HashMap<String, String>,
    pub timeout: Option<std::time::Duration>,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            secret: None,
            headers: HashMap::new(),
            timeout: Some(std::time::Duration::from_secs(30)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WebhookDeliveryResult {
    pub success: bool,
    pub status_code: u16,
    pub response_body: String,
    pub response_headers: HashMap<String, String>,
    pub duration_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WebhookStats {
    pub endpoint_id: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub last_request_at: Option<chrono::DateTime<chrono::Utc>>,
    pub average_response_time_ms: u64,
}

#[derive(Debug, Clone)]
pub struct WebhookTestResult {
    pub endpoint_id: String,
    pub success: bool,
    pub status_code: u16,
    pub response_time_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_factor: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_factor: 2.0,
        }
    }
}
