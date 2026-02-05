use crate::models::engagement::{EngagementEvent, EngagementEventRequest};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::{ContentId, EngagementEventId, SessionId, UserId};

#[derive(Debug, Clone)]
pub struct EngagementRepository {
    pool: Arc<PgPool>,
}

impl EngagementRepository {
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    #[allow(clippy::cast_precision_loss)]
    pub async fn create_event(
        &self,
        session_id: &SessionId,
        user_id: &UserId,
        request: &EngagementEventRequest,
    ) -> Result<EngagementEvent, sqlx::Error> {
        let id = EngagementEventId::generate();
        let now = Utc::now();
        let data = &request.data;

        sqlx::query_as!(
            EngagementEvent,
            r#"
            INSERT INTO engagement_events (
                id, session_id, user_id, page_url, content_id,
                time_on_page_ms, time_to_first_interaction_ms, time_to_first_scroll_ms,
                max_scroll_depth, scroll_velocity_avg, scroll_direction_changes,
                click_count, mouse_move_distance_px, keyboard_events, copy_events,
                focus_time_ms, blur_count, tab_switches,
                visible_time_ms, hidden_time_ms,
                is_rage_click, is_dead_click, reading_pattern,
                created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $24
            )
            RETURNING
                id as "id: EngagementEventId",
                session_id as "session_id: SessionId",
                user_id as "user_id: UserId",
                page_url,
                content_id as "content_id: ContentId",
                time_on_page_ms,
                time_to_first_interaction_ms,
                time_to_first_scroll_ms,
                max_scroll_depth,
                scroll_velocity_avg,
                scroll_direction_changes,
                click_count,
                mouse_move_distance_px,
                keyboard_events,
                copy_events,
                focus_time_ms,
                blur_count,
                tab_switches,
                visible_time_ms,
                hidden_time_ms,
                is_rage_click,
                is_dead_click,
                reading_pattern,
                created_at,
                updated_at
            "#,
            id.as_str(),
            session_id.as_str(),
            user_id.as_str(),
            &request.page_url,
            request.content_id.as_deref(),
            data.as_ref().and_then(|d| d.time_on_page_ms).unwrap_or(0),
            data.as_ref().and_then(|d| d.time_to_first_interaction_ms),
            data.as_ref().and_then(|d| d.time_to_first_scroll_ms),
            data.as_ref().and_then(|d| d.max_scroll_depth).unwrap_or(0),
            data.as_ref()
                .and_then(|d| d.scroll_velocity_avg)
                .map(|v| v as f32),
            data.as_ref().and_then(|d| d.scroll_direction_changes),
            data.as_ref().and_then(|d| d.click_count).unwrap_or(0),
            data.as_ref().and_then(|d| d.mouse_move_distance_px),
            data.as_ref().and_then(|d| d.keyboard_events),
            data.as_ref().and_then(|d| d.copy_events),
            data.as_ref().and_then(|d| d.focus_time_ms).unwrap_or(0),
            data.as_ref().and_then(|d| d.blur_count).unwrap_or(0),
            data.as_ref().and_then(|d| d.tab_switches).unwrap_or(0),
            data.as_ref().and_then(|d| d.visible_time_ms).unwrap_or(0),
            data.as_ref().and_then(|d| d.hidden_time_ms).unwrap_or(0),
            data.as_ref().and_then(|d| d.is_rage_click),
            data.as_ref().and_then(|d| d.is_dead_click),
            data.as_ref().and_then(|d| d.reading_pattern.clone()),
            now
        )
        .fetch_one(&*self.pool)
        .await
    }

    pub async fn create_events_batch(
        &self,
        session_id: &SessionId,
        user_id: &UserId,
        requests: &[EngagementEventRequest],
    ) -> Result<Vec<EngagementEvent>, sqlx::Error> {
        let mut events = Vec::with_capacity(requests.len());
        for request in requests {
            let event = self.create_event(session_id, user_id, request).await?;
            events.push(event);
        }
        Ok(events)
    }

    pub async fn get_events_by_page(
        &self,
        page_url: &str,
        limit: i64,
    ) -> Result<Vec<EngagementEvent>, sqlx::Error> {
        sqlx::query_as!(
            EngagementEvent,
            r#"
            SELECT
                id as "id: EngagementEventId",
                session_id as "session_id: SessionId",
                user_id as "user_id: UserId",
                page_url,
                content_id as "content_id: ContentId",
                time_on_page_ms,
                time_to_first_interaction_ms,
                time_to_first_scroll_ms,
                max_scroll_depth,
                scroll_velocity_avg,
                scroll_direction_changes,
                click_count,
                mouse_move_distance_px,
                keyboard_events,
                copy_events,
                focus_time_ms,
                blur_count,
                tab_switches,
                visible_time_ms,
                hidden_time_ms,
                is_rage_click,
                is_dead_click,
                reading_pattern,
                created_at,
                updated_at
            FROM engagement_events
            WHERE page_url = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            page_url,
            limit
        )
        .fetch_all(&*self.pool)
        .await
    }

    pub async fn get_events_by_session(
        &self,
        session_id: &SessionId,
    ) -> Result<Vec<EngagementEvent>, sqlx::Error> {
        sqlx::query_as!(
            EngagementEvent,
            r#"
            SELECT
                id as "id: EngagementEventId",
                session_id as "session_id: SessionId",
                user_id as "user_id: UserId",
                page_url,
                content_id as "content_id: ContentId",
                time_on_page_ms,
                time_to_first_interaction_ms,
                time_to_first_scroll_ms,
                max_scroll_depth,
                scroll_velocity_avg,
                scroll_direction_changes,
                click_count,
                mouse_move_distance_px,
                keyboard_events,
                copy_events,
                focus_time_ms,
                blur_count,
                tab_switches,
                visible_time_ms,
                hidden_time_ms,
                is_rage_click,
                is_dead_click,
                reading_pattern,
                created_at,
                updated_at
            FROM engagement_events
            WHERE session_id = $1
            ORDER BY created_at ASC
            "#,
            session_id.as_str()
        )
        .fetch_all(&*self.pool)
        .await
    }
}
