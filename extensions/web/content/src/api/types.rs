//! Request and response bodies for the content API.

use serde::{Deserialize, Serialize};
use systemprompt::identifiers::{CampaignId, CategoryId, ContentId, LinkId, SessionId};

use systemprompt_web_shared::models::UtmParams;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateLinkRequest {
    pub target_url: String,
    pub campaign_name: Option<String>,
    pub utm_params: Option<UtmParams>,
    pub source_content_id: Option<ContentId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateLinkResponse {
    pub id: LinkId,
    pub short_code: String,
    pub short_url: String,
    pub target_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentJourneyQuery {
    pub content_id: ContentId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordClickRequest {
    pub link_id: LinkId,
    pub session_id: Option<SessionId>,
    pub referrer_page: Option<String>,
    pub referrer_url: Option<String>,
    pub user_agent: Option<String>,
    pub device_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListLinksQuery {
    pub campaign_id: Option<CampaignId>,
    pub content_id: Option<ContentId>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub category_id: Option<CategoryId>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl ErrorResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            error: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct OkResponse {
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListLinksResponse<T> {
    pub links: Vec<T>,
    pub total: usize,
}

impl<T> ListLinksResponse<T> {
    pub const fn new(links: Vec<T>) -> Self {
        Self {
            total: links.len(),
            links,
        }
    }
}
