//! A2A Transport protocol types
//!
//! Transport protocol definitions and utilities.

use serde::{Deserialize, Serialize};

/// Supported A2A transport protocols as specified in A2A spec section 5.5.5
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TransportProtocol {
    #[serde(rename = "JSONRPC")]
    JsonRpc,
    #[serde(rename = "GRPC")]
    Grpc,
    #[serde(rename = "HTTP+JSON")]
    HttpJson,
}

impl Default for TransportProtocol {
    fn default() -> Self {
        Self::JsonRpc
    }
}

impl From<TransportProtocol> for String {
    fn from(transport: TransportProtocol) -> String {
        match transport {
            TransportProtocol::JsonRpc => "JSONRPC".to_string(),
            TransportProtocol::Grpc => "GRPC".to_string(),
            TransportProtocol::HttpJson => "HTTP+JSON".to_string(),
        }
    }
}

impl std::str::FromStr for TransportProtocol {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "JSONRPC" => Ok(Self::JsonRpc),
            "GRPC" => Ok(Self::Grpc),
            "HTTP+JSON" => Ok(Self::HttpJson),
            _ => Err(anyhow::anyhow!("Invalid transport protocol: {}", s)),
        }
    }
}

impl ToString for TransportProtocol {
    fn to_string(&self) -> String {
        String::from(self.clone())
    }
}

impl TransportProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransportProtocol::JsonRpc => "JSONRPC",
            TransportProtocol::Grpc => "GRPC",
            TransportProtocol::HttpJson => "HTTP+JSON",
        }
    }
}
