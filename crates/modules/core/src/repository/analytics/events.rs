use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;

use crate::models::analytics::{AnalyticsEvent, ErrorSummary};

#[derive(Debug)]
pub struct EventsRepository {
    pool: Arc<PgPool>,
}

impl EventsRepository {
    pub fn new(db: DbPool) -> Self {
        let pool = db.pool_arc().expect("Database must be PostgreSQL");
        Self { pool }
    }

    pub async fn log_event(
        &self,
        event_type: &str,
        event_category: &str,
        severity: &str,
        user_id: &str,
        session_id: Option<&str>,
        message: Option<&str>,
        metadata: Option<&str>,
    ) -> Result<AnalyticsEvent> {
        sqlx::query_as!(
            AnalyticsEvent,
            r#"
            INSERT INTO analytics_events (event_type, event_category, severity, user_id, session_id, message, metadata, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP)
            RETURNING id, event_type, event_category, severity, user_id, session_id, message, metadata, timestamp
            "#,
            event_type,
            event_category,
            severity,
            user_id,
            session_id,
            message,
            metadata
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn get_events(
        &self,
        event_type: Option<&str>,
        event_category: Option<&str>,
        since: Option<DateTime<Utc>>,
        limit: i64,
    ) -> Result<Vec<AnalyticsEvent>> {
        sqlx::query_as!(
            AnalyticsEvent,
            r#"
            SELECT id, event_type, event_category, severity, user_id, session_id, message, metadata, timestamp
            FROM analytics_events
            WHERE ($1::text IS NULL OR event_type = $1)
              AND ($2::text IS NULL OR event_category = $2)
              AND ($3::timestamptz IS NULL OR timestamp > $3)
            ORDER BY timestamp DESC
            LIMIT $4
            "#,
            event_type,
            event_category,
            since,
            limit
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn get_error_summary(&self, hours: i32) -> Result<Vec<ErrorSummary>> {
        let cutoff = Utc::now() - Duration::hours(i64::from(hours));
        sqlx::query_as!(
            ErrorSummary,
            r#"
            SELECT
                event_type as error_type,
                COUNT(*) as "count!",
                MAX(timestamp) as "last_occurred!",
                (SELECT message FROM analytics_events ae2
                 WHERE ae2.event_type = analytics_events.event_type
                 ORDER BY timestamp DESC LIMIT 1) as sample_message
            FROM analytics_events
            WHERE event_category = 'error' AND timestamp > $1
            GROUP BY event_type
            ORDER BY "count!" DESC
            "#,
            cutoff
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn cleanup_old_events(&self, days: i32) -> Result<u64> {
        let cutoff = Utc::now() - Duration::days(i64::from(days));
        let result = sqlx::query!("DELETE FROM analytics_events WHERE timestamp < $1", cutoff)
            .execute(&*self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}
