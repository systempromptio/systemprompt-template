use crate::models::a2a::jsonrpc::NumberOrString;
use axum::http::StatusCode;
use serde_json::{json, Value};
use systemprompt_core_logging::{LogLevel, LogService};

#[derive(Debug)]
pub struct JsonRpcErrorBuilder {
    code: i32,
    message: String,
    data: Option<Value>,
    log_message: Option<String>,
    log_level: LogLevel,
}

impl JsonRpcErrorBuilder {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
            log_message: None,
            log_level: LogLevel::Error,
        }
    }

    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn with_log(mut self, message: impl Into<String>, level: LogLevel) -> Self {
        self.log_message = Some(message.into());
        self.log_level = level;
        self
    }

    pub fn log_error(mut self, message: impl Into<String>) -> Self {
        self.log_message = Some(message.into());
        self.log_level = LogLevel::Error;
        self
    }

    pub fn log_warn(mut self, message: impl Into<String>) -> Self {
        self.log_message = Some(message.into());
        self.log_level = LogLevel::Warn;
        self
    }

    pub async fn build(self, request_id: &NumberOrString, log: &LogService) -> Value {
        if let Some(log_msg) = self.log_message {
            match self.log_level {
                LogLevel::Error => {
                    log.error("a2a_jsonrpc", &log_msg).await.ok();
                },
                LogLevel::Warn => {
                    log.warn("a2a_jsonrpc", &log_msg).await.ok();
                },
                LogLevel::Info => {
                    log.info("a2a_jsonrpc", &log_msg).await.ok();
                },
                LogLevel::Debug => {
                    log.debug("a2a_jsonrpc", &log_msg).await.ok();
                },
                LogLevel::Trace => {
                    log.debug("a2a_jsonrpc", &log_msg).await.ok();
                },
            }
        }

        let mut error = json!({
            "code": self.code,
            "message": self.message
        });

        if let Some(data) = self.data {
            error["data"] = data;
        }

        json!({
            "jsonrpc": "2.0",
            "error": error,
            "id": request_id
        })
    }

    pub async fn build_with_status(
        self,
        request_id: &NumberOrString,
        log: &LogService,
    ) -> (StatusCode, Value) {
        let status = match self.code {
            -32600 => StatusCode::BAD_REQUEST,
            -32601 => StatusCode::NOT_FOUND,
            -32602 => StatusCode::BAD_REQUEST,
            -32603 => StatusCode::INTERNAL_SERVER_ERROR,
            -32700 => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.build(request_id, log).await)
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

    pub fn parse_error() -> Self {
        Self::new(-32700, "Parse error")
    }

    pub fn unauthorized(reason: impl Into<String>) -> Self {
        Self::new(-32600, "Unauthorized").with_data(json!({
            "reason": reason.into()
        }))
    }

    pub fn forbidden(reason: impl Into<String>) -> Self {
        Self::new(-32600, "Forbidden").with_data(json!({
            "reason": reason.into()
        }))
    }
}

pub async fn unauthorized_response(
    reason: impl Into<String>,
    request_id: &NumberOrString,
    log: &LogService,
) -> (StatusCode, Value) {
    let reason_str = reason.into();
    (
        StatusCode::UNAUTHORIZED,
        JsonRpcErrorBuilder::unauthorized(&reason_str)
            .log_warn(&reason_str)
            .build(request_id, log)
            .await,
    )
}

pub async fn forbidden_response(
    reason: impl Into<String>,
    request_id: &NumberOrString,
    log: &LogService,
) -> (StatusCode, Value) {
    let reason_str = reason.into();
    (
        StatusCode::FORBIDDEN,
        JsonRpcErrorBuilder::forbidden(&reason_str)
            .log_warn(&reason_str)
            .build(request_id, log)
            .await,
    )
}
