//! API request and response types.

use serde::{Deserialize, Serialize};

use crate::models::UtmParams;

/// Request to generate a tracking link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateLinkRequest {
    pub target_url: String,
    pub campaign_name: Option<String>,
    pub utm_params: Option<UtmParams>,
    pub source_content_id: Option<String>,
}

/// Response with a generated link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateLinkResponse {
    pub id: String,
    pub short_code: String,
    pub short_url: String,
    pub target_url: String,
}

/// Query parameters for content journey.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentJourneyQuery {
    pub content_id: String,
}
