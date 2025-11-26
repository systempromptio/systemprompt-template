use crate::models::{
    CampaignLink, CampaignPerformance, ContentJourneyNode, LinkClick, LinkPerformance,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use systemprompt_core_database::DatabaseProvider;
use systemprompt_core_database::DatabaseQueryEnum;

#[derive(Debug)]
pub struct LinkRepository {
    db: Arc<dyn DatabaseProvider>,
}

impl LinkRepository {
    pub fn new(db: Arc<dyn DatabaseProvider>) -> Self {
        Self { db }
    }

    pub async fn create_link(
        &self,
        id: &str,
        short_code: &str,
        target_url: &str,
        link_type: &str,
        campaign_id: Option<&str>,
        campaign_name: Option<&str>,
        source_content_id: Option<&str>,
        source_page: Option<&str>,
        utm_params: Option<&str>,
        link_text: Option<&str>,
        link_position: Option<&str>,
        destination_type: &str,
        is_active: bool,
        expires_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<String> {
        let query = DatabaseQueryEnum::CreateLink.get(self.db.as_ref());

        let _row = self
            .db
            .fetch_one(
                &query,
                &[
                    &id,
                    &short_code,
                    &target_url,
                    &link_type,
                    &campaign_id,
                    &campaign_name,
                    &source_content_id,
                    &source_page,
                    &utm_params,
                    &link_text,
                    &link_position,
                    &destination_type,
                    &is_active,
                    &expires_at,
                    &created_at,
                    &updated_at,
                ],
            )
            .await
            .context(format!(
                "Failed to create link with short_code: {short_code}"
            ))?;

        Ok(id.to_string())
    }

    pub async fn get_link_by_id(&self, id: &str) -> Result<Option<CampaignLink>> {
        let query = DatabaseQueryEnum::GetLinkById.get(self.db.as_ref());

        let row = self
            .db
            .fetch_optional(&query, &[&id])
            .await
            .context(format!("Failed to get link by id: {id}"))?;

        row.as_ref()
            .map(CampaignLink::from_json_row)
            .transpose()
    }

    pub async fn get_link_by_short_code(&self, short_code: &str) -> Result<Option<CampaignLink>> {
        let query = DatabaseQueryEnum::GetLinkByShortCode.get(self.db.as_ref());

        let row = self
            .db
            .fetch_optional(&query, &[&short_code])
            .await
            .context(format!("Failed to get link by short_code: {short_code}"))?;

        row.as_ref()
            .map(CampaignLink::from_json_row)
            .transpose()
    }

    pub async fn list_links_by_campaign(&self, campaign_id: &str) -> Result<Vec<CampaignLink>> {
        let query = DatabaseQueryEnum::ListLinksByCampaign.get(self.db.as_ref());

        let rows = self
            .db
            .fetch_all(&query, &[&campaign_id])
            .await
            .context(format!(
                "Failed to list links for campaign: {campaign_id}"
            ))?;

        rows.iter()
            .map(CampaignLink::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn list_links_by_source_content(
        &self,
        source_content_id: &str,
    ) -> Result<Vec<CampaignLink>> {
        let query = DatabaseQueryEnum::ListLinksBySourceContent.get(self.db.as_ref());

        let rows = self
            .db
            .fetch_all(&query, &[&source_content_id])
            .await
            .context(format!(
                "Failed to list links for source content: {source_content_id}"
            ))?;

        rows.iter()
            .map(CampaignLink::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn increment_link_clicks(&self, link_id: &str, is_unique: bool) -> Result<()> {
        let query = DatabaseQueryEnum::IncrementLinkClicks.get(self.db.as_ref());

        self.db
            .execute(&query, &[&link_id, &is_unique])
            .await
            .context(format!("Failed to increment clicks for link: {link_id}"))?;

        Ok(())
    }

    pub async fn record_click(
        &self,
        id: &str,
        link_id: &str,
        session_id: &str,
        user_id: Option<&str>,
        context_id: Option<&str>,
        task_id: Option<&str>,
        referrer_page: Option<&str>,
        referrer_url: Option<&str>,
        clicked_at: DateTime<Utc>,
        user_agent: Option<&str>,
        ip_address: Option<&str>,
        device_type: Option<&str>,
        country: Option<&str>,
        is_first_click: bool,
        is_conversion: bool,
    ) -> Result<String> {
        let query = DatabaseQueryEnum::RecordClick.get(self.db.as_ref());

        let _row = self
            .db
            .fetch_one(
                &query,
                &[
                    &id,
                    &link_id,
                    &session_id,
                    &user_id,
                    &context_id,
                    &task_id,
                    &referrer_page,
                    &referrer_url,
                    &clicked_at,
                    &user_agent,
                    &ip_address,
                    &device_type,
                    &country,
                    &is_first_click,
                    &is_conversion,
                ],
            )
            .await
            .context(format!("Failed to record click for link: {link_id}"))?;

        Ok(id.to_string())
    }

    pub async fn check_session_clicked_link(
        &self,
        link_id: &str,
        session_id: &str,
    ) -> Result<bool> {
        let query = DatabaseQueryEnum::CheckSessionClickedLink.get(self.db.as_ref());

        let row = self
            .db
            .fetch_optional(&query, &[&link_id, &session_id])
            .await
            .context(format!(
                "Failed to check session click for link: {link_id}"
            ))?;

        if let Some(row) = row {
            let count = row.get("click_count").and_then(serde_json::Value::as_i64).unwrap_or(0);
            Ok(count > 0)
        } else {
            Ok(false)
        }
    }

    pub async fn get_clicks_by_link(
        &self,
        link_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<LinkClick>> {
        let query = DatabaseQueryEnum::GetClicksByLink.get(self.db.as_ref());

        let rows = self
            .db
            .fetch_all(&query, &[&link_id, &limit, &offset])
            .await
            .context(format!("Failed to get clicks for link: {link_id}"))?;

        rows.iter()
            .map(LinkClick::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn get_link_performance(&self, link_id: &str) -> Result<Option<LinkPerformance>> {
        let query = DatabaseQueryEnum::GetLinkPerformance.get(self.db.as_ref());

        let row = self
            .db
            .fetch_optional(&query, &[&link_id])
            .await
            .context(format!("Failed to get performance for link: {link_id}"))?;

        row.as_ref()
            .map(LinkPerformance::from_json_row)
            .transpose()
    }

    pub async fn get_campaign_performance(
        &self,
        campaign_id: &str,
    ) -> Result<Option<CampaignPerformance>> {
        let query = DatabaseQueryEnum::GetCampaignPerformance.get(self.db.as_ref());

        let row = self
            .db
            .fetch_optional(&query, &[&campaign_id])
            .await
            .context(format!(
                "Failed to get performance for campaign: {campaign_id}"
            ))?;

        row.as_ref()
            .map(CampaignPerformance::from_json_row)
            .transpose()
    }

    pub async fn get_content_journey_map(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ContentJourneyNode>> {
        let query = DatabaseQueryEnum::GetContentJourneyMap.get(self.db.as_ref());

        let rows = self
            .db
            .fetch_all(&query, &[&limit, &offset])
            .await
            .context("Failed to get content journey map")?;

        rows.iter()
            .map(ContentJourneyNode::from_json_row)
            .collect::<Result<Vec<_>>>()
    }
}
