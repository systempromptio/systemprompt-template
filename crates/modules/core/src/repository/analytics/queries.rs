use anyhow::{anyhow, Result};
use serde::Serialize;
use systemprompt_core_database::{DatabaseProvider, DatabaseQuery, DbPool, JsonRow, ToDbValue};
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};

const GET_USER_ANALYTICS_SUMMARY: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../../queries/analytics/queries/postgres/get_user_analytics_summary.sql"
));

const GET_TOP_USERS_SUMMARY: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../../queries/analytics/queries/postgres/get_top_users_summary.sql"
));

const GET_DAILY_ACTIVITY_TREND_BASE: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../../queries/analytics/queries/postgres/get_daily_activity_trend_base.sql"
));

const GET_AGENT_USAGE_ANALYTICS_BASE: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../../queries/analytics/queries/postgres/get_agent_usage_analytics_base.sql"
));

const GET_SYSTEM_HEALTH_METRICS: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../../queries/analytics/queries/postgres/get_system_health_metrics.sql"
));

#[derive(Debug, Clone)]
pub struct AnalyticsQueryRepository {
    db_pool: DbPool,
}

impl RepositoryTrait for AnalyticsQueryRepository {
    type Pool = DbPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db_pool
    }
}

impl AnalyticsQueryRepository {
    pub const fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub async fn get_user_analytics_summary(
        &self,
        user_id: &str,
        days: i32,
    ) -> Result<UserAnalyticsSummary> {
        let row = self
            .db_pool
            .fetch_one(&GET_USER_ANALYTICS_SUMMARY, &[&user_id, &days])
            .await?;

        UserAnalyticsSummary::from_json_row(&row)
    }

    pub async fn get_top_users_summary(
        &self,
        days: i32,
        limit: i32,
    ) -> Result<Vec<TopUserSummary>> {
        let rows = self
            .db_pool
            .fetch_all(&GET_TOP_USERS_SUMMARY, &[&days, &limit])
            .await?;

        rows.iter()
            .map(TopUserSummary::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn get_daily_activity_trend(
        &self,
        days: i32,
        user_id: Option<&str>,
    ) -> Result<Vec<DailyActivity>> {
        let base_query = GET_DAILY_ACTIVITY_TREND_BASE.postgres();
        let mut query = base_query.to_string();

        let mut params: Vec<Box<dyn ToDbValue>> = vec![Box::new(days)];
        let mut param_index = 1;

        let placeholder = |idx: &mut i32| {
            let placeholder = format!("${idx}");
            *idx += 1;
            placeholder
        };

        if let Some(uid) = user_id {
            query.push_str(&format!(
                " AND s.user_id = {}",
                placeholder(&mut param_index)
            ));
            params.push(Box::new(uid.to_string()));
        }

        query.push_str(" GROUP BY DATE(s.started_at) ORDER BY activity_date DESC");

        let param_refs: Vec<&dyn ToDbValue> = params.iter().map(|p| &**p).collect();

        let rows = self.db_pool.fetch_all(&query, &param_refs).await?;

        rows.iter()
            .map(DailyActivity::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn get_agent_usage_analytics(
        &self,
        agent_id: Option<&str>,
        days: i32,
    ) -> Result<Vec<AgentUsageAnalytics>> {
        let base_query = GET_AGENT_USAGE_ANALYTICS_BASE.postgres();
        let mut query = base_query.to_string();

        let mut params: Vec<Box<dyn ToDbValue>> = vec![Box::new(days)];
        let mut param_index = 1;

        let placeholder = |idx: &mut i32| {
            let placeholder = format!("${idx}");
            *idx += 1;
            placeholder
        };

        if let Some(aid) = agent_id {
            let json_extract = format!(
                "(t.metadata->'agent_id')::text = {}",
                placeholder(&mut param_index)
            );
            query.push_str(&format!(" AND {json_extract}"));
            params.push(Box::new(aid.to_string()));
        }

        query.push_str(" GROUP BY (t.metadata->'agent_id')::text ORDER BY total_tasks DESC");

        let param_refs: Vec<&dyn ToDbValue> = params.iter().map(|p| &**p).collect();

        let rows = self.db_pool.fetch_all(&query, &param_refs).await?;

        rows.iter()
            .map(AgentUsageAnalytics::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn get_ai_provider_usage(
        &self,
        days: i32,
        user_id: Option<&str>,
    ) -> Result<Vec<ProviderUsage>> {
        let base_query = r"
            SELECT
                provider,
                model,
                COUNT(*) as request_count,
                SUM(tokens_used) as total_tokens,
                SUM(cost_cents) as total_cost_cents,
                AVG(latency_ms) as avg_latency_ms,
                COUNT(DISTINCT user_id) as unique_users,
                COUNT(DISTINCT session_id) as unique_sessions
            FROM ai_requests
            WHERE created_at >= NOW() - INTERVAL '1 day' * $1
            ";

        let mut query = base_query.to_string();
        let mut params: Vec<Box<dyn ToDbValue>> = vec![Box::new(days)];
        let mut param_index = 2;

        let placeholder = |idx: &mut i32| {
            let placeholder = format!("${idx}");
            *idx += 1;
            placeholder
        };

        if let Some(uid) = user_id {
            query.push_str(&format!(" AND user_id = {}", placeholder(&mut param_index)));
            params.push(Box::new(uid.to_string()));
        }

        query.push_str(" GROUP BY provider, model ORDER BY request_count DESC");

        let param_refs: Vec<&dyn ToDbValue> = params.iter().map(|p| &**p).collect();

        let rows = self.db_pool.fetch_all(&query, &param_refs).await?;

        rows.iter()
            .map(ProviderUsage::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn get_system_health_metrics(&self, hours: i32) -> Result<SystemHealthMetrics> {
        let row = self
            .db_pool
            .fetch_one(
                &GET_SYSTEM_HEALTH_METRICS,
                &[&hours, &hours, &hours, &hours, &hours, &hours, &hours],
            )
            .await?;

        SystemHealthMetrics::from_json_row(&row)
    }
}

#[derive(Debug, Copy, Clone, Serialize)]
pub struct UserAnalyticsSummary {
    pub total_sessions: i32,
    pub total_requests: Option<i32>,
    pub total_ai_requests: Option<i32>,
    pub total_tokens: Option<i32>,
    pub total_cost_cents: Option<i32>,
    pub avg_response_time: Option<f64>,
    pub total_tasks: Option<i32>,
    pub total_messages: Option<i32>,
    pub active_days: i32,
    pub total_errors: Option<i32>,
    pub avg_success_rate: Option<f64>,
}

impl UserAnalyticsSummary {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let total_sessions = row
            .get("total_sessions")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing total_sessions"))? as i32;

        let total_requests = row
            .get("total_requests")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let total_ai_requests = row
            .get("total_ai_requests")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let total_tokens = row
            .get("total_tokens")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let total_cost_cents = row
            .get("total_cost_cents")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let avg_response_time = row.get("avg_response_time").and_then(serde_json::Value::as_f64);

        let total_tasks = row
            .get("total_tasks")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let total_messages = row
            .get("total_messages")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let active_days = row
            .get("active_days")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing active_days"))? as i32;

        let total_errors = row
            .get("total_errors")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let avg_success_rate = row.get("avg_success_rate").and_then(serde_json::Value::as_f64);

        Ok(Self {
            total_sessions,
            total_requests,
            total_ai_requests,
            total_tokens,
            total_cost_cents,
            avg_response_time,
            total_tasks,
            total_messages,
            active_days,
            total_errors,
            avg_success_rate,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct TopUserSummary {
    pub user_id: Option<String>,
    pub total_sessions: i32,
    pub total_requests: Option<i32>,
    pub total_ai_requests: Option<i32>,
    pub total_tokens: Option<i32>,
    pub total_cost_cents: Option<i32>,
    pub avg_response_time: Option<f64>,
    pub active_days: i32,
    pub total_errors: Option<i32>,
    pub avg_success_rate: Option<f64>,
}

impl TopUserSummary {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let user_id = row
            .get("user_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        let total_sessions = row
            .get("total_sessions")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing total_sessions"))? as i32;

        let total_requests = row
            .get("total_requests")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let total_ai_requests = row
            .get("total_ai_requests")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let total_tokens = row
            .get("total_tokens")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let total_cost_cents = row
            .get("total_cost_cents")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let avg_response_time = row.get("avg_response_time").and_then(serde_json::Value::as_f64);

        let active_days = row
            .get("active_days")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing active_days"))? as i32;

        let total_errors = row
            .get("total_errors")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let avg_success_rate = row.get("avg_success_rate").and_then(serde_json::Value::as_f64);

        Ok(Self {
            user_id,
            total_sessions,
            total_requests,
            total_ai_requests,
            total_tokens,
            total_cost_cents,
            avg_response_time,
            active_days,
            total_errors,
            avg_success_rate,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct DailyActivity {
    pub activity_date: String,
    pub sessions: i32,
    pub unique_users: i32,
    pub total_requests: Option<i32>,
    pub ai_requests: Option<i32>,
    pub tokens_used: Option<i32>,
    pub cost_cents: Option<i32>,
    pub avg_response_time: Option<f64>,
}

impl DailyActivity {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let activity_date = row
            .get("activity_date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing activity_date"))?
            .to_string();

        let sessions = row
            .get("sessions")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing sessions"))? as i32;

        let unique_users = row
            .get("unique_users")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing unique_users"))? as i32;

        let total_requests = row
            .get("total_requests")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let ai_requests = row
            .get("ai_requests")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let tokens_used = row
            .get("tokens_used")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let cost_cents = row
            .get("cost_cents")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let avg_response_time = row.get("avg_response_time").and_then(serde_json::Value::as_f64);

        Ok(Self {
            activity_date,
            sessions,
            unique_users,
            total_requests,
            ai_requests,
            tokens_used,
            cost_cents,
            avg_response_time,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct AgentUsageAnalytics {
    pub agent_id: Option<String>,
    pub total_tasks: i32,
    pub unique_users: i32,
    pub unique_sessions: i32,
    pub total_messages: i32,
    pub avg_completion_time_seconds: Option<f64>,
    pub failed_tasks: i32,
    pub completed_tasks: i32,
}

impl AgentUsageAnalytics {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let agent_id = row
            .get("agent_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        let total_tasks = row
            .get("total_tasks")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing total_tasks"))? as i32;

        let unique_users = row
            .get("unique_users")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing unique_users"))? as i32;

        let unique_sessions =
            row.get("unique_sessions")
                .and_then(serde_json::Value::as_i64)
                .ok_or_else(|| anyhow!("Missing unique_sessions"))? as i32;

        let total_messages = row
            .get("total_messages")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing total_messages"))? as i32;

        let avg_completion_time_seconds = row
            .get("avg_completion_time_seconds")
            .and_then(serde_json::Value::as_f64);

        let failed_tasks = row
            .get("failed_tasks")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing failed_tasks"))? as i32;

        let completed_tasks =
            row.get("completed_tasks")
                .and_then(serde_json::Value::as_i64)
                .ok_or_else(|| anyhow!("Missing completed_tasks"))? as i32;

        Ok(Self {
            agent_id,
            total_tasks,
            unique_users,
            unique_sessions,
            total_messages,
            avg_completion_time_seconds,
            failed_tasks,
            completed_tasks,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct ProviderUsage {
    pub provider: String,
    pub model: String,
    pub request_count: i32,
    pub total_tokens: Option<i32>,
    pub total_cost_cents: Option<i32>,
    pub avg_latency_ms: Option<f64>,
    pub unique_users: i32,
    pub unique_sessions: i32,
}

impl ProviderUsage {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let provider = row
            .get("provider")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing provider"))?
            .to_string();

        let model = row
            .get("model")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing model"))?
            .to_string();

        let request_count = row
            .get("request_count")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing request_count"))? as i32;

        let total_tokens = row
            .get("total_tokens")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let total_cost_cents = row
            .get("total_cost_cents")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let avg_latency_ms = row.get("avg_latency_ms").and_then(serde_json::Value::as_f64);

        let unique_users = row
            .get("unique_users")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing unique_users"))? as i32;

        let unique_sessions =
            row.get("unique_sessions")
                .and_then(serde_json::Value::as_i64)
                .ok_or_else(|| anyhow!("Missing unique_sessions"))? as i32;

        Ok(Self {
            provider,
            model,
            request_count,
            total_tokens,
            total_cost_cents,
            avg_latency_ms,
            unique_users,
            unique_sessions,
        })
    }
}

#[derive(Debug, Copy, Clone, Serialize)]
pub struct SystemHealthMetrics {
    pub active_sessions: i32,
    pub total_requests: Option<i32>,
    pub system_avg_response_time: Option<f64>,
    pub total_errors: Option<i32>,
    pub system_success_rate: Option<f64>,
    pub active_users: i32,
    pub critical_events: i32,
    pub error_events: i32,
}

impl SystemHealthMetrics {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let active_sessions =
            row.get("active_sessions")
                .and_then(serde_json::Value::as_i64)
                .ok_or_else(|| anyhow!("Missing active_sessions"))? as i32;

        let total_requests = row
            .get("total_requests")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let system_avg_response_time = row.get("system_avg_response_time").and_then(serde_json::Value::as_f64);

        let total_errors = row
            .get("total_errors")
            .and_then(serde_json::Value::as_i64)
            .map(|i| i as i32);

        let system_success_rate = row.get("system_success_rate").and_then(serde_json::Value::as_f64);

        let active_users = row
            .get("active_users")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing active_users"))? as i32;

        let critical_events =
            row.get("critical_events")
                .and_then(serde_json::Value::as_i64)
                .ok_or_else(|| anyhow!("Missing critical_events"))? as i32;

        let error_events = row
            .get("error_events")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing error_events"))? as i32;

        Ok(Self {
            active_sessions,
            total_requests,
            system_avg_response_time,
            total_errors,
            system_success_rate,
            active_users,
            critical_events,
            error_events,
        })
    }
}
