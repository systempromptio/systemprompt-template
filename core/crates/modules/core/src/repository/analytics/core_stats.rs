use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, DbPool, JsonRow};
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};

#[derive(Debug, Clone)]
pub struct CoreStatsRepository {
    db_pool: DbPool,
}

impl RepositoryTrait for CoreStatsRepository {
    type Pool = DbPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db_pool
    }
}

impl CoreStatsRepository {
    pub const fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub async fn get_platform_overview(&self, hours: i32) -> Result<PlatformOverview> {
        let query = DatabaseQueryEnum::GetPlatformOverview.get(self.db_pool.as_ref());
        let row = self
            .db_pool
            .fetch_one(
                &query,
                &[&hours, &hours, &hours, &hours, &hours, &hours, &hours],
            )
            .await
            .context("Failed to fetch platform overview")?;

        PlatformOverview::from_json_row(&row)
    }

    pub async fn get_user_metrics(&self) -> Result<UserMetrics> {
        let query = DatabaseQueryEnum::GetUserMetrics.get(self.db_pool.as_ref());
        let row = self
            .db_pool
            .fetch_one(&query, &[])
            .await
            .context("Failed to fetch user metrics")?;
        UserMetrics::from_json_row(&row)
    }

    pub async fn get_cost_breakdown(&self, days: i32) -> Result<Vec<CostBreakdownRow>> {
        let query = DatabaseQueryEnum::GetCostBreakdown.get(self.db_pool.as_ref());
        let rows = self
            .db_pool
            .fetch_all(&query, &[&days])
            .await
            .context("Failed to fetch cost breakdown")?;
        rows.iter()
            .map(CostBreakdownRow::from_json_row)
            .collect()
    }

    pub async fn get_system_health(&self) -> Result<SystemHealth> {
        let query = DatabaseQueryEnum::GetSystemHealth.get(self.db_pool.as_ref());
        let row = self
            .db_pool
            .fetch_one(&query, &[])
            .await
            .context("Failed to fetch system health")?;
        SystemHealth::from_json_row(&row)
    }

    pub async fn get_top_activity(&self, days: i32, limit: i32) -> Result<TopActivity> {
        let query_users = DatabaseQueryEnum::GetTopUsers.get(self.db_pool.as_ref());
        let query_agents = DatabaseQueryEnum::GetTopAgents.get(self.db_pool.as_ref());
        let query_tools = DatabaseQueryEnum::GetTopTools.get(self.db_pool.as_ref());

        let top_users = self
            .db_pool
            .fetch_all(&query_users, &[&days, &limit])
            .await
            .context("Failed to fetch top users")?;

        let top_agents = self
            .db_pool
            .fetch_all(&query_agents, &[&days, &limit])
            .await
            .context("Failed to fetch top agents")?;

        let top_tools = self
            .db_pool
            .fetch_all(&query_tools, &[&days, &limit])
            .await
            .context("Failed to fetch top tools")?;

        Ok(TopActivity {
            users: top_users
                .iter()
                .map(TopActivityItem::from_json_row)
                .collect::<Result<Vec<_>>>()?,
            agents: top_agents
                .iter()
                .map(TopActivityItem::from_json_row)
                .collect::<Result<Vec<_>>>()?,
            tools: top_tools
                .iter()
                .map(TopActivityItem::from_json_row)
                .collect::<Result<Vec<_>>>()?,
        })
    }

    pub async fn get_activity_trend(&self, days: i32) -> Result<Vec<ActivityTrendPoint>> {
        let query = DatabaseQueryEnum::GetActivityTrend.get(self.db_pool.as_ref());
        let rows = self
            .db_pool
            .fetch_all(&query, &[&days])
            .await
            .context("Failed to fetch activity trend")?;
        rows.iter()
            .map(ActivityTrendPoint::from_json_row)
            .collect()
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct PlatformOverview {
    pub total_users: i32,
    pub active_users: i32,
    pub active_sessions: i32,
    pub ai_requests_24h: i32,
    pub cost_cents_24h: i32,
    pub cost_cents_7d: i32,
    pub avg_response_time_ms: f64,
    pub success_rate: f64,
    pub total_errors: i32,
}

impl PlatformOverview {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        Ok(Self {
            total_users: row
                .get("total_users")
                .and_then(serde_json::Value::as_i64)
                .ok_or_else(|| anyhow!("Missing total_users"))? as i32,
            active_users: row
                .get("active_users")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
            active_sessions: row
                .get("active_sessions")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
            ai_requests_24h: row
                .get("ai_requests_24h")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
            cost_cents_24h: row
                .get("cost_cents_24h")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
            cost_cents_7d: row
                .get("cost_cents_7d")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
            avg_response_time_ms: row
                .get("avg_response_time_ms")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0),
            success_rate: row
                .get("success_rate")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(1.0),
            total_errors: row
                .get("total_errors")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct UserMetrics {
    pub dau: i32,
    pub wau: i32,
    pub mau: i32,
    pub new_users_7d: i32,
    pub new_users_30d: i32,
    pub growth_rate: Option<f64>,
}

impl UserMetrics {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        Ok(Self {
            dau: row.get("dau").and_then(serde_json::Value::as_i64).unwrap_or(0) as i32,
            wau: row.get("wau").and_then(serde_json::Value::as_i64).unwrap_or(0) as i32,
            mau: row.get("mau").and_then(serde_json::Value::as_i64).unwrap_or(0) as i32,
            new_users_7d: row
                .get("new_users_7d")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
            new_users_30d: row
                .get("new_users_30d")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
            growth_rate: row.get("growth_rate").and_then(serde_json::Value::as_f64),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct CostBreakdownRow {
    pub provider: String,
    pub model: String,
    pub request_count: i32,
    pub total_tokens: i32,
    pub cost_cents: i32,
    pub avg_latency_ms: f64,
    pub unique_users: i32,
    pub unique_sessions: i32,
}

impl CostBreakdownRow {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        Ok(Self {
            provider: row
                .get("provider")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            model: row
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            request_count: row
                .get("request_count")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
            total_tokens: row
                .get("total_tokens")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
            cost_cents: row.get("cost_cents").and_then(serde_json::Value::as_i64).unwrap_or(0) as i32,
            avg_latency_ms: row
                .get("avg_latency_ms")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0),
            unique_users: row
                .get("unique_users")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
            unique_sessions: row
                .get("unique_sessions")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct SystemHealth {
    pub active_services: i32,
    pub total_services: i32,
    pub db_size_mb: f64,
    pub recent_errors: i32,
    pub recent_critical: i32,
    pub recent_warnings: i32,
    pub services_json: String,
}

impl SystemHealth {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        Ok(Self {
            active_services: row
                .get("active_services")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
            total_services: row
                .get("total_services")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
            db_size_mb: row
                .get("db_size_mb")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0),
            recent_errors: row
                .get("recent_errors")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
            recent_critical: row
                .get("recent_critical")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
            recent_warnings: row
                .get("recent_warnings")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
            services_json: row
                .get("services_json")
                .and_then(|v| v.as_str())
                .unwrap_or("[]")
                .to_string(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct TopActivity {
    pub users: Vec<TopActivityItem>,
    pub agents: Vec<TopActivityItem>,
    pub tools: Vec<TopActivityItem>,
}

#[derive(Debug, Serialize)]
pub struct TopActivityItem {
    pub rank: i32,
    pub label: String,
    pub value: i32,
    pub badge: String,
}

impl TopActivityItem {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        Ok(Self {
            rank: row.get("rank").and_then(serde_json::Value::as_i64).unwrap_or(0) as i32,
            label: row
                .get("label")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            value: row.get("value").and_then(serde_json::Value::as_i64).unwrap_or(0) as i32,
            badge: row
                .get("badge")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct ActivityTrendPoint {
    pub date: String,
    pub daily_active_users: i32,
    pub new_sessions: i32,
    pub total_requests: i32,
}

impl ActivityTrendPoint {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        Ok(Self {
            date: row
                .get("date")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            daily_active_users: row
                .get("daily_active_users")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
            new_sessions: row
                .get("new_sessions")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
            total_requests: row
                .get("total_requests")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32,
        })
    }
}
