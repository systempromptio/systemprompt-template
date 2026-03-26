use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::{LinkId, SessionId};

use crate::error::BlogError;
use crate::models::{CampaignLink, CreateLinkParams, LinkClick, LinkPerformance, TrackClickParams};
use crate::repository::{LinkAnalyticsRepository, LinkRepository};

#[derive(Debug, Clone)]
pub struct LinkService {
    link_repo: LinkRepository,
    analytics_repo: LinkAnalyticsRepository,
}

impl LinkService {
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            link_repo: LinkRepository::new(pool.clone()),
            analytics_repo: LinkAnalyticsRepository::new(pool),
        }
    }

    pub async fn create(&self, params: &CreateLinkParams) -> Result<CampaignLink, BlogError> {
        self.link_repo
            .create_link(params)
            .await
            .map_err(BlogError::from)
    }

    pub async fn get_by_short_code(
        &self,
        short_code: &str,
    ) -> Result<Option<CampaignLink>, BlogError> {
        self.link_repo
            .get_link_by_short_code(short_code)
            .await
            .map_err(BlogError::from)
    }

    pub async fn get_performance(
        &self,
        link_id: &str,
    ) -> Result<Option<LinkPerformance>, BlogError> {
        let link_id = LinkId::new(link_id.to_string());
        self.analytics_repo
            .get_link_performance(&link_id)
            .await
            .map_err(BlogError::from)
    }

    pub async fn get_clicks(&self, link_id: &str, limit: i64) -> Result<Vec<LinkClick>, BlogError> {
        let link_id = LinkId::new(link_id.to_string());
        self.analytics_repo
            .get_clicks_by_link(&link_id, limit, 0)
            .await
            .map_err(BlogError::from)
    }

    pub async fn process_redirect(
        &self,
        short_code: &str,
        session_id: &str,
        user_agent: Option<&str>,
        ip_address: Option<&str>,
    ) -> Result<String, BlogError> {
        let link = self
            .link_repo
            .get_link_by_short_code(short_code)
            .await
            .map_err(BlogError::from)?
            .ok_or_else(|| BlogError::LinkNotFound(short_code.to_string()))?;

        let session_id = SessionId::new(session_id.to_string());
        let params = TrackClickParams::new(link.id.clone(), session_id)
            .with_user_agent(user_agent.map(String::from))
            .with_ip_address(ip_address.map(String::from));

        self.analytics_repo
            .track_click(&params)
            .await
            .map_err(BlogError::from)?;

        Ok(link.get_full_url())
    }
}
