//! A2A Authentication domain types
//!
//! Authentication and security scheme definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Security Scheme as specified in A2A spec section 5.5.3
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SecurityScheme {
    #[serde(rename = "apiKey")]
    ApiKey {
        name: String,
        #[serde(rename = "in")]
        location: ApiKeyLocation,
        description: Option<String>,
    },
    #[serde(rename = "http")]
    Http {
        scheme: String,
        bearer_format: Option<String>,
        description: Option<String>,
    },
    #[serde(rename = "oauth2")]
    OAuth2 {
        flows: OAuth2Flows,
        description: Option<String>,
    },
    #[serde(rename = "openIdConnect")]
    OpenIdConnect {
        open_id_connect_url: String,
        description: Option<String>,
    },
    #[serde(rename = "mutualTLS")]
    MutualTls { description: Option<String> },
}

/// API key location options
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ApiKeyLocation {
    Query,
    Header,
    Cookie,
}

impl std::fmt::Display for ApiKeyLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Query => write!(f, "query"),
            Self::Header => write!(f, "header"),
            Self::Cookie => write!(f, "cookie"),
        }
    }
}

impl std::str::FromStr for ApiKeyLocation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "query" => Ok(Self::Query),
            "header" => Ok(Self::Header),
            "cookie" => Ok(Self::Cookie),
            _ => Err(format!("Invalid API key location: {s}")),
        }
    }
}

/// OAuth2 flows configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2Flows {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implicit: Option<OAuth2Flow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<OAuth2Flow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_credentials: Option<OAuth2Flow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_code: Option<OAuth2Flow>,
}

/// OAuth2 flow configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2Flow {
    pub authorization_url: Option<String>,
    pub token_url: Option<String>,
    pub refresh_url: Option<String>,
    pub scopes: HashMap<String, String>,
}

/// Agent authentication configuration (flexible JSON value)
pub type AgentAuthentication = serde_json::Value;
