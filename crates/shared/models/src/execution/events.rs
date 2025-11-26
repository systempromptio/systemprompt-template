use axum::response::sse::Event;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastEvent {
    pub event_type: String,
    pub context_id: String,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

impl BroadcastEvent {
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
