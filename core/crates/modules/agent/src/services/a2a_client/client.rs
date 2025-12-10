use std::sync::Arc;
use std::time::Duration;

use crate::models::a2a::agent::AgentCard;
use crate::models::a2a::message::Message;
use crate::models::a2a::task::Task;

use super::error::{ClientError, ClientResult};
use super::protocol::{
    CancelTaskRequest, MessageConfiguration, MessageSendRequest, ProtocolHandler, TaskQueryRequest,
};
use super::streaming::SseStream;
use super::transport::{HttpTransport, Transport};
use systemprompt_core_logging::LogService;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub auth_token: Option<String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            timeout: Duration::from_secs(30),
            auth_token: None,
        }
    }
}

pub struct A2aClient {
    protocol: ProtocolHandler,
    transport: Arc<dyn Transport>,
    config: ClientConfig,
    log_service: Option<LogService>,
}

impl std::fmt::Debug for A2aClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("A2aClient")
            .field("protocol", &self.protocol)
            .field("transport", &"<transport>")
            .field("config", &self.config)
            .field("log_service", &self.log_service.is_some())
            .finish()
    }
}

impl A2aClient {
    pub fn new(config: ClientConfig) -> ClientResult<Self> {
        let mut transport = HttpTransport::new(&config.base_url)?.with_timeout(config.timeout)?;

        if let Some(token) = &config.auth_token {
            transport = transport.with_auth_token(token)?;
        }

        let transport: Arc<dyn Transport> = Arc::new(transport);

        Ok(Self {
            protocol: ProtocolHandler::new(),
            transport,
            config,
            log_service: None,
        })
    }

    pub fn with_logger(mut self, log_service: LogService) -> Self {
        self.log_service = Some(log_service);
        self
    }

    fn create_http_client(&self) -> ClientResult<reqwest::Client> {
        Ok(reqwest::Client::builder()
            .timeout(self.config.timeout)
            .build()?)
    }

    pub async fn send_message(&self, message: Message) -> ClientResult<Task> {
        let request_params = MessageSendRequest {
            message,
            configuration: Some(MessageConfiguration {
                blocking: Some(true),
                history_length: None,
            }),
        };

        let request = self
            .protocol
            .create_request("message/send", request_params)?;
        let response = self
            .transport
            .send_request("/", serde_json::to_value(&request)?)
            .await?;

        if !response.status().is_success() {
            return Err(ClientError::invalid_response(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let response_json = response.json().await?;
        self.protocol.parse_response(response_json)
    }

    pub async fn send_streaming_message(&self, message: Message) -> ClientResult<SseStream> {
        let request_params = MessageSendRequest {
            message,
            configuration: Some(MessageConfiguration {
                blocking: Some(false),
                history_length: None,
            }),
        };

        let request = self
            .protocol
            .create_request("message/send", request_params)?;
        let url = format!("{}/stream", self.config.base_url);

        let client = self.create_http_client()?;

        SseStream::new_with_auth(
            client,
            url,
            serde_json::to_value(&request)?,
            self.config.auth_token.clone(),
            self.log_service.clone(),
        )
        .await
    }

    pub async fn get_task(&self, task_id: &str) -> ClientResult<Task> {
        let request_params = TaskQueryRequest {
            id: task_id.to_string(),
        };

        let request = self.protocol.create_request("tasks/get", request_params)?;
        let response = self
            .transport
            .send_request("/", serde_json::to_value(&request)?)
            .await?;

        if !response.status().is_success() {
            return Err(ClientError::invalid_response(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let response_json = response.json().await?;
        self.protocol.parse_response(response_json)
    }

    pub async fn cancel_task(&self, task_id: &str) -> ClientResult<Task> {
        let request_params = CancelTaskRequest {
            id: task_id.to_string(),
        };

        let request = self
            .protocol
            .create_request("tasks/cancel", request_params)?;
        let response = self
            .transport
            .send_request("/", serde_json::to_value(&request)?)
            .await?;

        if !response.status().is_success() {
            return Err(ClientError::invalid_response(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let response_json = response.json().await?;
        self.protocol.parse_response(response_json)
    }

    pub async fn get_authenticated_extended_card(&self) -> ClientResult<AgentCard> {
        let request = self
            .protocol
            .create_request("agent/getAuthenticatedExtendedCard", ())?;
        let response = self
            .transport
            .send_request("/", serde_json::to_value(&request)?)
            .await?;

        if !response.status().is_success() {
            return Err(ClientError::invalid_response(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let response_json = response.json().await?;
        self.protocol.parse_response(response_json)
    }

    pub async fn fetch_agent_card(&self) -> ClientResult<AgentCard> {
        let url = format!("{}/.well-known/agent-card.json", self.config.base_url);

        let client = self.create_http_client()?;

        let mut request_builder = client.get(&url);

        if let Some(token) = &self.config.auth_token {
            request_builder = request_builder.header("Authorization", format!("Bearer {token}"));
        }

        let response = request_builder.send().await?;

        if !response.status().is_success() {
            return Err(ClientError::invalid_response(format!(
                "Failed to fetch agent card: HTTP {}",
                response.status()
            )));
        }

        let agent_card: AgentCard = response.json().await?;
        Ok(agent_card)
    }
}
