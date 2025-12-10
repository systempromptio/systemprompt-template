use crate::models::{
    CampaignLink, CampaignPerformance, ContentJourneyNode, LinkClick, LinkPerformance,
};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use uuid::Uuid;

#[derive(Debug)]
pub struct LinkRepository {
    pool: Arc<PgPool>,
}

impl LinkRepository {
    pub fn new(db: DbPool) -> Self {
        let pool = db.pool_arc().expect("Database must be PostgreSQL");
        Self { pool }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_link(
        &self,
        short_code: &str,
        target_url: &str,
        link_type: &str,
        source_content_id: Option<&str>,
        source_page: Option<&str>,
        campaign_id: Option<&str>,
        campaign_name: Option<&str>,
        utm_params: Option<&str>,
        link_text: Option<&str>,
        link_position: Option<&str>,
        destination_type: Option<&str>,
        is_active: bool,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<CampaignLink, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
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
            RETURNING id, short_code, target_url, link_type, campaign_id, campaign_name,
                      source_content_id, source_page, utm_params, link_text, link_position,
                      destination_type, click_count, unique_click_count, conversion_count,
                      is_active, expires_at, created_at, updated_at
            "#,
            id,
            short_code,
            target_url,
            link_type,
            source_content_id,
            source_page,
            campaign_id,
            campaign_name,
            utm_params,
            link_text,
            link_position,
            destination_type,
            is_active,
            expires_at,
            now
        )
        .fetch_one(&*self.pool)
        .await
    }

    pub async fn get_link_by_short_code(
        &self,
        short_code: &str,
    ) -> Result<Option<CampaignLink>, sqlx::Error> {
        sqlx::query_as!(
            CampaignLink,
            r#"
            SELECT id, short_code, target_url, link_type, campaign_id, campaign_name,
                   source_content_id, source_page, utm_params, link_text, link_position,
                   destination_type, click_count, unique_click_count, conversion_count,
                   is_active, expires_at, created_at, updated_at
            FROM campaign_links
            WHERE short_code = $1 AND is_active = true
            "#,
            short_code
        )
        .fetch_optional(&*self.pool)
        .await
    }

    pub async fn track_click(
        &self,
        link_id: &str,
        session_id: &str,
        user_id: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        referrer_page: Option<&str>,
        referrer_url: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.as_ref().begin().await?;

        let id = Uuid::new_v4().to_string();
        sqlx::query!(
            r#"
            INSERT INTO link_clicks (id, link_id, session_id, user_id, ip_address,
                                     user_agent, referrer_page, referrer_url, clicked_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            id,
            link_id,
            session_id,
            user_id,
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
            link_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_link_performance(
        &self,
        link_id: &str,
    ) -> Result<Option<LinkPerformance>, sqlx::Error> {
        sqlx::query_as!(
            LinkPerformance,
            r#"
            SELECT
                l.id as link_id,
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
            link_id
        )
        .fetch_optional(&*self.pool)
        .await
    }

    pub async fn check_session_clicked_link(
        &self,
        link_id: &str,
        session_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM link_clicks WHERE link_id = $1 AND session_id = $2",
            link_id,
            session_id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result.count.unwrap_or(0) > 0)
    }

    pub async fn increment_link_clicks(
        &self,
        link_id: &str,
        is_first_click: bool,
    ) -> Result<(), sqlx::Error> {
        if is_first_click {
            sqlx::query!(
                "UPDATE campaign_links SET click_count = click_count + 1, unique_click_count = \
                 unique_click_count + 1 WHERE id = $1",
                link_id
            )
            .execute(&*self.pool)
            .await?;
        } else {
            sqlx::query!(
                "UPDATE campaign_links SET click_count = click_count + 1 WHERE id = $1",
                link_id
            )
            .execute(&*self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn list_links_by_campaign(
        &self,
        campaign_id: &str,
    ) -> Result<Vec<CampaignLink>, sqlx::Error> {
        sqlx::query_as!(
            CampaignLink,
            r#"
            SELECT id, short_code, target_url, link_type, campaign_id, campaign_name,
                   source_content_id, source_page, utm_params, link_text, link_position,
                   destination_type, click_count, unique_click_count, conversion_count,
                   is_active, expires_at, created_at, updated_at
            FROM campaign_links
            WHERE campaign_id = $1
            ORDER BY created_at DESC
            "#,
            campaign_id
        )
        .fetch_all(&*self.pool)
        .await
    }

    pub async fn list_links_by_source_content(
        &self,
        content_id: &str,
    ) -> Result<Vec<CampaignLink>, sqlx::Error> {
        sqlx::query_as!(
            CampaignLink,
            r#"
            SELECT id, short_code, target_url, link_type, campaign_id, campaign_name,
                   source_content_id, source_page, utm_params, link_text, link_position,
                   destination_type, click_count, unique_click_count, conversion_count,
                   is_active, expires_at, created_at, updated_at
            FROM campaign_links
            WHERE source_content_id = $1
            ORDER BY created_at DESC
            "#,
            content_id
        )
        .fetch_all(&*self.pool)
        .await
    }

    pub async fn get_clicks_by_link(
        &self,
        link_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<LinkClick>, sqlx::Error> {
        sqlx::query_as!(
            LinkClick,
            r#"
            SELECT id, link_id, session_id, user_id, context_id, task_id,
                   referrer_page, referrer_url, clicked_at, user_agent, ip_address,
                   device_type, country, is_first_click, is_conversion, conversion_at,
                   time_on_page_seconds, scroll_depth_percent
            FROM link_clicks
            WHERE link_id = $1
            ORDER BY clicked_at DESC
            LIMIT $2 OFFSET $3
            "#,
            link_id,
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
            SELECT source_content_id, target_url, COALESCE(click_count, 0) as click_count
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
                    source_content_id: r.source_content_id?,
                    target_url: r.target_url,
                    click_count: r.click_count.unwrap_or(0),
                })
            })
            .collect())
    }

    pub async fn get_campaign_performance(
        &self,
        campaign_id: &str,
    ) -> Result<Option<CampaignPerformance>, sqlx::Error> {
        sqlx::query_as!(
            CampaignPerformance,
            r#"
            SELECT
                campaign_id as "campaign_id!",
                COALESCE(SUM(click_count), 0)::bigint as "total_clicks!",
                COUNT(*)::bigint as "link_count!",
                COUNT(DISTINCT source_content_id) as unique_visitors,
                COALESCE(SUM(conversion_count), 0)::bigint as conversion_count
            FROM campaign_links
            WHERE campaign_id = $1
            GROUP BY campaign_id
            "#,
            campaign_id
        )
        .fetch_optional(&*self.pool)
        .await
    }

    pub async fn record_click(
        &self,
        click_id: &str,
        link_id: &str,
        session_id: &str,
        user_id: Option<&str>,
        context_id: Option<&str>,
        task_id: Option<&str>,
        referrer_page: Option<&str>,
        referrer_url: Option<&str>,
        clicked_at: chrono::DateTime<Utc>,
        user_agent: Option<&str>,
        ip_address: Option<&str>,
        device_type: Option<&str>,
        country: Option<&str>,
        is_first_click: bool,
        is_conversion: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO link_clicks (
                id, link_id, session_id, user_id, context_id, task_id,
                referrer_page, referrer_url, clicked_at, user_agent, ip_address,
                device_type, country, is_first_click, is_conversion
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            click_id,
            link_id,
            session_id,
            user_id,
            context_id,
            task_id,
            referrer_page,
            referrer_url,
            clicked_at,
            user_agent,
            ip_address,
            device_type,
            country,
            is_first_click,
            is_conversion
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }
}
