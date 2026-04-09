use std::collections::HashMap;
use std::sync::Arc;

use systemprompt::identifiers::UserId;
use tokio::sync::{broadcast, RwLock};

#[derive(Clone, Default, Debug)]
pub struct EventHub {
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<()>>>>,
}

impl EventHub {
    #[must_use]
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn notify(&self, user_id: &UserId) {
        let channels = self.channels.read().await;
        if let Some(tx) = channels.get(user_id.as_str()) {
            let _ = tx.send(());
        }
    }

    pub async fn subscribe(&self, user_id: &UserId) -> broadcast::Receiver<()> {
        let mut channels = self.channels.write().await;
        let tx = channels
            .entry(user_id.as_str().to_string())
            .or_insert_with(|| broadcast::channel(16).0);
        tx.subscribe()
    }
}
