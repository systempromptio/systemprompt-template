//! `GET /admin/api/sse/overview/{pane}` — Server-Sent Events for the five
//! Live Overview panes.
//!
//! Subscribes to the global [`crate::audit_event_bus`] (which fans the
//! Postgres `LISTEN audit_events` channel out to in-process subscribers) and
//! filters / re-emits per-pane typed events. Each pane receives only the
//! events its UI knows how to render — the on-the-wire `event:` name maps to
//! a JS handler in the matching `admin-overview-<pane>.js` module.
//!
//! Wire-level event names per pane:
//!
//! | pane         | events                                          |
//! |--------------|-------------------------------------------------|
//! | `pulse`      | `request`, `decision`, `tool`                   |
//! | `identity`   | `request`, `decision`                           |
//! | `cost`       | `request`                                       |
//! | `governance` | `decision`                                      |
//! | `services`   | `request`, `tool`                               |
//! | `index`      | All of the above (used by the summary cards)    |

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{Extension, Path, State},
    response::sse::{Event, KeepAlive, Sse},
};
use sqlx::PgPool;
use tokio_stream::Stream;

use crate::audit_event_bus;
use crate::types::UserContext;

#[derive(Debug, Clone, Copy)]
enum Pane {
    Index,
    Pulse,
    Identity,
    Cost,
    Governance,
    Services,
}

impl Pane {
    fn parse(s: &str) -> Self {
        match s {
            "pulse" => Self::Pulse,
            "identity" => Self::Identity,
            "cost" => Self::Cost,
            "governance" => Self::Governance,
            "services" => Self::Services,
            _ => Self::Index,
        }
    }

    const fn wants(self, kind: EventKind) -> bool {
        matches!(
            (self, kind),
            (Self::Index, _)
                | (
                    Self::Pulse,
                    EventKind::Request | EventKind::Decision | EventKind::Tool
                )
                | (Self::Identity, EventKind::Request | EventKind::Decision)
                | (Self::Cost, EventKind::Request)
                | (Self::Governance, EventKind::Decision)
                | (Self::Services, EventKind::Request | EventKind::Tool)
        )
    }
}

#[derive(Debug, Clone, Copy)]
enum EventKind {
    Request,
    Decision,
    Tool,
}

impl EventKind {
    const fn name(self) -> &'static str {
        match self {
            Self::Request => "request",
            Self::Decision => "decision",
            Self::Tool => "tool",
        }
    }
}

/// Classify a raw `audit_events` NOTIFY payload into an `EventKind`.
///
/// The trigger emits `{"table": "ai_requests" | "governance_decisions" |
/// "plugin_usage_events", ...}`. Anything else is dropped silently.
fn classify(payload: &str) -> Option<EventKind> {
    let v: serde_json::Value = serde_json::from_str(payload).ok()?;
    match v.get("table").and_then(|t| t.as_str())? {
        "ai_requests" => Some(EventKind::Request),
        "governance_decisions" => Some(EventKind::Decision),
        "plugin_usage_events" | "mcp_tool_executions" => Some(EventKind::Tool),
        _ => None,
    }
}

pub async fn overview_sse(
    Path(pane): Path<String>,
    Extension(_user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let pane = Pane::parse(&pane);
    let bus = audit_event_bus::get_or_init(pool);
    let mut rx = bus.subscribe();

    let stream = async_stream::stream! {
        yield Ok(Event::default().event("hello").data("{}"));

        loop {
            match rx.recv().await {
                Ok(payload) => {
                    if let Some(kind) = classify(&payload) {
                        if pane.wants(kind) {
                            yield Ok(Event::default().event(kind.name()).data(payload));
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    let msg = format!(r#"{{"skipped":{skipped}}}"#);
                    yield Ok(Event::default().event("lagged").data(msg));
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}
