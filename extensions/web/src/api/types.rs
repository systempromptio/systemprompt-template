use serde::{Deserialize, Serialize};

use crate::models::UtmParams;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateLinkRequest {
    pub target_url: String,
    pub campaign_name: Option<String>,
    pub utm_params: Option<UtmParams>,
    pub source_content_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateLinkResponse {
    pub id: String,
    pub short_code: String,
    pub short_url: String,
    pub target_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentJourneyQuery {
    pub content_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordClickRequest {
    pub link_id: String,
    pub session_id: Option<String>,
    pub referrer_page: Option<String>,
    pub referrer_url: Option<String>,
    pub user_agent: Option<String>,
    pub device_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListLinksQuery {
    pub campaign_id: Option<String>,
    pub content_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub category_id: Option<String>,
    pub limit: Option<i64>,
}
