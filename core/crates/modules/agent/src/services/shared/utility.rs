use crate::services::shared::error::{AgentServiceError, Result};

pub fn parse_service_endpoint(endpoint: &str) -> Result<ServiceEndpoint> {
    let url = url::Url::parse(endpoint).map_err(|e| {
        AgentServiceError::Configuration(
            "ServiceEndpoint".to_string(),
            format!("Invalid endpoint URL: {e}"),
        )
    })?;

    let host = url
        .host_str()
        .ok_or_else(|| {
            AgentServiceError::Configuration(
                "ServiceEndpoint".to_string(),
                "Endpoint missing host".to_string(),
            )
        })?
        .to_string();

    let port = url.port().unwrap_or_else(|| match url.scheme() {
        "https" => 443,
        "http" => 80,
        _ => 80,
    });

    Ok(ServiceEndpoint {
        scheme: url.scheme().to_string(),
        host,
        port,
        path: url.path().to_string(),
    })
}

#[derive(Debug, Clone)]
pub struct ServiceEndpoint {
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub path: String,
}

impl ServiceEndpoint {
    pub fn to_url(&self) -> String {
        format!("{}://{}:{}{}", self.scheme, self.host, self.port, self.path)
    }
}

pub fn generate_unique_service_id(service_name: &str) -> String {
    format!("{}-{}", service_name, uuid::Uuid::new_v4())
}

pub fn validate_service_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(AgentServiceError::Validation(
            "service_name".to_string(),
            "cannot be empty".to_string(),
        ));
    }

    if name.len() > 128 {
        return Err(AgentServiceError::Validation(
            "service_name".to_string(),
            "exceeds maximum length of 128 characters".to_string(),
        ));
    }

    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AgentServiceError::Validation(
            "service_name".to_string(),
            "contains invalid characters (only alphanumeric, -, _ allowed)".to_string(),
        ));
    }

    Ok(())
}
