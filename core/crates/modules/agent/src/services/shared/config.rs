use crate::services::shared::error::{AgentServiceError, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ServiceConfiguration {
    pub enabled: bool,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub retry_delay_milliseconds: u64,
    pub max_connections: usize,
}

impl ServiceConfiguration {
    pub const fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }

    pub const fn retry_delay(&self) -> Duration {
        Duration::from_millis(self.retry_delay_milliseconds)
    }

    pub fn validate(&self) -> Result<()> {
        if self.retry_attempts == 0 {
            return Err(AgentServiceError::Configuration(
                "ServiceConfiguration".to_string(),
                "retry_attempts must be at least 1".to_string(),
            ));
        }
        if self.max_connections == 0 {
            return Err(AgentServiceError::Configuration(
                "ServiceConfiguration".to_string(),
                "max_connections must be at least 1".to_string(),
            ));
        }
        Ok(())
    }
}

impl Default for ServiceConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_seconds: 30,
            retry_attempts: 3,
            retry_delay_milliseconds: 500,
            max_connections: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfiguration {
    pub agent_id: String,
    pub name: String,
    pub port: u16,
    pub host: String,
    pub ssl_enabled: bool,
    pub auth_required: bool,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RuntimeConfigurationBuilder {
    agent_id: String,
    name: String,
    port: u16,
    host: String,
    ssl_enabled: bool,
    auth_required: bool,
    system_prompt: Option<String>,
}

impl RuntimeConfigurationBuilder {
    pub fn new(agent_id: String, name: String) -> Self {
        Self {
            agent_id,
            name,
            port: 8080,
            host: "localhost".to_string(),
            ssl_enabled: false,
            auth_required: false,
            system_prompt: None,
        }
    }

    pub const fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn host(mut self, host: String) -> Self {
        self.host = host;
        self
    }

    pub const fn enable_ssl(mut self) -> Self {
        self.ssl_enabled = true;
        self
    }

    pub const fn require_auth(mut self) -> Self {
        self.auth_required = true;
        self
    }

    pub fn system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = Some(prompt);
        self
    }

    pub fn build(self) -> RuntimeConfiguration {
        RuntimeConfiguration {
            agent_id: self.agent_id,
            name: self.name,
            port: self.port,
            host: self.host,
            ssl_enabled: self.ssl_enabled,
            auth_required: self.auth_required,
            system_prompt: self.system_prompt,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfiguration {
    pub url: String,
    pub timeout_seconds: u64,
    pub keepalive_enabled: bool,
    pub pool_size: usize,
}

impl ConnectionConfiguration {
    pub const fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }
}

pub trait ConfigValidation {
    fn validate(&self) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentServiceConfig {
    pub agent_id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub endpoint: String,
    pub port: u16,
    pub is_active: bool,
}

impl ConfigValidation for AgentServiceConfig {
    fn validate(&self) -> Result<()> {
        if self.agent_id.is_empty() {
            return Err(AgentServiceError::Validation(
                "agent_id".to_string(),
                "cannot be empty".to_string(),
            ));
        }
        if self.port == 0 {
            return Err(AgentServiceError::Validation(
                "port".to_string(),
                "must be greater than 0".to_string(),
            ));
        }
        if self.name.is_empty() {
            return Err(AgentServiceError::Validation(
                "name".to_string(),
                "cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

impl Default for AgentServiceConfig {
    fn default() -> Self {
        Self {
            agent_id: uuid::Uuid::new_v4().to_string(),
            name: "Default Agent".to_string(),
            description: "Default agent instance".to_string(),
            version: "0.1.0".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            port: 8080,
            is_active: true,
        }
    }
}
