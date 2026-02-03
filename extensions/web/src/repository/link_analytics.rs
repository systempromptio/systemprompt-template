use crate::models::{
    CampaignPerformance, ContentJourneyNode, LinkClick, LinkPerformance, RecordClickParams,
    TrackClickParams,
};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::{
    CampaignId, ContentId, ContextId, LinkClickId, LinkId, SessionId, TaskId, UserId,
};

#[derive(Debug, Clone)]
pub struct LinkAnalyticsRepository {
    pool: Arc<PgPool>,
}

impl LinkAnalyticsRepository {
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn track_click(&self, params: &TrackClickParams) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.as_ref().begin().await?;

        let id = LinkClickId::generate();
        sqlx::query!(
            r#"
            INSERT INTO link_clicks (id, link_id, session_id, user_id, ip_address,
                                     user_agent, referrer_page, referrer_url, clicked_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            id.as_str(),
            params.link_id.as_str(),
            params.session_id.as_str(),
            params.user_id.as_ref().map(UserId::as_str),
            params.ip_address.as_deref(),
            params.user_agent.as_deref(),
            params.referrer_page.as_deref(),
            params.referrer_url.as_deref(),
            Utc::now()
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE campaign_links SET click_count = click_count + 1 WHERE id = $1",
            params.link_id.as_str()
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

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

    pub async fn get_content_journey(
        &self,
        content_id: &ContentId,
    ) -> Result<Vec<ContentJourneyNode>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT
                cl.source_content_id,
                cl.target_url,
                COUNT(lc.id)::integer as "click_count!"
            FROM campaign_links cl
            LEFT JOIN link_clicks lc ON cl.id = lc.link_id
            WHERE cl.source_content_id = $1
            GROUP BY cl.source_content_id, cl.target_url
            ORDER BY 3 DESC
            "#,
            content_id.as_str()
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
