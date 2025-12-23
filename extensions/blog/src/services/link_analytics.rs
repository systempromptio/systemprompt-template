//! Link analytics service - aggregated link performance data.

use sqlx::PgPool;
use std::sync::Arc;

use crate::error::BlogError;
use crate::models::CampaignPerformance;

/// Service for link analytics.
#[derive(Debug, Clone)]
pub struct LinkAnalyticsService {
    pool: Arc<PgPool>,
}

impl LinkAnalyticsService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Get campaign performance.
    pub async fn get_campaign_performance(
        &self,
        campaign_id: &str,
    ) -> Result<Option<CampaignPerformance>, BlogError> {
        let perf = sqlx::query_as::<_, CampaignPerformance>(
            r#"
            SELECT
                campaign_id,
                SUM(COALESCE(click_count, 0))::bigint as total_clicks,
                COUNT(*)::bigint as link_count,
                COUNT(DISTINCT source_content_id)::bigint as unique_visitors,
                SUM(COALESCE(conversion_count, 0))::bigint as conversion_count
            FROM campaign_links
            WHERE campaign_id = $1
            GROUP BY campaign_id
            "#,
        )
        .bind(campaign_id)
        .fetch_optional(&*self.pool)
        .await?;

        Ok(perf)
    }

    /// Get content journey data (how users move between content).
    pub async fn get_content_journey(
        &self,
        content_id: &str,
    ) -> Result<Vec<ContentJourneyData>, BlogError> {
        let journey = sqlx::query_as::<_, ContentJourneyData>(
            r#"
            SELECT
                cl.source_content_id,
                cl.target_url,
                COUNT(lc.id)::integer as click_count
            FROM campaign_links cl
            LEFT JOIN link_clicks lc ON cl.id = lc.link_id
            WHERE cl.source_content_id = $1
            GROUP BY cl.source_content_id, cl.target_url
            ORDER BY click_count DESC
            "#,
        )
        .bind(content_id)
        .fetch_all(&*self.pool)
        .await?;

        Ok(journey)
    }

    /// Get top performing links.
    pub async fn get_top_links(&self, limit: i64) -> Result<Vec<TopLink>, BlogError> {
        let links = sqlx::query_as::<_, TopLink>(
            r#"
            SELECT
                id,
                short_code,
                target_url,
                campaign_name,
                COALESCE(click_count, 0) as click_count,
                COALESCE(conversion_count, 0) as conversion_count
            FROM campaign_links
            WHERE is_active = true
            ORDER BY click_count DESC NULLS LAST
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&*self.pool)
        .await?;

        Ok(links)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct ContentJourneyData {
    pub source_content_id: Option<String>,
    pub target_url: String,
    pub click_count: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct TopLink {
    pub id: String,
    pub short_code: String,
    pub target_url: String,
    pub campaign_name: Option<String>,
    pub click_count: i32,
    pub conversion_count: i32,
}
