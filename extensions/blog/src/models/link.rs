//! Link tracking models for campaign analytics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use systemprompt::identifiers::{
    CampaignId, ContentId, ContextId, LinkClickId, LinkId, SessionId, TaskId, UserId,
};

/// A trackable campaign link.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CampaignLink {
    pub id: LinkId,
    pub short_code: String,
    pub target_url: String,
    pub link_type: String,
    pub campaign_id: Option<CampaignId>,
    pub campaign_name: Option<String>,
    pub source_content_id: Option<ContentId>,
    pub source_page: Option<String>,
    pub utm_params: Option<String>,
    pub link_text: Option<String>,
    pub link_position: Option<String>,
    pub destination_type: Option<String>,
    pub click_count: Option<i32>,
    pub unique_click_count: Option<i32>,
    pub conversion_count: Option<i32>,
    pub is_active: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl CampaignLink {
    /// Get the full URL with UTM parameters appended.
    pub fn get_full_url(&self) -> String {
        if let Some(ref params_json) = self.utm_params {
            if let Ok(params) = serde_json::from_str::<UtmParams>(params_json) {
                let query = params.to_query_string();
                if !query.is_empty() {
                    let separator = if self.target_url.contains('?') {
                        "&"
                    } else {
                        "?"
                    };
                    return format!("{}{}{}", self.target_url, separator, query);
                }
            }
        }
        self.target_url.clone()
    }
}

/// A click event on a campaign link.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LinkClick {
    pub id: LinkClickId,
    pub link_id: LinkId,
    pub session_id: SessionId,
    pub user_id: Option<UserId>,
    pub context_id: Option<ContextId>,
    pub task_id: Option<TaskId>,
    pub referrer_page: Option<String>,
    pub referrer_url: Option<String>,
    pub clicked_at: Option<DateTime<Utc>>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub device_type: Option<String>,
    pub country: Option<String>,
    pub is_first_click: Option<bool>,
    pub is_conversion: Option<bool>,
    pub conversion_at: Option<DateTime<Utc>>,
    pub time_on_page_seconds: Option<i32>,
    pub scroll_depth_percent: Option<i32>,
}

/// The type of link.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LinkType {
    Redirect,
    Utm,
    Both,
}

impl LinkType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Redirect => "redirect",
            Self::Utm => "utm",
            Self::Both => "both",
        }
    }
}

impl std::fmt::Display for LinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// The destination type of a link.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DestinationType {
    Internal,
    External,
}

impl DestinationType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Internal => "internal",
            Self::External => "external",
        }
    }
}

impl std::fmt::Display for DestinationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// UTM tracking parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtmParams {
    pub source: Option<String>,
    pub medium: Option<String>,
    pub campaign: Option<String>,
    pub term: Option<String>,
    pub content: Option<String>,
}

impl UtmParams {
    /// Convert UTM params to a query string.
    pub fn to_query_string(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref source) = self.source {
            parts.push(format!("utm_source={source}"));
        }
        if let Some(ref medium) = self.medium {
            parts.push(format!("utm_medium={medium}"));
        }
        if let Some(ref campaign) = self.campaign {
            parts.push(format!("utm_campaign={campaign}"));
        }
        if let Some(ref term) = self.term {
            parts.push(format!("utm_term={term}"));
        }
        if let Some(ref content) = self.content {
            parts.push(format!("utm_content={content}"));
        }
        parts.join("&")
    }

    /// Convert to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Performance metrics for a single link.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LinkPerformance {
    pub link_id: LinkId,
    pub click_count: i64,
    pub unique_click_count: i64,
    pub conversion_count: i64,
    pub conversion_rate: Option<f64>,
}

/// Performance metrics for a campaign.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CampaignPerformance {
    pub campaign_id: CampaignId,
    pub total_clicks: i64,
    pub link_count: i64,
    pub unique_visitors: Option<i64>,
    pub conversion_count: Option<i64>,
}

/// A node in the content journey graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentJourneyNode {
    pub source_content_id: ContentId,
    pub target_url: String,
    pub click_count: i32,
}
