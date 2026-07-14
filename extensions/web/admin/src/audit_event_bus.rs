//! Global audit-event bus.
//!
//! One [`sqlx::postgres::PgListener`] runs per process, subscribed to the
//! Postgres `audit_events` LISTEN channel. Each NOTIFY payload (a JSON blob
//! emitted by the `audit_event_notify` triggers) is forwarded to a Tokio
//! broadcast channel; SSE handlers subscribe to that channel.
//!
//! The listener is started lazily on first subscription and survives for the
//! lifetime of the process. Reconnects are handled by `PgListener::recv`
//! itself (it transparently re-subscribes on transient errors).

use std::sync::{Arc, OnceLock};

use sqlx::PgPool;
use sqlx::postgres::PgListener;
use tokio::sync::broadcast;

const CHANNEL_NAME: &str = "audit_events";
const BROADCAST_CAPACITY: usize = 256;

#[derive(Clone, Debug)]
pub struct AuditEventBus {
    sender: broadcast::Sender<String>,
}

impl AuditEventBus {
    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.sender.subscribe()
    }
}

static BUS: OnceLock<AuditEventBus> = OnceLock::new();

/// Bus accessor that survives a missing `audit_events` channel.
///
/// If the listener fails to start (e.g. the `audit_events` channel doesn't
/// exist yet because the migration hasn't run), the bus is still returned and
/// SSE subscribers receive an empty stream — keep-alives but no payloads.
pub fn get_or_init(pool: Arc<PgPool>) -> AuditEventBus {
    BUS.get_or_init(|| {
        let (sender, _) = broadcast::channel::<String>(BROADCAST_CAPACITY);
        spawn_listener(pool, sender.clone());
        AuditEventBus { sender }
    })
    .clone()
}

fn spawn_listener(pool: Arc<PgPool>, sender: broadcast::Sender<String>) {
    tokio::spawn(async move {
        loop {
            match PgListener::connect_with(&pool).await {
                Ok(mut listener) => {
                    if let Err(e) = listener.listen(CHANNEL_NAME).await {
                        tracing::warn!(
                            error = %e,
                            "audit_event_bus: LISTEN {CHANNEL_NAME} failed; retrying"
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        continue;
                    }
                    tracing::info!("audit_event_bus: listening on {CHANNEL_NAME}");
                    loop {
                        match listener.recv().await {
                            Ok(notification) => {
                                let payload = notification.payload().to_owned();
                                // Why: `broadcast::Sender::send` returns Err when there are
                                // zero subscribers. That's a normal idle state for this bus
                                // — no SSE clients connected — not a failure to log.
                                drop(sender.send(payload));
                            },
                            Err(e) => {
                                tracing::warn!(
                                    error = %e,
                                    "audit_event_bus: recv failed; reconnecting"
                                );
                                break;
                            },
                        }
                    }
                },
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "audit_event_bus: PgListener::connect failed; retrying"
                    );
                },
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });
}
