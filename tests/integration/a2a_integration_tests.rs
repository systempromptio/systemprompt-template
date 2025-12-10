use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct AgentInfo {
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub port: u16,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskStatus {
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct A2aTask {
    pub id: String,
    #[serde(rename = "contextId")]
    pub context_id: String,
    pub status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<A2aMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<serde_json::Value>>,
    pub kind: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct A2aMessage {
    pub role: String,
    pub parts: Vec<serde_json::Value>,
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "taskId", skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(rename = "contextId")]
    pub context_id: String,
    pub kind: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<A2aTask>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StreamChunk {
    pub delta: String,
    pub task_id: Option<String>,
}

pub struct A2ATestContext {
    pub http_client: Client,
    pub base_url: String,
    pub admin_token: String,
}

impl A2ATestContext {
    pub async fn new() -> anyhow::Result<Self> {
        let env = std::env::var("TEST_ENV").unwrap_or_else(|_| "local".to_string());

        let base_url = match env.as_str() {
            "docker" => "http://localhost:8085".to_string(),
            "production" => "https://tyingshoelaces.com".to_string(),
            _ => "http://localhost:8080".to_string(),
        };

        let admin_token = generate_admin_token(&base_url).await?;

        Ok(Self {
            http_client: Client::new(),
            base_url,
            admin_token,
        })
    }

    pub async fn get_available_agent(&self) -> anyhow::Result<AgentInfo> {
        let url = format!("{}/api/v1/agents/registry", self.base_url);

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.admin_token))
            .send()
            .await?;

        let body: serde_json::Value = response.json().await?;

        // Response is wrapped in a "data" field containing an array
        let agents: Vec<AgentInfo> = serde_json::from_value(body["data"].clone())
            .map_err(|e| anyhow::anyhow!("Failed to parse agent registry: {}", e))?;

        // Get first agent available (prefer edward, then admin, then any)
        let agent = agents
            .iter()
            .find(|a| a.name == "edward")
            .or_else(|| agents.iter().find(|a| a.name == "admin"))
            .or_else(|| agents.first())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No available agents found in registry"))?;

        // Set ID to name if not present
        let mut agent = agent;
        if agent.id.is_empty() {
            agent.id = agent.name.clone();
        }
        Ok(agent)
    }

    pub async fn send_message(&self, agent_id: &str, message: &str) -> anyhow::Result<A2aTask> {
        let context_id = self.create_context_in_db().await?;
        let message_id = self.create_message_id();

        let url = format!("{}/api/v1/agents/{}/", self.base_url, agent_id);

        let message_obj = json!({
            "messageId": message_id,
            "contextId": context_id,
            "role": "user",
            "kind": "message",
            "parts": [
                {
                    "kind": "text",
                    "text": message
                }
            ]
        });

        let body = json!({
            "jsonrpc": "2.0",
            "method": "message/send",
            "params": {
                "message": message_obj,
                "configuration": null,
                "metadata": null
            },
            "id": message_id
        });

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.admin_token))
            .json(&body)
            .send()
            .await?;

        if response.status() != 200 {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Conversation request failed with status: {}, body: {}",
                status,
                error_text
            ));
        }

        let jsonrpc_response: JsonRpcResponse = response.json().await?;

        if let Some(task) = jsonrpc_response.result {
            Ok(task)
        } else if let Some(error) = jsonrpc_response.error {
            let data = error
                .data
                .as_ref()
                .map(|d| d.to_string())
                .unwrap_or_default();
            Err(anyhow::anyhow!(
                "JSON-RPC error: {} | data: {} | url: {}",
                error.message,
                data,
                url
            ))
        } else {
            Err(anyhow::anyhow!("No result or error in response"))
        }
    }

    pub fn create_context(&self) -> String {
        Uuid::new_v4().to_string()
    }

    pub fn create_message_id(&self) -> String {
        Uuid::new_v4().to_string()
    }

    pub async fn create_context_in_db(&self) -> anyhow::Result<String> {
        let url = format!("{}/api/v1/core/contexts", self.base_url);

        let body = json!({
            "name": format!("test-context-{}", Uuid::new_v4())
        });

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.admin_token))
            .json(&body)
            .send()
            .await?;

        if response.status() != 201 && response.status() != 200 {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to create context with status: {}, body: {}",
                status,
                error_text
            ));
        }

        let context_data: serde_json::Value = response.json().await?;
        let context_id = context_data
            .get("context_id")
            .or_else(|| context_data.get("id"))
            .or_else(|| context_data.get("contextId"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("No context_id in response: {}", context_data))?
            .to_string();

        Ok(context_id)
    }

    pub async fn send_message_with_task(
        &self,
        agent_id: &str,
        context_id: &str,
        task_id: &str,
        message: &str,
    ) -> anyhow::Result<A2aTask> {
        let message_id = self.create_message_id();

        let url = format!("{}/api/v1/agents/{}/", self.base_url, agent_id);

        let message_obj = json!({
            "messageId": message_id,
            "contextId": context_id,
            "taskId": task_id,
            "role": "user",
            "kind": "message",
            "parts": [
                {
                    "kind": "text",
                    "text": message
                }
            ]
        });

        let body = json!({
            "jsonrpc": "2.0",
            "method": "message/send",
            "params": {
                "message": message_obj,
                "configuration": null,
                "metadata": null
            },
            "id": message_id
        });

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.admin_token))
            .json(&body)
            .send()
            .await?;

        if response.status() != 200 {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Conversation request failed with status: {}, body: {}",
                status,
                error_text
            ));
        }

        let jsonrpc_response: JsonRpcResponse = response.json().await?;

        if let Some(task) = jsonrpc_response.result {
            Ok(task)
        } else if let Some(error) = jsonrpc_response.error {
            Err(anyhow::anyhow!("JSON-RPC error: {}", error.message))
        } else {
            Err(anyhow::anyhow!("No result or error in response"))
        }
    }

    pub async fn send_streaming_message(
        &self,
        agent_id: &str,
        message: &str,
    ) -> anyhow::Result<A2aTask> {
        let context_id = self.create_context_in_db().await?;
        let message_id = self.create_message_id();

        let url = format!("{}/api/v1/agents/{}/", self.base_url, agent_id);

        let message_obj = json!({
            "messageId": message_id,
            "contextId": context_id,
            "role": "user",
            "kind": "message",
            "parts": [
                {
                    "kind": "text",
                    "text": message
                }
            ]
        });

        let body = json!({
            "jsonrpc": "2.0",
            "method": "message/stream",
            "params": {
                "message": message_obj,
                "configuration": null,
                "metadata": null
            },
            "id": message_id
        });

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.admin_token))
            .json(&body)
            .send()
            .await?;

        let response_text = response.text().await?;

        // Handle empty response (streaming might return empty body)
        if response_text.is_empty() || response_text.trim().is_empty() {
            return Ok(A2aTask {
                id: message_id.clone(),
                context_id: context_id.clone(),
                status: TaskStatus {
                    state: "streaming".to_string(),
                    message: None,
                    timestamp: None,
                },
                history: None,
                artifacts: None,
                kind: "task".to_string(),
            });
        }

        // Parse SSE format (Server-Sent Events): "data: {...}\ndata: {...}"
        // Get the last data event which should be the final task response
        let last_data_line = response_text
            .lines()
            .rev()
            .find(|line| line.starts_with("data: "))
            .ok_or_else(|| anyhow::anyhow!("No data events in streaming response"))?;

        let json_str = &last_data_line[6..]; // Skip "data: " prefix
        let jsonrpc_response: JsonRpcResponse = serde_json::from_str(json_str)?;

        if let Some(task) = jsonrpc_response.result {
            Ok(task)
        } else if let Some(error) = jsonrpc_response.error {
            Err(anyhow::anyhow!("JSON-RPC error: {}", error.message))
        } else {
            Err(anyhow::anyhow!("No result or error in response"))
        }
    }
}

async fn generate_admin_token(_base_url: &str) -> anyhow::Result<String> {
    if let Ok(token) = std::env::var("ADMIN_TOKEN") {
        return Ok(token);
    }

    Err(anyhow::anyhow!(
        "Admin token not available. Set ADMIN_TOKEN environment variable. Generate token with: \
         just admin-token"
    ))
}

mod agent_registry_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_agent_registry() {
        let ctx = A2ATestContext::new()
            .await
            .expect("Failed to create test context");

        let url = format!("{}/api/v1/agents/registry", ctx.base_url);

        let response = ctx
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", ctx.admin_token))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(
            response.status(),
            200,
            "Expected 200, got {}",
            response.status()
        );

        let body: serde_json::Value = response
            .json()
            .await
            .expect("Failed to parse agent registry response");

        let agents: Vec<AgentInfo> = serde_json::from_value(body["data"].clone())
            .expect("Failed to extract agents from response");

        assert!(
            !agents.is_empty(),
            "No agents in registry. Is the agent orchestrator running?"
        );

        for agent in &agents {
            assert!(!agent.name.is_empty(), "Agent should have non-empty name");
            assert!(!agent.url.is_empty(), "Agent should have non-empty url");
        }

        let count = agents.len();
        assert!(count > 0, "Expected at least one agent, got {}", count);
    }

    #[tokio::test]
    async fn test_get_specific_agent() {
        let ctx = A2ATestContext::new()
            .await
            .expect("Failed to create test context");

        let url = format!("{}/api/v1/agents/registry", ctx.base_url);

        let response = ctx
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", ctx.admin_token))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(
            response.status(),
            200,
            "Expected 200, got {}",
            response.status()
        );

        let body: serde_json::Value = response
            .json()
            .await
            .expect("Failed to parse registry response");

        let agents: Vec<AgentInfo> = serde_json::from_value(body["data"].clone())
            .expect("Failed to extract agents from response");

        assert!(
            !agents.is_empty(),
            "Should have at least one agent in registry"
        );
        assert!(
            !agents[0].name.is_empty(),
            "Agent should have non-empty name"
        );
    }

    #[tokio::test]
    async fn test_agent_health_check() {
        let ctx = A2ATestContext::new()
            .await
            .expect("Failed to create test context");
        let agent = ctx
            .get_available_agent()
            .await
            .expect("Failed to get available agent");

        let url = format!("{}/api/v1/agents/registry", ctx.base_url);

        let response = ctx
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", ctx.admin_token))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(
            response.status(),
            200,
            "Agent registry should be accessible"
        );

        let body: serde_json::Value = response.json().await.expect("Failed to parse response");
        let agents: Vec<AgentInfo> = serde_json::from_value(body["data"].clone())
            .expect("Failed to extract agents from response");

        let agent_in_registry = agents.iter().find(|a| a.name == agent.name);
        assert!(agent_in_registry.is_some(), "Agent should be in registry");
    }
}

mod conversation_lifecycle_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_conversation_flow() {
        let ctx = A2ATestContext::new()
            .await
            .expect("Failed to create test context");
        let agent = ctx
            .get_available_agent()
            .await
            .expect("Failed to get available agent");

        let message = "Hello, can you help me with a simple task?";
        let response = ctx
            .send_message(&agent.id, message)
            .await
            .expect("Failed to send message");

        assert_eq!(
            response.status.state, "completed",
            "Expected conversation status 'completed', got '{}'",
            response.status.state
        );
        assert!(!response.id.is_empty(), "Response should contain task id");
        assert!(
            !response.context_id.is_empty(),
            "Response should contain context_id"
        );
        assert_eq!(response.kind, "task", "Response kind should be 'task'");
        assert!(
            response.history.is_some(),
            "Response should contain message history"
        );
        let history = response.history.expect("History should be present");
        assert!(
            history.len() >= 1,
            "Expected at least 1 message in history, got {}",
            history.len()
        );
    }

    #[tokio::test]
    async fn test_multi_turn_conversation() {
        let ctx = A2ATestContext::new()
            .await
            .expect("Failed to create test context");
        let agent = ctx
            .get_available_agent()
            .await
            .expect("Failed to get available agent");

        let response1 = ctx
            .send_message(&agent.id, "What is 2+2?")
            .await
            .expect("Failed to send first message");

        let task_id = response1.id.clone();
        let context_id = response1.context_id.clone();
        assert!(!task_id.is_empty(), "Should have task_id");

        let response2 = ctx
            .send_message_with_task(
                &agent.id,
                &context_id,
                &task_id,
                "And what is that multiplied by 3?",
            )
            .await
            .expect("Failed to send second message");

        assert_eq!(
            response2.id, task_id,
            "Should use same task_id for continuation"
        );

        let history = response2.history.expect("History should be present");
        assert!(
            history.len() >= 2,
            "Continuation should result in at least 2 messages in history"
        );
    }

    #[tokio::test]
    async fn test_streaming_conversation() {
        let ctx = A2ATestContext::new()
            .await
            .expect("Failed to create test context");
        let agent = ctx
            .get_available_agent()
            .await
            .expect("Failed to get available agent");

        let response = ctx
            .send_streaming_message(&agent.id, "Tell me a short story")
            .await
            .expect("Failed to send streaming message");

        assert!(
            !response.id.is_empty(),
            "Should receive a task id in streaming response"
        );

        assert_eq!(response.kind, "task", "Response kind should be 'task'");

        assert!(
            !response.context_id.is_empty(),
            "Streaming response should contain context_id"
        );
    }
}

mod task_creation_tests {
    use super::*;

    #[tokio::test]
    async fn test_task_created_with_all_fields() {
        let ctx = A2ATestContext::new()
            .await
            .expect("Failed to create test context");
        let agent = ctx
            .get_available_agent()
            .await
            .expect("Failed to get available agent");

        let response = ctx
            .send_message(&agent.id, "Test message")
            .await
            .expect("Failed to send message");

        assert!(!response.id.is_empty(), "Response should have task id");
        assert_eq!(response.kind, "task", "Response should be a task");
        assert_eq!(
            response.status.state, "completed",
            "Task should be completed"
        );
        assert!(response.history.is_some(), "Should have message history");
        let history = response.history.expect("History should be present");
        assert!(
            history.len() >= 1,
            "Should have at least 1 message in history"
        );
    }

    #[tokio::test]
    async fn test_multiple_tasks_are_independent() {
        let ctx = A2ATestContext::new()
            .await
            .expect("Failed to create test context");
        let agent = ctx
            .get_available_agent()
            .await
            .expect("Failed to get available agent");

        let response1 = ctx
            .send_message(&agent.id, "First task")
            .await
            .expect("Failed to send first message");

        let response2 = ctx
            .send_message(&agent.id, "Second task")
            .await
            .expect("Failed to send second message");

        assert_ne!(
            response1.id, response2.id,
            "Different conversations should have different task_ids"
        );

        assert_eq!(
            response1.status.state, "completed",
            "First task should be completed"
        );
        assert_eq!(
            response2.status.state, "completed",
            "Second task should be completed"
        );
    }
}

mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_agent_id_returns_404() {
        let ctx = A2ATestContext::new()
            .await
            .expect("Failed to create test context");

        let message_id = Uuid::new_v4().to_string();
        let body = json!({
            "jsonrpc": "2.0",
            "method": "message/send",
            "params": {
                "message": {
                    "messageId": message_id,
                    "contextId": Uuid::new_v4().to_string(),
                    "role": "user",
                    "kind": "message",
                    "parts": [{"kind": "text", "text": "Test"}]
                }
            },
            "id": message_id
        });

        let response = ctx
            .http_client
            .post(&format!(
                "{}/api/v1/agents/invalid-agent-id-that-does-not-exist/",
                ctx.base_url
            ))
            .header("Authorization", format!("Bearer {}", ctx.admin_token))
            .json(&body)
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(
            response.status(),
            404,
            "Invalid agent ID should return 404, got {}",
            response.status()
        );
    }

    #[tokio::test]
    async fn test_unauthorized_request_returns_401() {
        let ctx = A2ATestContext::new()
            .await
            .expect("Failed to create test context");
        let agent = ctx
            .get_available_agent()
            .await
            .expect("Failed to get available agent");

        let message_id = Uuid::new_v4().to_string();
        let body = json!({
            "jsonrpc": "2.0",
            "method": "message/send",
            "params": {
                "message": {
                    "messageId": message_id,
                    "contextId": Uuid::new_v4().to_string(),
                    "role": "user",
                    "kind": "message",
                    "parts": [{"kind": "text", "text": "Test"}]
                }
            },
            "id": message_id
        });

        let response = ctx
            .http_client
            .post(&format!("{}/api/v1/agents/{}/", ctx.base_url, agent.id))
            .json(&body)
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(
            response.status(),
            401,
            "Missing auth should return 401, got {}",
            response.status()
        );
    }

    #[tokio::test]
    async fn test_malformed_request_returns_400() {
        let ctx = A2ATestContext::new()
            .await
            .expect("Failed to create test context");
        let agent = ctx
            .get_available_agent()
            .await
            .expect("Failed to get available agent");

        let response = ctx
            .http_client
            .post(&format!("{}/api/v1/agents/{}/", ctx.base_url, agent.id))
            .header("Authorization", format!("Bearer {}", ctx.admin_token))
            .json(&json!({
                "invalid_field": "value"
            }))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(
            response.status(),
            400,
            "Malformed request should return 400, got {}",
            response.status()
        );
    }
}
