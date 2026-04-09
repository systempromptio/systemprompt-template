use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{Extension, State},
    response::sse::{Event, KeepAlive, Sse},
};
use sqlx::PgPool;
use tokio_stream::Stream;

use crate::admin::activity;
use crate::admin::repositories::dashboard_queries;
use crate::admin::types::UserContext;

pub async fn dashboard_sse(
    Extension(_user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        let mut last_event_id: Option<String> = None;
        let mut interval = tokio::time::interval(Duration::from_secs(15));

        loop {
            interval.tick().await;

            if let Ok(events) = activity::queries::fetch_new_events(&pool, last_event_id.as_deref()).await {
                if !events.is_empty() {
                    last_event_id = Some(events[0].id.clone());
                    if let Ok(json) = serde_json::to_string(&events) {
                        yield Ok(Event::default().event("activity").data(json));
                    }
                }
            }

            if let Ok(stats) = dashboard_queries::fetch_stats_snapshot(&pool).await {
                if let Ok(json) = serde_json::to_string(&stats) {
                    yield Ok(Event::default().event("stats").data(json));
                }
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}
