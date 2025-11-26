//! A2A Protocol request/response types
//!
//! Protocol-level types for A2A requests, responses, and JSON-RPC handling.

use super::agent::AgentCard;
use super::auth::AgentAuthentication;
use super::task::{Task, TaskState, TaskStatus};
use serde::{Deserialize, Serialize};

// Import JSON-RPC types
use super::jsonrpc::{JsonRpcResponse, RequestId};

// ===== Request Parameter Types =====

/// Parameters for message send operations
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MessageSendParams {
    pub message: super::message::Message,
    pub configuration: Option<MessageSendConfiguration>,
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
}

/// Configuration for message send operations
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageSendConfiguration {
    pub accepted_output_modes: Option<Vec<String>>,
    pub history_length: Option<u32>,
    pub push_notification_config: Option<PushNotificationConfig>,
    pub blocking: Option<bool>,
}

/// Push notification configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PushNotificationConfig {
    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub endpoint: String, // Legacy field, use url instead
    pub headers: Option<serde_json::Map<String, serde_json::Value>>,
    pub url: String,
    pub token: Option<String>,
    pub authentication: Option<AgentAuthentication>,
}

/// Parameters for task query operations
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TaskQueryParams {
    pub id: String,
    pub history_length: Option<u32>,
}

/// Parameters for task ID operations
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TaskIdParams {
    pub id: String,
}

// ===== A2A Request/Response Enums =====

/// A2A protocol request enum - currently unused, kept for future implementation
// #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
// #[serde(tag = "method", content = "params")]
// pub enum A2aRequest {
//     #[serde(rename = "message/send")]
//     SendMessageRequest(MessageSendParams),
//     #[serde(rename = "tasks/get")]
//     GetTaskRequest(TaskQueryParams),
//     #[serde(rename = "tasks/cancel")]
//     CancelTaskRequest(TaskIdParams),
// }

// Simplified placeholder for now
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct A2aRequest {
    pub method: String,
    pub params: serde_json::Value,
}

/// A2A protocol response enum
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum A2aResponse {
    SendMessage(SendMessageResponse),
    GetTask(GetTaskResponse),
    CancelTask(CancelTaskResponse),
    GetAuthenticatedExtendedCard(GetAuthenticatedExtendedCardResponse),
    SendStreamingMessage(SendStreamingMessageResponse),
}

// Method types removed - not needed with simplified approach

pub type SendMessageResponse = JsonRpcResponse<Task>;
pub type GetTaskResponse = JsonRpcResponse<Task>;
pub type CancelTaskResponse = JsonRpcResponse<Task>;
pub type GetAuthenticatedExtendedCardResponse = JsonRpcResponse<AgentCard>;
pub type SendStreamingMessageResponse = JsonRpcResponse<Task>;

// ===== JSON-RPC Message Types =====

/// A2A JSON-RPC Request wrapper
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct A2aJsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: RequestId,
}

impl A2aJsonRpcRequest {
    /// Parse the method and params into a strongly-typed A2A request
    pub fn parse_request(&self) -> Result<A2aRequestParams, A2aParseError> {
        match self.method.as_str() {
            "message/send" => {
                let params: MessageSendParams = serde_json::from_value(self.params.clone())
                    .map_err(|e| A2aParseError::InvalidParams {
                        method: self.method.clone(),
                        error: e.to_string(),
                    })?;
                Ok(A2aRequestParams::SendMessage(params))
            },
            "tasks/get" => {
                let params: TaskQueryParams =
                    serde_json::from_value(self.params.clone()).map_err(|e| {
                        A2aParseError::InvalidParams {
                            method: self.method.clone(),
                            error: e.to_string(),
                        }
                    })?;
                Ok(A2aRequestParams::GetTask(params))
            },
            "tasks/cancel" => {
                let params: TaskIdParams =
                    serde_json::from_value(self.params.clone()).map_err(|e| {
                        A2aParseError::InvalidParams {
                            method: self.method.clone(),
                            error: e.to_string(),
                        }
                    })?;
                Ok(A2aRequestParams::CancelTask(params))
            },
            "agent/getAuthenticatedExtendedCard" => {
                let params: serde_json::Value = serde_json::from_value(self.params.clone())
                    .map_err(|e| A2aParseError::InvalidParams {
                        method: self.method.clone(),
                        error: e.to_string(),
                    })?;
                Ok(A2aRequestParams::GetAuthenticatedExtendedCard(params))
            },
            "message/stream" => {
                let params: MessageSendParams = serde_json::from_value(self.params.clone())
                    .map_err(|e| A2aParseError::InvalidParams {
                        method: self.method.clone(),
                        error: e.to_string(),
                    })?;
                Ok(A2aRequestParams::SendStreamingMessage(params))
            },
            "tasks/resubscribe" => {
                let params: TaskResubscriptionRequest = serde_json::from_value(self.params.clone())
                    .map_err(|e| A2aParseError::InvalidParams {
                        method: self.method.clone(),
                        error: e.to_string(),
                    })?;
                Ok(A2aRequestParams::TaskResubscription(params))
            },
            "tasks/pushNotificationConfig/set" => {
                let params: SetTaskPushNotificationConfigRequest =
                    serde_json::from_value(self.params.clone()).map_err(|e| {
                        A2aParseError::InvalidParams {
                            method: self.method.clone(),
                            error: e.to_string(),
                        }
                    })?;
                Ok(A2aRequestParams::SetTaskPushNotificationConfig(params))
            },
            "tasks/pushNotificationConfig/get" => {
                let params: GetTaskPushNotificationConfigRequest =
                    serde_json::from_value(self.params.clone()).map_err(|e| {
                        A2aParseError::InvalidParams {
                            method: self.method.clone(),
                            error: e.to_string(),
                        }
                    })?;
                Ok(A2aRequestParams::GetTaskPushNotificationConfig(params))
            },
            "tasks/pushNotificationConfig/list" => {
                let params: ListTaskPushNotificationConfigRequest =
                    serde_json::from_value(self.params.clone()).map_err(|e| {
                        A2aParseError::InvalidParams {
                            method: self.method.clone(),
                            error: e.to_string(),
                        }
                    })?;
                Ok(A2aRequestParams::ListTaskPushNotificationConfig(params))
            },
            "tasks/pushNotificationConfig/delete" => {
                let params: DeleteTaskPushNotificationConfigRequest =
                    serde_json::from_value(self.params.clone()).map_err(|e| {
                        A2aParseError::InvalidParams {
                            method: self.method.clone(),
                            error: e.to_string(),
                        }
                    })?;
                Ok(A2aRequestParams::DeleteTaskPushNotificationConfig(params))
            },
            _ => Err(A2aParseError::UnsupportedMethod {
                method: self.method.clone(),
            }),
        }
    }
}

/// Strongly-typed A2A request parameters
#[derive(Debug, Clone, PartialEq)]
pub enum A2aRequestParams {
    SendMessage(MessageSendParams),
    GetTask(TaskQueryParams),
    CancelTask(TaskIdParams),
    GetAuthenticatedExtendedCard(serde_json::Value),
    SendStreamingMessage(MessageSendParams),
    TaskResubscription(TaskResubscriptionRequest),
    SetTaskPushNotificationConfig(SetTaskPushNotificationConfigRequest),
    GetTaskPushNotificationConfig(GetTaskPushNotificationConfigRequest),
    ListTaskPushNotificationConfig(ListTaskPushNotificationConfigRequest),
    DeleteTaskPushNotificationConfig(DeleteTaskPushNotificationConfigRequest),
}

/// Parse errors for A2A requests
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum A2aParseError {
    #[error("Unsupported method: {method}")]
    UnsupportedMethod { method: String },

    #[error("Invalid parameters for method '{method}': {error}")]
    InvalidParams { method: String, error: String },
}

// ===== Helper Functions for Response Creation =====

impl A2aResponse {
    /// Create a SendMessage response from a Task
    pub fn send_message(task: Task, id: RequestId) -> Self {
        A2aResponse::SendMessage(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(task),
            error: None,
        })
    }

    /// Create a GetTask response from a Task
    pub fn get_task(task: Task, id: RequestId) -> Self {
        A2aResponse::GetTask(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(task),
            error: None,
        })
    }

    /// Create a CancelTask response from a Task
    pub fn cancel_task(task: Task, id: RequestId) -> Self {
        A2aResponse::CancelTask(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(task),
            error: None,
        })
    }
}

// ===== Event Types =====

/// Task status update event
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TaskStatusUpdateEvent {
    pub task_id: String,
    pub status: TaskStatus,
    pub timestamp: String,
}

/// Task artifact update event
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TaskArtifactUpdateEvent {
    pub task_id: String,
    pub artifacts: Vec<super::artifact::Artifact>,
    pub timestamp: String,
}

// ===== Error Types =====

/// Task not found error
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TaskNotFoundError {
    pub task_id: String,
    pub message: String,
    pub code: i32,
    pub data: serde_json::Value,
}

/// Task not cancelable error
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TaskNotCancelableError {
    pub task_id: String,
    pub state: TaskState,
    pub message: String,
    pub code: i32,
    pub data: serde_json::Value,
}

/// Unsupported operation error
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UnsupportedOperationError {
    pub operation: String,
    pub message: String,
    pub code: i32,
    pub data: serde_json::Value,
}

/// Push notification not supported error
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PushNotificationNotSupportedError {
    pub message: String,
}

// ===== Push Notification Types =====

/// Task push notification configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TaskPushNotificationConfig {
    pub id: String,
    pub push_notification_config: PushNotificationConfig,
}

/// Set task push notification config request
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SetTaskPushNotificationConfigRequest {
    pub task_id: String,
    pub config: PushNotificationConfig,
}

/// Set task push notification config response
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SetTaskPushNotificationConfigResponse {
    pub success: bool,
    pub message: Option<String>,
}

/// Get task push notification config request
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GetTaskPushNotificationConfigRequest {
    pub task_id: String,
}

/// Get task push notification config response
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GetTaskPushNotificationConfigResponse {
    pub config: Option<PushNotificationConfig>,
}

/// Get task push notification config params
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GetTaskPushNotificationConfigParams {
    pub id: String,
}

/// Delete task push notification config request
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DeleteTaskPushNotificationConfigRequest {
    pub task_id: String,
}

/// Delete task push notification config response
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DeleteTaskPushNotificationConfigResponse {
    pub success: bool,
    pub message: Option<String>,
}

/// Delete task push notification config params
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DeleteTaskPushNotificationConfigParams {
    pub id: String,
}

/// List task push notification config request
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ListTaskPushNotificationConfigRequest {
    pub task_id: String,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// List task push notification config response
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ListTaskPushNotificationConfigResponse {
    pub configs: Vec<PushNotificationConfig>,
    pub total: u32,
}

/// Task resubscription request
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TaskResubscriptionRequest {
    pub task_id: String,
    pub config: PushNotificationConfig,
}

/// Task resubscription response
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TaskResubscriptionResponse {
    pub success: bool,
    pub message: Option<String>,
}
