use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt_core_database::JsonRow;

/// Blog article performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogMetrics {
    pub id: String,
    pub content_id: String,
    pub total_views: i32,
    pub unique_visitors: i32,
    pub avg_time_on_page_seconds: i32,
    pub shares_total: i32,
    pub shares_linkedin: i32,
    pub shares_twitter: i32,
    pub comments_count: i32,
    pub search_impressions: i32,
    pub search_clicks: i32,
    pub avg_search_position: Option<f64>,
    pub views_last_7_days: i32,
    pub views_last_30_days: i32,
    pub trend_direction: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BlogMetrics {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        use systemprompt_core_database::parse_database_datetime;

        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing id"))?
            .to_string();

        let content_id = row
            .get("content_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing content_id"))?
            .to_string();

        let total_views = row
            .get("total_views")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing total_views"))? as i32;

        let unique_visitors =
            row.get("unique_visitors")
                .and_then(serde_json::Value::as_i64)
                .ok_or_else(|| anyhow!("Missing unique_visitors"))? as i32;

        let avg_time_on_page_seconds = row
            .get("avg_time_on_page_seconds")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid avg_time_on_page_seconds"))? as i32;

        let shares_total = row
            .get("shares_total")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid shares_total"))? as i32;

        let shares_linkedin = row
            .get("shares_linkedin")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid shares_linkedin"))? as i32;

        let shares_twitter = row
            .get("shares_twitter")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid shares_twitter"))? as i32;

        let comments_count = row
            .get("comments_count")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid comments_count"))? as i32;

        let search_impressions = row
            .get("search_impressions")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid search_impressions"))? as i32;

        let search_clicks = row
            .get("search_clicks")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid search_clicks"))? as i32;

        let avg_search_position = row.get("avg_search_position").and_then(serde_json::Value::as_f64);

        let views_last_7_days = row
            .get("views_last_7_days")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid views_last_7_days"))? as i32;

        let views_last_30_days = row
            .get("views_last_30_days")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid views_last_30_days"))? as i32;

        let trend_direction = row
            .get("trend_direction")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing or invalid trend_direction"))?
            .to_string();

        let created_at = row
            .get("created_at")
            .and_then(parse_database_datetime)
            .ok_or_else(|| anyhow!("Invalid created_at"))?;

        let updated_at = row
            .get("updated_at")
            .and_then(parse_database_datetime)
            .ok_or_else(|| anyhow!("Invalid updated_at"))?;

        Ok(Self {
            id,
            content_id,
            total_views,
            unique_visitors,
            avg_time_on_page_seconds,
            shares_total,
            shares_linkedin,
            shares_twitter,
            comments_count,
            search_impressions,
            search_clicks,
            avg_search_position,
            views_last_7_days,
            views_last_30_days,
            trend_direction,
            created_at,
            updated_at,
        })
    }
}

/// Analytics aggregate (hourly, daily, weekly rollups)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsAggregate {
    pub id: String,
    pub content_id: String,
    pub aggregate_type: Option<String>, // 'hourly', 'daily', 'weekly'
    pub aggregate_date: DateTime<Utc>,
    pub views: i32,
    pub unique_visitors: i32,
    pub avg_engagement: Option<f64>,
    pub created_at: DateTime<Utc>,
}

impl AnalyticsAggregate {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        use systemprompt_core_database::parse_database_datetime;

        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing id"))?
            .to_string();

        let content_id = row
            .get("content_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing content_id"))?
            .to_string();

        let aggregate_type = row
            .get("aggregate_type")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let aggregate_date = row
            .get("aggregate_date")
            .and_then(parse_database_datetime)
            .ok_or_else(|| anyhow!("Invalid aggregate_date"))?;

        let views = row
            .get("views")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid views"))? as i32;

        let unique_visitors = row
            .get("unique_visitors")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid unique_visitors"))? as i32;

        let avg_engagement = row.get("avg_engagement").and_then(serde_json::Value::as_f64);

        let created_at = row
            .get("created_at")
            .and_then(parse_database_datetime)
            .ok_or_else(|| anyhow!("Invalid created_at"))?;

        Ok(Self {
            id,
            content_id,
            aggregate_type,
            aggregate_date,
            views,
            unique_visitors,
            avg_engagement,
            created_at,
        })
    }
}

/// Trending content ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingContent {
    pub id: String,
    pub content_id: String,
    pub trend_score: f64,
    pub trend_rank: Option<i32>,
    pub views_change_percent: Option<f64>,
    pub period: String, // 'daily', 'weekly', 'monthly'
    pub created_at: DateTime<Utc>,
}

impl TrendingContent {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        use systemprompt_core_database::parse_database_datetime;

        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing id"))?
            .to_string();

        let content_id = row
            .get("content_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing content_id"))?
            .to_string();

        let trend_score = row
            .get("trend_score")
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| anyhow!("Missing trend_score"))?;

        let trend_rank = row
            .get("trend_rank")
            .and_then(serde_json::Value::as_i64)
            .map(|v| v as i32);

        let views_change_percent = row.get("views_change_percent").and_then(serde_json::Value::as_f64);

        let period = row
            .get("period")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing period"))?
            .to_string();

        let created_at = row
            .get("created_at")
            .and_then(parse_database_datetime)
            .ok_or_else(|| anyhow!("Invalid created_at"))?;

        Ok(Self {
            id,
            content_id,
            trend_score,
            trend_rank,
            views_change_percent,
            period,
            created_at,
        })
    }
}
