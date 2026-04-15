use std::collections::HashMap;
use std::sync::Arc;

use systemprompt::identifiers::UserId;
use tokio::sync::{broadcast, RwLock};

#[derive(Clone, Default, Debug)]
pub struct EventHub {
    channels: Arc<RwLock<HashMap<UserId, broadcast::Sender<()>>>>,
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
        if let Some(tx) = channels.get(user_id) {
            if let Err(e) = tx.send(()) {
                tracing::debug!(
                    error = %e,
                    user_id = %user_id,
                    "EventHub notify: no active subscribers"
                );
            }
        }
    }

    pub async fn subscribe(&self, user_id: &UserId) -> broadcast::Receiver<()> {
        self.channels
            .write()
            .await
            .entry(user_id.clone())
            .or_insert_with(|| broadcast::channel(16).0)
            .subscribe()
    }
}
