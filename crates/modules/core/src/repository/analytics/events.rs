use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::Value;
use systemprompt_core_database::{
    DatabaseProvider, DatabaseQuery, DatabaseQueryEnum, DbPool, JsonRow, ToDbValue,
};
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};

const GET_EVENTS_BASE: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../../queries/analytics/events/postgres/get_events_base.sql"
));

const CLEANUP_OLD_EVENTS: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../../queries/analytics/events/postgres/cleanup_old_events.sql"
));

#[derive(Debug, Clone)]
pub struct EventRepository {
    db_pool: DbPool,
}

impl RepositoryTrait for EventRepository {
    type Pool = DbPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db_pool
    }
}

impl EventRepository {
    pub const fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub async fn log_event(
        &self,
        user_id: Option<&str>,
        session_id: &str,
        event_type: &str,
        event_category: &str,
        severity: &str,
        endpoint: Option<&str>,
        error_code: Option<i32>,
        response_time_ms: Option<i32>,
        agent_id: Option<&str>,
        task_id: Option<&str>,
        metadata: Value,
    ) -> Result<i64> {
        let metadata_str = metadata.to_string();
        let query = DatabaseQueryEnum::RecordEvent.get(self.db_pool.as_ref());
        let rows_affected = self
            .db_pool
            .execute(
                &query,
                &[
                    &user_id,
                    &session_id,
                    &event_type,
                    &event_category,
                    &severity,
                    &endpoint,
                    &error_code,
                    &response_time_ms,
                    &agent_id,
                    &task_id,
                    &metadata_str,
                ],
            )
            .await?;

        Ok(rows_affected as i64)
    }

    pub async fn get_events(
        &self,
        filters: EventFilters,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<AnalyticsEvent>> {
        let base_query = GET_EVENTS_BASE.postgres();
        let mut query = base_query.to_string();

        let mut params: Vec<Box<dyn ToDbValue>> = Vec::new();
        let mut param_index = 1;

        let placeholder = |idx: &mut i32| {
            let placeholder = format!("${idx}");
            *idx += 1;
            placeholder
        };

        if let Some(user_id) = &filters.user_id {
            query.push_str(&format!(" AND user_id = {}", placeholder(&mut param_index)));
            params.push(Box::new(user_id.clone()));
        }

        if let Some(session_id) = &filters.session_id {
            query.push_str(&format!(
                " AND session_id = {}",
                placeholder(&mut param_index)
            ));
            params.push(Box::new(session_id.clone()));
        }

        if let Some(event_type) = &filters.event_type {
            query.push_str(&format!(
                " AND event_type = {}",
                placeholder(&mut param_index)
            ));
            params.push(Box::new(event_type.clone()));
        }

        if let Some(category) = &filters.event_category {
            query.push_str(&format!(
                " AND event_category = {}",
                placeholder(&mut param_index)
            ));
            params.push(Box::new(category.clone()));
        }

        if let Some(severity) = &filters.severity {
            query.push_str(&format!(
                " AND severity = {}",
                placeholder(&mut param_index)
            ));
            params.push(Box::new(severity.clone()));
        }

        if let Some(start_time) = &filters.start_time {
            query.push_str(&format!(
                " AND timestamp >= {}",
                placeholder(&mut param_index)
            ));
            params.push(Box::new(start_time.clone()));
        }

        if let Some(end_time) = &filters.end_time {
            query.push_str(&format!(
                " AND timestamp <= {}",
                placeholder(&mut param_index)
            ));
            params.push(Box::new(end_time.clone()));
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {limit}"));
        }

        if let Some(offset) = offset {
            query.push_str(&format!(" OFFSET {offset}"));
        }

        let param_refs: Vec<&dyn ToDbValue> = params.iter().map(|p| &**p).collect();

        let rows = self.db_pool.fetch_all(&query, &param_refs).await?;

        rows.iter()
            .map(AnalyticsEvent::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn get_error_summary(&self, time_range_hours: i32) -> Result<Vec<ErrorSummary>> {
        let query = DatabaseQueryEnum::GetEventsSummary.get(self.db_pool.as_ref());
        let rows = self.db_pool.fetch_all(&query, &[&time_range_hours]).await?;

        rows.iter()
            .map(ErrorSummary::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn cleanup_old_events(&self, days_to_keep: i32) -> Result<u64> {
        let rows_affected = self
            .db_pool
            .execute(&CLEANUP_OLD_EVENTS, &[&days_to_keep])
            .await?;

        Ok(rows_affected)
    }
}

#[derive(Debug, Default)]
pub struct EventFilters {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub event_type: Option<String>,
    pub event_category: Option<String>,
    pub severity: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AnalyticsEvent {
    pub id: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub event_type: String,
    pub event_category: String,
    pub severity: String,
    pub endpoint: Option<String>,
    pub error_code: Option<i32>,
    pub response_time_ms: Option<i32>,
    pub agent_id: Option<String>,
    pub task_id: Option<String>,
    pub message: Option<String>,
    pub metadata: String,
    pub timestamp: String,
}

impl AnalyticsEvent {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing id"))?
            .to_string();

        let user_id = row
            .get("user_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        let session_id = row
            .get("session_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        let event_type = row
            .get("event_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing event_type"))?
            .to_string();

        let event_category = row
            .get("event_category")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing event_category"))?
            .to_string();

        let severity = row
            .get("severity")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing severity"))?
            .to_string();

        let endpoint = row
            .get("endpoint")
            .and_then(|v| v.as_str())
            .map(String::from);

        let error_code = row
            .get("error_code")
            .and_then(Value::as_i64)
            .map(|i| i as i32);

        let response_time_ms = row
            .get("response_time_ms")
            .and_then(Value::as_i64)
            .map(|i| i as i32);

        let agent_id = row
            .get("agent_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        let task_id = row
            .get("task_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        let message = row
            .get("message")
            .and_then(|v| v.as_str())
            .map(String::from);

        let metadata = row
            .get("metadata")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing metadata"))?
            .to_string();

        let timestamp = row
            .get("timestamp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing timestamp"))?
            .to_string();

        Ok(Self {
            id,
            user_id,
            session_id,
            event_type,
            event_category,
            severity,
            endpoint,
            error_code,
            response_time_ms,
            agent_id,
            task_id,
            message,
            metadata,
            timestamp,
        })
    }
}

#[derive(Debug)]
pub struct ErrorSummary {
    pub event_type: String,
    pub error_code: Option<i32>,
    pub endpoint: Option<String>,
    pub error_count: i32,
    pub affected_sessions: i32,
    pub avg_response_time: Option<f64>,
}

impl ErrorSummary {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let event_type = row
            .get("event_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing event_type"))?
            .to_string();

        let error_code = row
            .get("error_code")
            .and_then(Value::as_i64)
            .map(|i| i as i32);

        let endpoint = row
            .get("endpoint")
            .and_then(|v| v.as_str())
            .map(String::from);

        let error_count = row
            .get("error_count")
            .and_then(Value::as_i64)
            .ok_or_else(|| anyhow!("Missing error_count"))? as i32;

        let affected_sessions =
            row.get("affected_sessions")
                .and_then(Value::as_i64)
                .ok_or_else(|| anyhow!("Missing affected_sessions"))? as i32;

        let avg_response_time = row.get("avg_response_time").and_then(Value::as_f64);

        Ok(Self {
            event_type,
            error_code,
            endpoint,
            error_count,
            affected_sessions,
            avg_response_time,
        })
    }
}
