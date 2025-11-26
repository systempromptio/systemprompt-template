use axum::response::sse::Event;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use systemprompt_models::execution::BroadcastEvent;
use tokio::sync::{mpsc::UnboundedSender, RwLock};

pub type EventSender = UnboundedSender<Result<Event, std::convert::Infallible>>;

pub struct ContextBroadcaster {
    connections: Arc<RwLock<HashMap<String, HashMap<String, EventSender>>>>,
}

impl std::fmt::Debug for ContextBroadcaster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextBroadcaster")
            .field("connections", &"<RwLock<HashMap>>")
            .finish()
    }
}

impl ContextBroadcaster {
    fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, user_id: &str, conn_id: &str, sender: EventSender) {
        let mut conns = self.connections.write().await;
        let user_conns = conns
            .entry(user_id.to_string())
            .or_insert_with(HashMap::new);
        user_conns.insert(conn_id.to_string(), sender);
    }

    pub async fn unregister(&self, user_id: &str, conn_id: &str) {
        let mut conns = self.connections.write().await;
        if let Some(user_conns) = conns.get_mut(user_id) {
            user_conns.remove(conn_id);

            if user_conns.is_empty() {
                conns.remove(user_id);
            }
        }
    }

    pub async fn broadcast_to_user(&self, user_id: &str, event: BroadcastEvent) -> usize {
        let mut conns = self.connections.write().await;

        if let Some(user_conns) = conns.get_mut(user_id) {
            let mut successful = 0;
            let sse_event = event.to_sse();

            user_conns.retain(|_conn_id, sender| {
                if sender.send(Ok(sse_event.clone())).is_ok() {
                    successful += 1;
                    true
                } else {
                    false
                }
            });

            if user_conns.is_empty() {
                conns.remove(user_id);
            }

            successful
        } else {
            0
        }
    }

    pub async fn connection_count(&self, user_id: &str) -> usize {
        let conns = self.connections.read().await;
        conns.get(user_id).map_or(0, HashMap::len)
    }

    pub async fn total_connections(&self) -> usize {
        let conns = self.connections.read().await;
        conns.values().map(HashMap::len).sum()
    }

    pub async fn connected_users(&self) -> Vec<String> {
        let conns = self.connections.read().await;
        conns.keys().cloned().collect()
    }
}

pub static CONTEXT_BROADCASTER: Lazy<ContextBroadcaster> = Lazy::new(ContextBroadcaster::new);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_broadcast() {
        let broadcaster = ContextBroadcaster::new();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        broadcaster.register("user1", "conn1", tx).await;

        let event = BroadcastEvent {
            event_type: "test".to_string(),
            context_id: "ctx1".to_string(),
            user_id: "user1".to_string(),
            data: serde_json::json!({}),
            timestamp: chrono::Utc::now(),
        };

        let count = broadcaster.broadcast_to_user("user1", event).await;
        assert_eq!(count, 1);

        let received = rx.recv().await;
        assert!(received.is_some());
    }

    #[tokio::test]
    async fn test_unregister() {
        let broadcaster = ContextBroadcaster::new();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

        broadcaster.register("user1", "conn1", tx).await;
        assert_eq!(broadcaster.connection_count("user1").await, 1);

        broadcaster.unregister("user1", "conn1").await;
        assert_eq!(broadcaster.connection_count("user1").await, 0);
    }

    #[tokio::test]
    async fn test_dead_channel_cleanup() {
        let broadcaster = ContextBroadcaster::new();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        broadcaster.register("user1", "conn1", tx).await;
        drop(rx);

        let event = BroadcastEvent {
            event_type: "test".to_string(),
            context_id: "ctx1".to_string(),
            user_id: "user1".to_string(),
            data: serde_json::json!({}),
            timestamp: chrono::Utc::now(),
        };

        let count = broadcaster.broadcast_to_user("user1", event).await;
        assert_eq!(count, 0);
        assert_eq!(broadcaster.connection_count("user1").await, 0);
    }
}
