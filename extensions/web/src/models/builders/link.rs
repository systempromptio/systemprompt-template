use chrono::{DateTime, Utc};
use systemprompt::identifiers::{
    CampaignId, ContentId, ContextId, LinkClickId, LinkId, SessionId, TaskId, UserId,
};

#[derive(Debug, Clone)]
pub struct CreateLinkParams {
    pub short_code: String,
    pub target_url: String,
    pub link_type: String,
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
    pub const fn new(short_code: String, target_url: String, link_type: String) -> Self {
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

#[derive(Debug, Clone)]
pub struct RecordClickParams {
    pub click_id: LinkClickId,
    pub link_id: LinkId,
    pub session_id: SessionId,
    pub user_id: Option<UserId>,
    pub context_id: Option<ContextId>,
    pub task_id: Option<TaskId>,
    pub referrer_page: Option<String>,
    pub referrer_url: Option<String>,
    pub clicked_at: DateTime<Utc>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub device_type: Option<String>,
    pub country: Option<String>,
    pub is_first_click: bool,
    pub is_conversion: bool,
}

impl RecordClickParams {
    #[must_use]
    pub const fn new(
        click_id: LinkClickId,
        link_id: LinkId,
        session_id: SessionId,
        clicked_at: DateTime<Utc>,
    ) -> Self {
        Self {
            click_id,
            link_id,
            session_id,
            user_id: None,
            context_id: None,
            task_id: None,
            referrer_page: None,
            referrer_url: None,
            clicked_at,
            user_agent: None,
            ip_address: None,
            device_type: None,
            country: None,
            is_first_click: false,
            is_conversion: false,
        }
    }

    #[must_use]
    pub fn with_user_id(mut self, user_id: Option<UserId>) -> Self {
        self.user_id = user_id;
        self
    }

    #[must_use]
    pub fn with_context_id(mut self, context_id: Option<ContextId>) -> Self {
        self.context_id = context_id;
        self
    }

    #[must_use]
    pub fn with_task_id(mut self, task_id: Option<TaskId>) -> Self {
        self.task_id = task_id;
        self
    }

    #[must_use]
    pub fn with_referrer_page(mut self, referrer_page: Option<String>) -> Self {
        self.referrer_page = referrer_page;
        self
    }

    #[must_use]
    pub fn with_referrer_url(mut self, referrer_url: Option<String>) -> Self {
        self.referrer_url = referrer_url;
        self
    }

    #[must_use]
    pub fn with_user_agent(mut self, user_agent: Option<String>) -> Self {
        self.user_agent = user_agent;
        self
    }

    #[must_use]
    pub fn with_ip_address(mut self, ip_address: Option<String>) -> Self {
        self.ip_address = ip_address;
        self
    }

    #[must_use]
    pub fn with_device_type(mut self, device_type: Option<String>) -> Self {
        self.device_type = device_type;
        self
    }

    #[must_use]
    pub fn with_country(mut self, country: Option<String>) -> Self {
        self.country = country;
        self
    }

    #[must_use]
    pub const fn with_is_first_click(mut self, is_first_click: bool) -> Self {
        self.is_first_click = is_first_click;
        self
    }

    #[must_use]
    pub const fn with_is_conversion(mut self, is_conversion: bool) -> Self {
        self.is_conversion = is_conversion;
        self
    }
}

#[derive(Debug, Clone)]
pub struct TrackClickParams {
    pub link_id: LinkId,
    pub session_id: SessionId,
    pub user_id: Option<UserId>,
    pub context_id: Option<ContextId>,
    pub task_id: Option<TaskId>,
    pub referrer_page: Option<String>,
    pub referrer_url: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub device_type: Option<String>,
    pub country: Option<String>,
}

impl TrackClickParams {
    #[must_use]
    pub const fn new(link_id: LinkId, session_id: SessionId) -> Self {
        Self {
            link_id,
            session_id,
            user_id: None,
            context_id: None,
            task_id: None,
            referrer_page: None,
            referrer_url: None,
            user_agent: None,
            ip_address: None,
            device_type: None,
            country: None,
        }
    }

    #[must_use]
    pub fn with_user_id(mut self, user_id: Option<UserId>) -> Self {
        self.user_id = user_id;
        self
    }

    #[must_use]
    pub fn with_context_id(mut self, context_id: Option<ContextId>) -> Self {
        self.context_id = context_id;
        self
    }

    #[must_use]
    pub fn with_task_id(mut self, task_id: Option<TaskId>) -> Self {
        self.task_id = task_id;
        self
    }

    #[must_use]
    pub fn with_referrer_page(mut self, referrer_page: Option<String>) -> Self {
        self.referrer_page = referrer_page;
        self
    }

    #[must_use]
    pub fn with_referrer_url(mut self, referrer_url: Option<String>) -> Self {
        self.referrer_url = referrer_url;
        self
    }

    #[must_use]
    pub fn with_user_agent(mut self, user_agent: Option<String>) -> Self {
        self.user_agent = user_agent;
        self
    }

    #[must_use]
    pub fn with_ip_address(mut self, ip_address: Option<String>) -> Self {
        self.ip_address = ip_address;
        self
    }

    #[must_use]
    pub fn with_device_type(mut self, device_type: Option<String>) -> Self {
        self.device_type = device_type;
        self
    }

    #[must_use]
    pub fn with_country(mut self, country: Option<String>) -> Self {
        self.country = country;
        self
    }
}
