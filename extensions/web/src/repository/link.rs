use crate::models::{CampaignLink, CreateLinkParams};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::{CampaignId, ContentId, LinkId};

#[derive(Debug, Clone)]
pub struct LinkRepository {
    pool: Arc<PgPool>,
}

impl LinkRepository {
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

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
