//! `GET /admin/api/sse/audit` — Server-Sent Events stream of governance and
//! gateway activity.
//!
//! Backed by the global [`crate::audit_event_bus`] which fans one Postgres
//! `LISTEN audit_events` connection out to every connected client. Payload
//! is the JSON emitted by the `audit_event_notify` triggers.

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{Extension, State},
    response::sse::{Event, KeepAlive, Sse},
};
use sqlx::PgPool;
use tokio_stream::Stream;

use crate::audit_event_bus;
use crate::types::UserContext;

pub async fn audit_sse(
    Extension(_user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let bus = audit_event_bus::get_or_init(pool);
    let mut rx = bus.subscribe();

    let stream = async_stream::stream! {
        // Initial hello — lets the client distinguish "connected, no events
        // yet" from "still connecting".
        yield Ok(Event::default().event("hello").data("{}"));

        loop {
            match rx.recv().await {
                Ok(payload) => {
                    yield Ok(Event::default().event("audit").data(payload));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    let msg = format!(r#"{{"skipped":{skipped}}}"#);
                    yield Ok(Event::default().event("lagged").data(msg));
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    // Bus is global so this should never close, but if it did
                    // we end the stream cleanly.
                    break;
                }
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}
