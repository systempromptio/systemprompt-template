//! JSON-RPC 2.0 types for A2A protocol
//!
//! Standard JSON-RPC 2.0 types used by the A2A protocol.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC version constant
pub const JSON_RPC_VERSION_2_0: &str = "2.0";

/// JSON-RPC request ID (can be string or number)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
}

/// Alias for backward compatibility
pub type NumberOrString = RequestId;

/// JSON-RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request<T> {
    pub jsonrpc: String,
    pub method: String,
    pub params: T,
    pub id: RequestId,
}

/// JSON-RPC response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: RequestId,
}

/// JSON-RPC error
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(code: i32, message: impl Into<String>, data: Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }

    // Standard JSON-RPC error codes
    pub fn parse_error() -> Self {
        Self::new(-32700, "Parse error")
    }

    pub fn invalid_request() -> Self {
        Self::new(-32600, "Invalid Request")
    }

    pub fn method_not_found() -> Self {
        Self::new(-32601, "Method not found")
    }

    pub fn invalid_params() -> Self {
        Self::new(-32602, "Invalid params")
    }

    pub fn internal_error() -> Self {
        Self::new(-32603, "Internal error")
    }
}
