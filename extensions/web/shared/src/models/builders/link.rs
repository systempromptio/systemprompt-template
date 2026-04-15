use chrono::{DateTime, Utc};
use systemprompt::identifiers::{CampaignId, ContentId};

#[derive(Debug, Clone)]
pub struct CreateLinkParams {
    pub short_code: String,
    pub target_url: String,
    pub link_type: &'static str,
    pub source_content_id: Option<ContentId>,
    pub source_page: Option<String>,
    pub campaign_id: Option<CampaignId>,
    pub campaign_name: Option<String>,
    pub utm_params: Option<String>,
    pub link_text: Option<String>,
    pub link_position: Option<String>,
    pub destination_type: Option<String>,
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
}

impl CreateLinkParams {
    #[must_use]
    pub const fn new(short_code: String, target_url: String, link_type: &'static str) -> Self {
        Self {
            short_code,
            target_url,
            link_type,
            source_content_id: None,
            source_page: None,
            campaign_id: None,
            campaign_name: None,
            utm_params: None,
            link_text: None,
            link_position: None,
            destination_type: None,
            is_active: true,
            expires_at: None,
        }
    }

    #[must_use]
    pub fn with_source_content_id(mut self, source_content_id: Option<ContentId>) -> Self {
        self.source_content_id = source_content_id;
        self
    }

    #[must_use]
    pub fn with_source_page(mut self, source_page: Option<String>) -> Self {
        self.source_page = source_page;
        self
    }

    #[must_use]
    pub fn with_campaign_id(mut self, campaign_id: Option<CampaignId>) -> Self {
        self.campaign_id = campaign_id;
        self
    }

    #[must_use]
    pub fn with_campaign_name(mut self, campaign_name: Option<String>) -> Self {
        self.campaign_name = campaign_name;
        self
    }

    #[must_use]
    pub fn with_utm_params(mut self, utm_params: Option<String>) -> Self {
        self.utm_params = utm_params;
        self
    }

    #[must_use]
    pub fn with_link_text(mut self, link_text: Option<String>) -> Self {
        self.link_text = link_text;
        self
    }

    #[must_use]
    pub fn with_link_position(mut self, link_position: Option<String>) -> Self {
        self.link_position = link_position;
        self
    }

    #[must_use]
    pub fn with_destination_type(mut self, destination_type: Option<String>) -> Self {
        self.destination_type = destination_type;
        self
    }

    #[must_use]
    pub const fn with_is_active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }

    #[must_use]
    pub const fn with_expires_at(mut self, expires_at: Option<DateTime<Utc>>) -> Self {
        self.expires_at = expires_at;
        self
    }
}
