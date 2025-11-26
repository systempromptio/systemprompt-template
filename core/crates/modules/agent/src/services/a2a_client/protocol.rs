use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicI64, Ordering};

use super::error::{ClientError, ClientResult};
use crate::models::a2a::jsonrpc::{JsonRpcResponse, Request, RequestId};

const PROTOCOL_VERSION: &str = "2.0";

#[derive(Debug)]
pub struct ProtocolHandler {
    request_counter: AtomicI64,
}

impl ProtocolHandler {
    pub fn new() -> Self {
        Self {
            request_counter: AtomicI64::new(1),
        }
    }

    pub fn create_request<T: Serialize>(
        &self,
        method: &str,
        params: T,
    ) -> ClientResult<Request<T>> {
        let id = self.next_request_id();
        Ok(Request {
            jsonrpc: PROTOCOL_VERSION.to_string(),
            id,
            method: method.to_string(),
            params,
        })
    }

    pub fn parse_response<T>(&self, response_json: Value) -> ClientResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response: JsonRpcResponse<T> = serde_json::from_value(response_json)?;

        if let Some(error) = response.error {
            return Err(ClientError::agent(error.code, error.message));
        }

        response
            .result
            .ok_or_else(|| ClientError::protocol("Response missing both result and error"))
    }

    fn next_request_id(&self) -> RequestId {
        let id = self.request_counter.fetch_add(1, Ordering::SeqCst);
        RequestId::Number(id)
    }
}

impl Default for ProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageSendRequest {
    pub message: crate::models::a2a::message::Message,
    pub configuration: Option<MessageConfiguration>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MessageConfiguration {
    pub blocking: Option<bool>,
    pub history_length: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskQueryRequest {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelTaskRequest {
    pub id: String,
}
