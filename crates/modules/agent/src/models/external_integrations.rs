use serde::{Deserialize, Serialize};
use systemprompt_models::ai::tools::McpTool;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IntegrationError {
    #[error("OAuth error: {0}")]
    OAuth(String),
    #[error("MCP error: {0}")]
    Mcp(String),
    #[error("Webhook error: {0}")]
    Webhook(String),
    #[error("Repository error: {0}")]
    Repository(#[from] crate::repository::RepositoryError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("OAuth2 error: {0}")]
    OAuth2(String),
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Server not found: {0}")]
    ServerNotFound(String),
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    #[error("Invalid signature")]
    InvalidSignature,
}

pub type IntegrationResult<T> = Result<T, IntegrationError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scopes: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationRequest {
    pub authorization_url: String,
    pub state: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationResult {
    pub agent_id: String,
    pub provider: String,
    pub success: bool,
    pub tokens: Option<TokenInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackParams {
    pub code: Option<String>,
    pub state: String,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredMcpServer {
    pub id: String,
    pub name: String,
    pub url: String,
    pub status: String,
    pub capabilities: Vec<String>,
    pub tools: Vec<McpTool>,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub tool_name: String,
    pub server_id: String,
    pub result: serde_json::Value,
    pub execution_time_ms: u64,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    pub id: String,
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
    pub headers: std::collections::HashMap<String, String>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRequest {
    pub headers: std::collections::HashMap<String, String>,
    pub body: serde_json::Value,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResponse {
    pub status: u16,
    pub body: Option<serde_json::Value>,
}
