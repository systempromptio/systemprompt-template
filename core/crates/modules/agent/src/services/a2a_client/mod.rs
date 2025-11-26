mod client;
mod error;
mod protocol;
mod streaming;
mod transport;

pub use client::{A2aClient, ClientConfig};
pub use error::{ClientError, ClientResult};
pub use protocol::{
    CancelTaskRequest, MessageConfiguration, MessageSendRequest, ProtocolHandler, TaskQueryRequest,
};
pub use streaming::{SseStream, StreamEvent};
pub use transport::{HttpTransport, Transport};
