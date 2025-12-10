use axum::response::sse::Event;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use systemprompt_models::execution::BroadcastEvent;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;

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
