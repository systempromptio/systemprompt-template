use crate::models::{CampaignPerformance, ContentJourneyNode, LinkClick, LinkPerformance};
use crate::repository::LinkRepository;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use systemprompt_core_database::DatabaseProvider;

#[derive(Debug)]
pub struct LinkAnalyticsService {
    link_repo: LinkRepository,
}

impl LinkAnalyticsService {
    pub fn new(db: Arc<dyn DatabaseProvider>) -> Self {
        Self {
            link_repo: LinkRepository::new(db),
        }
    }

    pub async fn track_click(
        &self,
        link_id: &str,
        session_id: &str,
        user_id: Option<String>,
        context_id: Option<String>,
        task_id: Option<String>,
        referrer_page: Option<String>,
        referrer_url: Option<String>,
        user_agent: Option<String>,
        ip_address: Option<String>,
        device_type: Option<String>,
        country: Option<String>,
    ) -> Result<LinkClick> {
        let is_first_click = !self
            .link_repo
            .check_session_clicked_link(link_id, session_id)
            .await?;

        let click_id = uuid::Uuid::new_v4().to_string();
        let clicked_at = Utc::now();

        self.link_repo
            .record_click(
                &click_id,
                link_id,
                session_id,
                user_id.as_deref(),
                context_id.as_deref(),
                task_id.as_deref(),
                referrer_page.as_deref(),
                referrer_url.as_deref(),
                clicked_at,
                user_agent.as_deref(),
                ip_address.as_deref(),
                device_type.as_deref(),
                country.as_deref(),
                is_first_click,
                false,
            )
            .await?;

        self.link_repo
            .increment_link_clicks(link_id, is_first_click)
            .await?;

        Ok(LinkClick {
            id: click_id,
            link_id: link_id.to_string(),
            session_id: session_id.to_string(),
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
            is_conversion: false,
            conversion_at: None,
            time_on_page_seconds: None,
            scroll_depth_percent: None,
        })
    }

    pub async fn get_link_performance(&self, link_id: &str) -> Result<Option<LinkPerformance>> {
        self.link_repo.get_link_performance(link_id).await
    }

    pub async fn get_campaign_performance(
        &self,
        campaign_id: &str,
    ) -> Result<Option<CampaignPerformance>> {
        self.link_repo.get_campaign_performance(campaign_id).await
    }

    pub async fn get_content_journey_map(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ContentJourneyNode>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);
        self.link_repo.get_content_journey_map(limit, offset).await
    }

    pub async fn get_link_clicks(
        &self,
        link_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<LinkClick>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        self.link_repo
            .get_clicks_by_link(link_id, limit, offset)
            .await
    }

    pub async fn get_links_by_campaign(
        &self,
        campaign_id: &str,
    ) -> Result<Vec<crate::models::CampaignLink>> {
        self.link_repo.list_links_by_campaign(campaign_id).await
    }

    pub async fn get_links_by_source_content(
        &self,
        source_content_id: &str,
    ) -> Result<Vec<crate::models::CampaignLink>> {
        self.link_repo
            .list_links_by_source_content(source_content_id)
            .await
    }
}
