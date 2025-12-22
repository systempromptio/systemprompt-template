//! Link repository - database access layer for campaign links.

use crate::models::{
    CampaignLink, CampaignPerformance, ContentJourneyNode, CreateLinkParams, LinkClick,
    LinkPerformance, RecordClickParams,
};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::{
    CampaignId, ContentId, ContextId, LinkClickId, LinkId, SessionId, TaskId, UserId,
};

/// Repository for link operations.
#[derive(Debug, Clone)]
pub struct LinkRepository {
    pool: Arc<PgPool>,
}

impl LinkRepository {
    /// Create a new link repository.
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Create a new campaign link.
    pub async fn create_link(
        &self,
        params: &CreateLinkParams,
    ) -> Result<CampaignLink, sqlx::Error> {
        let id = LinkId::generate();
        let now = Utc::now();
        sqlx::query_as!(
            CampaignLink,
            r#"
            INSERT INTO campaign_links (
                id, short_code, target_url, link_type, source_content_id, source_page,
                campaign_id, campaign_name, utm_params, link_text, link_position,
                destination_type, is_active, expires_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $15)
            RETURNING id as "id: LinkId", short_code, target_url, link_type,
                      campaign_id as "campaign_id: CampaignId", campaign_name,
                      source_content_id as "source_content_id: ContentId", source_page,
                      utm_params, link_text, link_position, destination_type,
                      click_count, unique_click_count, conversion_count,
                      is_active, expires_at, created_at, updated_at
            "#,
            id.as_str(),
            params.short_code,
            params.target_url,
            params.link_type,
            params.source_content_id.as_ref().map(ContentId::as_str),
            params.source_page,
            params.campaign_id.as_ref().map(CampaignId::as_str),
            params.campaign_name,
            params.utm_params,
            params.link_text,
            params.link_position,
            params.destination_type,
            params.is_active,
            params.expires_at,
            now
        )
        .fetch_one(&*self.pool)
        .await
    }

    /// Get a link by short code.
    pub async fn get_link_by_short_code(
        &self,
        short_code: &str,
    ) -> Result<Option<CampaignLink>, sqlx::Error> {
        sqlx::query_as!(
            CampaignLink,
            r#"
            SELECT id as "id: LinkId", short_code, target_url, link_type,
                   campaign_id as "campaign_id: CampaignId", campaign_name,
                   source_content_id as "source_content_id: ContentId", source_page,
                   utm_params, link_text, link_position, destination_type,
                   click_count, unique_click_count, conversion_count,
                   is_active, expires_at, created_at, updated_at
            FROM campaign_links
            WHERE short_code = $1 AND is_active = true
            "#,
            short_code
        )
        .fetch_optional(&*self.pool)
        .await
    }

    /// List links by campaign.
    pub async fn list_links_by_campaign(
        &self,
        campaign_id: &CampaignId,
    ) -> Result<Vec<CampaignLink>, sqlx::Error> {
        sqlx::query_as!(
            CampaignLink,
            r#"
            SELECT id as "id: LinkId", short_code, target_url, link_type,
                   campaign_id as "campaign_id: CampaignId", campaign_name,
                   source_content_id as "source_content_id: ContentId", source_page,
                   utm_params, link_text, link_position, destination_type,
                   click_count, unique_click_count, conversion_count,
                   is_active, expires_at, created_at, updated_at
            FROM campaign_links
            WHERE campaign_id = $1
            ORDER BY created_at DESC
            "#,
            campaign_id.as_str()
        )
        .fetch_all(&*self.pool)
        .await
    }

    /// List links by source content.
    pub async fn list_links_by_source_content(
        &self,
        content_id: &ContentId,
    ) -> Result<Vec<CampaignLink>, sqlx::Error> {
        sqlx::query_as!(
            CampaignLink,
            r#"
            SELECT id as "id: LinkId", short_code, target_url, link_type,
                   campaign_id as "campaign_id: CampaignId", campaign_name,
                   source_content_id as "source_content_id: ContentId", source_page,
                   utm_params, link_text, link_position, destination_type,
                   click_count, unique_click_count, conversion_count,
                   is_active, expires_at, created_at, updated_at
            FROM campaign_links
            WHERE source_content_id = $1
            ORDER BY created_at DESC
            "#,
            content_id.as_str()
        )
        .fetch_all(&*self.pool)
        .await
    }
}

/// Repository for link analytics.
#[derive(Debug, Clone)]
pub struct LinkAnalyticsRepository {
    pool: Arc<PgPool>,
}

impl LinkAnalyticsRepository {
    /// Create a new link analytics repository.
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Track a click on a link.
    pub async fn track_click(
        &self,
        link_id: &LinkId,
        session_id: &SessionId,
        user_id: Option<&UserId>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        referrer_page: Option<&str>,
        referrer_url: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.as_ref().begin().await?;

        let id = LinkClickId::generate();
        sqlx::query!(
            r#"
            INSERT INTO link_clicks (id, link_id, session_id, user_id, ip_address,
                                     user_agent, referrer_page, referrer_url, clicked_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            id.as_str(),
            link_id.as_str(),
            session_id.as_str(),
            user_id.map(UserId::as_str),
            ip_address,
            user_agent,
            referrer_page,
            referrer_url,
            Utc::now()
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE campaign_links SET click_count = click_count + 1 WHERE id = $1",
            link_id.as_str()
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Get link performance metrics.
    pub async fn get_link_performance(
        &self,
        link_id: &LinkId,
    ) -> Result<Option<LinkPerformance>, sqlx::Error> {
        sqlx::query_as!(
            LinkPerformance,
            r#"
            SELECT
                l.id as "link_id: LinkId",
                COALESCE(l.click_count, 0)::bigint as "click_count!",
                COALESCE(l.unique_click_count, 0)::bigint as "unique_click_count!",
                COALESCE(l.conversion_count, 0)::bigint as "conversion_count!",
                CASE
                    WHEN COALESCE(l.click_count, 0) > 0 THEN
                        COALESCE(l.conversion_count, 0)::float / l.click_count
                    ELSE 0.0
                END as conversion_rate
            FROM campaign_links l
            WHERE l.id = $1
            "#,
            link_id.as_str()
        )
        .fetch_optional(&*self.pool)
        .await
    }

    /// Check if a session has clicked a link.
    pub async fn check_session_clicked_link(
        &self,
        link_id: &LinkId,
        session_id: &SessionId,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"SELECT COALESCE(COUNT(*), 0)::bigint as "count!" FROM link_clicks WHERE link_id = $1 AND session_id = $2"#,
            link_id.as_str(),
            session_id.as_str()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result.count > 0)
    }

    /// Increment link click counts.
    pub async fn increment_link_clicks(
        &self,
        link_id: &LinkId,
        is_first_click: bool,
    ) -> Result<(), sqlx::Error> {
        if is_first_click {
            sqlx::query!(
                "UPDATE campaign_links SET click_count = click_count + 1, unique_click_count = \
                 unique_click_count + 1 WHERE id = $1",
                link_id.as_str()
            )
            .execute(&*self.pool)
            .await?;
        } else {
            sqlx::query!(
                "UPDATE campaign_links SET click_count = click_count + 1 WHERE id = $1",
                link_id.as_str()
            )
            .execute(&*self.pool)
            .await?;
        }
        Ok(())
    }

    /// Get clicks for a link.
    pub async fn get_clicks_by_link(
        &self,
        link_id: &LinkId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<LinkClick>, sqlx::Error> {
        sqlx::query_as!(
            LinkClick,
            r#"
            SELECT id as "id: LinkClickId", link_id as "link_id: LinkId",
                   session_id as "session_id: SessionId", user_id as "user_id: UserId",
                   context_id as "context_id: ContextId", task_id as "task_id: TaskId",
                   referrer_page, referrer_url, clicked_at, user_agent, ip_address,
                   device_type, country, is_first_click, is_conversion, conversion_at,
                   time_on_page_seconds, scroll_depth_percent
            FROM link_clicks
            WHERE link_id = $1
            ORDER BY clicked_at DESC
            LIMIT $2 OFFSET $3
            "#,
            link_id.as_str(),
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
    }

    /// Get content journey map.
    pub async fn get_content_journey_map(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ContentJourneyNode>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT source_content_id, target_url, COALESCE(click_count, 0) as "click_count!"
            FROM campaign_links
            WHERE source_content_id IS NOT NULL AND click_count > 0
            ORDER BY click_count DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|r| {
                Some(ContentJourneyNode {
                    source_content_id: ContentId::new(r.source_content_id?),
                    target_url: r.target_url,
                    click_count: r.click_count,
                })
            })
            .collect())
    }

    /// Get campaign performance.
    pub async fn get_campaign_performance(
        &self,
        campaign_id: &CampaignId,
    ) -> Result<Option<CampaignPerformance>, sqlx::Error> {
        sqlx::query_as!(
            CampaignPerformance,
            r#"
            SELECT
                campaign_id as "campaign_id!: CampaignId",
                COALESCE(SUM(click_count), 0)::bigint as "total_clicks!",
                COUNT(*)::bigint as "link_count!",
                COUNT(DISTINCT source_content_id) as unique_visitors,
                COALESCE(SUM(conversion_count), 0)::bigint as conversion_count
            FROM campaign_links
            WHERE campaign_id = $1
            GROUP BY campaign_id
            "#,
            campaign_id.as_str()
        )
        .fetch_optional(&*self.pool)
        .await
    }

    /// Record a click event.
    pub async fn record_click(&self, params: &RecordClickParams) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO link_clicks (
                id, link_id, session_id, user_id, context_id, task_id,
                referrer_page, referrer_url, clicked_at, user_agent, ip_address,
                device_type, country, is_first_click, is_conversion
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            params.click_id.as_str(),
            params.link_id.as_str(),
            params.session_id.as_str(),
            params.user_id.as_ref().map(UserId::as_str),
            params.context_id.as_ref().map(ContextId::as_str),
            params.task_id.as_ref().map(TaskId::as_str),
            params.referrer_page,
            params.referrer_url,
            params.clicked_at,
            params.user_agent,
            params.ip_address,
            params.device_type,
            params.country,
            params.is_first_click,
            params.is_conversion
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }
}
