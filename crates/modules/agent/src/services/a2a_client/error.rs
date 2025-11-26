use serde_json::Error as JsonError;

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] JsonError),

    #[error("Protocol error: {message}")]
    Protocol { message: String },

    #[error("Authentication failed")]
    Authentication,

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Invalid response: {reason}")]
    InvalidResponse { reason: String },

    #[error("Timeout")]
    Timeout,

    #[error("Agent error: {code} - {message}")]
    Agent { code: i32, message: String },

    #[error("Stream error: {reason}")]
    Stream { reason: String },
}

pub type ClientResult<T> = Result<T, ClientError>;

impl ClientError {
    pub fn protocol(message: impl Into<String>) -> Self {
        Self::Protocol {
            message: message.into(),
        }
    }

    pub fn invalid_response(reason: impl Into<String>) -> Self {
        Self::InvalidResponse {
            reason: reason.into(),
        }
    }

    pub fn agent(code: i32, message: impl Into<String>) -> Self {
        Self::Agent {
            code,
            message: message.into(),
        }
    }

    pub fn stream(reason: impl Into<String>) -> Self {
        Self::Stream {
            reason: reason.into(),
        }
    }
}
