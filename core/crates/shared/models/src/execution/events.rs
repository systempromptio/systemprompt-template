use axum::response::sse::Event;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::event_payloads::BroadcastEventData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastEvent {
    pub event_type: String,
    pub context_id: String,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

impl BroadcastEvent {
    /// Create a new broadcast event with typed payload.
    /// This is the preferred constructor - it enforces correct event structure.
    pub fn new_typed(
        context_id: impl Into<String>,
        user_id: impl Into<String>,
        payload: BroadcastEventData,
    ) -> Self {
        Self {
            event_type: payload.event_type().to_string(),
            context_id: context_id.into(),
            user_id: user_id.into(),
            data: payload.to_data_value(),
            timestamp: Utc::now(),
        }
    }

    /// Create a new broadcast event with raw JSON data.
    /// Prefer `new_typed` when possible for type safety.
    pub fn new_raw(
        event_type: impl Into<String>,
        context_id: impl Into<String>,
        user_id: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            event_type: event_type.into(),
            context_id: context_id.into(),
            user_id: user_id.into(),
            data,
            timestamp: Utc::now(),
        }
    }

    pub fn to_sse(&self) -> Event {
        let data = match serde_json::to_string(self) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("[ERROR] Failed to serialize BroadcastEvent: {e}");
                eprintln!(
                    "[ERROR] Event type: {}, Context: {}",
                    self.event_type, self.context_id
                );
                serde_json::json!({
                    "error": "serialization_failed",
                    "event_type": self.event_type,
                    "context_id": self.context_id
                })
                .to_string()
            },
        };
        Event::default().event(&self.event_type).data(data)
    }
}
