use chrono::{DateTime, Utc};
use systemprompt::identifiers::{ContextId, LinkClickId, LinkId, SessionId, TaskId, UserId};

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
