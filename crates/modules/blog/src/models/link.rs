use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt_core_database::JsonRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignLink {
    pub id: String,
    pub short_code: String,
    pub target_url: String,
    pub link_type: LinkType,
    pub campaign_id: Option<String>,
    pub campaign_name: Option<String>,
    pub source_content_id: Option<String>,
    pub source_page: Option<String>,
    pub utm_params: Option<UtmParams>,
    pub link_text: Option<String>,
    pub link_position: Option<String>,
    pub destination_type: DestinationType,
    pub click_count: i64,
    pub unique_click_count: i64,
    pub conversion_count: i64,
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LinkType {
    Redirect,
    Utm,
    Both,
}

impl LinkType {
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Redirect => "redirect",
            Self::Utm => "utm",
            Self::Both => "both",
        }
    }
}

impl std::str::FromStr for LinkType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "redirect" => Ok(Self::Redirect),
            "utm" => Ok(Self::Utm),
            "both" => Ok(Self::Both),
            _ => Err(anyhow!("Invalid link type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DestinationType {
    Internal,
    External,
}

impl DestinationType {
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Internal => "internal",
            Self::External => "external",
        }
    }
}

impl std::str::FromStr for DestinationType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "internal" => Ok(Self::Internal),
            "external" => Ok(Self::External),
            _ => Err(anyhow!("Invalid destination type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtmParams {
    #[serde(rename = "utm_source")]
    pub source: Option<String>,
    #[serde(rename = "utm_medium")]
    pub medium: Option<String>,
    #[serde(rename = "utm_campaign")]
    pub campaign: Option<String>,
    #[serde(rename = "utm_term")]
    pub term: Option<String>,
    #[serde(rename = "utm_content")]
    pub content: Option<String>,
}

impl UtmParams {
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();

        if let Some(source) = &self.source {
            params.push(format!("utm_source={}", urlencoding::encode(source)));
        }
        if let Some(medium) = &self.medium {
            params.push(format!("utm_medium={}", urlencoding::encode(medium)));
        }
        if let Some(campaign) = &self.campaign {
            params.push(format!("utm_campaign={}", urlencoding::encode(campaign)));
        }
        if let Some(term) = &self.term {
            params.push(format!("utm_term={}", urlencoding::encode(term)));
        }
        if let Some(content) = &self.content {
            params.push(format!("utm_content={}", urlencoding::encode(content)));
        }

        params.join("&")
    }

    pub fn from_json(value: &str) -> Result<Self> {
        serde_json::from_str(value).map_err(|e| anyhow!("Failed to parse UTM params: {e}"))
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| anyhow!("Failed to serialize UTM params: {e}"))
    }
}

impl CampaignLink {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing id"))?
            .to_string();

        let short_code = row
            .get("short_code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing short_code"))?
            .to_string();

        let target_url = row
            .get("target_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing target_url"))?
            .to_string();

        let link_type_str = row
            .get("link_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing link_type"))?;
        let link_type = link_type_str.parse()?;

        let destination_type_str = row
            .get("destination_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing destination_type"))?;
        let destination_type = destination_type_str.parse()?;

        let campaign_id = row
            .get("campaign_id")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let campaign_name = row
            .get("campaign_name")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let source_content_id = row
            .get("source_content_id")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let source_page = row
            .get("source_page")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let utm_params = row
            .get("utm_params")
            .and_then(|v| v.as_str())
            .and_then(|s| UtmParams::from_json(s).ok());

        let link_text = row
            .get("link_text")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let link_position = row
            .get("link_position")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let click_count = row
            .get("click_count")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid click_count"))?;

        let unique_click_count = row
            .get("unique_click_count")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid unique_click_count"))?;

        let conversion_count = row
            .get("conversion_count")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid conversion_count"))?;

        let is_active = row
            .get("is_active")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| anyhow!("Missing or invalid is_active"))?;

        let expires_at = row
            .get("expires_at")
            .and_then(systemprompt_core_database::parse_database_datetime);

        let created_at = row
            .get("created_at")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .ok_or_else(|| anyhow!("Missing or invalid created_at"))?;

        let updated_at = row
            .get("updated_at")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .ok_or_else(|| anyhow!("Missing or invalid updated_at"))?;

        Ok(Self {
            id,
            short_code,
            target_url,
            link_type,
            campaign_id,
            campaign_name,
            source_content_id,
            source_page,
            utm_params,
            link_text,
            link_position,
            destination_type,
            click_count,
            unique_click_count,
            conversion_count,
            is_active,
            expires_at,
            created_at,
            updated_at,
        })
    }

    pub fn get_full_url(&self) -> String {
        if let Some(utm_params) = &self.utm_params {
            let query_string = utm_params.to_query_string();
            if query_string.is_empty() {
                self.target_url.clone()
            } else {
                let separator = if self.target_url.contains('?') {
                    "&"
                } else {
                    "?"
                };
                format!("{}{}{}", self.target_url, separator, query_string)
            }
        } else {
            self.target_url.clone()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkClick {
    pub id: String,
    pub link_id: String,
    pub session_id: String,
    pub user_id: Option<String>,
    pub context_id: Option<String>,
    pub task_id: Option<String>,
    pub referrer_page: Option<String>,
    pub referrer_url: Option<String>,
    pub clicked_at: DateTime<Utc>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub device_type: Option<String>,
    pub country: Option<String>,
    pub is_first_click: bool,
    pub is_conversion: bool,
    pub conversion_at: Option<DateTime<Utc>>,
    pub time_on_page_seconds: Option<i32>,
    pub scroll_depth_percent: Option<i32>,
}

impl LinkClick {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing id"))?
            .to_string();

        let link_id = row
            .get("link_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing link_id"))?
            .to_string();

        let session_id = row
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing session_id"))?
            .to_string();

        let user_id = row
            .get("user_id")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let context_id = row
            .get("context_id")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let task_id = row
            .get("task_id")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let referrer_page = row
            .get("referrer_page")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let referrer_url = row
            .get("referrer_url")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let clicked_at = row
            .get("clicked_at")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .ok_or_else(|| anyhow!("Missing or invalid clicked_at"))?;

        let user_agent = row
            .get("user_agent")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let ip_address = row
            .get("ip_address")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let device_type = row
            .get("device_type")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let country = row
            .get("country")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let is_first_click = row
            .get("is_first_click")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| anyhow!("Missing or invalid is_first_click"))?;

        let is_conversion = row
            .get("is_conversion")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| anyhow!("Missing or invalid is_conversion"))?;

        let conversion_at = row
            .get("conversion_at")
            .and_then(systemprompt_core_database::parse_database_datetime);

        let time_on_page_seconds = row
            .get("time_on_page_seconds")
            .and_then(serde_json::Value::as_i64)
            .map(|v| v as i32);

        let scroll_depth_percent = row
            .get("scroll_depth_percent")
            .and_then(serde_json::Value::as_i64)
            .map(|v| v as i32);

        Ok(Self {
            id,
            link_id,
            session_id,
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
            is_conversion,
            conversion_at,
            time_on_page_seconds,
            scroll_depth_percent,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkPerformance {
    pub id: String,
    pub short_code: String,
    pub target_url: String,
    pub campaign_id: Option<String>,
    pub campaign_name: Option<String>,
    pub source_page: Option<String>,
    pub click_count: i64,
    pub unique_click_count: i64,
    pub conversion_count: i64,
    pub session_count: i64,
    pub user_count: i64,
    pub actual_conversions: i64,
    pub conversion_rate: Option<f64>,
    pub first_click_at: Option<DateTime<Utc>>,
    pub last_click_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl LinkPerformance {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing id"))?
            .to_string();

        let short_code = row
            .get("short_code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing short_code"))?
            .to_string();

        let target_url = row
            .get("target_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing target_url"))?
            .to_string();

        let campaign_id = row
            .get("campaign_id")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let campaign_name = row
            .get("campaign_name")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let source_page = row
            .get("source_page")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let click_count = row
            .get("click_count")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid click_count"))?;

        let unique_click_count = row
            .get("unique_click_count")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid unique_click_count"))?;

        let conversion_count = row
            .get("conversion_count")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid conversion_count"))?;

        let session_count = row
            .get("session_count")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid session_count"))?;

        let user_count = row
            .get("user_count")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid user_count"))?;

        let actual_conversions = row
            .get("actual_conversions")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid actual_conversions"))?;

        let conversion_rate = row.get("conversion_rate").and_then(serde_json::Value::as_f64);

        let first_click_at = row
            .get("first_click_at")
            .and_then(systemprompt_core_database::parse_database_datetime);

        let last_click_at = row
            .get("last_click_at")
            .and_then(systemprompt_core_database::parse_database_datetime);

        let created_at = row
            .get("created_at")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .ok_or_else(|| anyhow!("Missing or invalid created_at"))?;

        let updated_at = row
            .get("updated_at")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .ok_or_else(|| anyhow!("Missing or invalid updated_at"))?;

        Ok(Self {
            id,
            short_code,
            target_url,
            campaign_id,
            campaign_name,
            source_page,
            click_count,
            unique_click_count,
            conversion_count,
            session_count,
            user_count,
            actual_conversions,
            conversion_rate,
            first_click_at,
            last_click_at,
            created_at,
            updated_at,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignPerformance {
    pub campaign_id: String,
    pub campaign_name: Option<String>,
    pub link_count: i64,
    pub total_clicks: i64,
    pub total_unique_clicks: i64,
    pub total_conversions: i64,
    pub session_count: i64,
    pub conversion_rate: Option<f64>,
    pub campaign_start: Option<DateTime<Utc>>,
    pub last_activity: Option<DateTime<Utc>>,
}

impl CampaignPerformance {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let campaign_id = row
            .get("campaign_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing campaign_id"))?
            .to_string();

        let campaign_name = row
            .get("campaign_name")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let link_count = row
            .get("link_count")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid link_count"))?;

        let total_clicks = row
            .get("total_clicks")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid total_clicks"))?;

        let total_unique_clicks = row
            .get("total_unique_clicks")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid total_unique_clicks"))?;

        let total_conversions = row
            .get("total_conversions")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid total_conversions"))?;

        let session_count = row
            .get("session_count")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid session_count"))?;

        let conversion_rate = row.get("conversion_rate").and_then(serde_json::Value::as_f64);

        let campaign_start = row
            .get("campaign_start")
            .and_then(systemprompt_core_database::parse_database_datetime);

        let last_activity = row
            .get("last_activity")
            .and_then(systemprompt_core_database::parse_database_datetime);

        Ok(Self {
            campaign_id,
            campaign_name,
            link_count,
            total_clicks,
            total_unique_clicks,
            total_conversions,
            session_count,
            conversion_rate,
            campaign_start,
            last_activity,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentJourneyNode {
    pub source_content_id: String,
    pub source_slug: String,
    pub source_title: String,
    pub target_url: String,
    pub target_slug: Option<String>,
    pub target_title: Option<String>,
    pub click_count: i64,
    pub unique_sessions: i64,
    pub conversions: i64,
    pub conversion_rate: Option<f64>,
}

impl ContentJourneyNode {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let source_content_id = row
            .get("source_content_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing source_content_id"))?
            .to_string();

        let source_slug = row
            .get("source_slug")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing source_slug"))?
            .to_string();

        let source_title = row
            .get("source_title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing source_title"))?
            .to_string();

        let target_url = row
            .get("target_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing target_url"))?
            .to_string();

        let target_slug = row
            .get("target_slug")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let target_title = row
            .get("target_title")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let click_count = row
            .get("click_count")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid click_count"))?;

        let unique_sessions = row
            .get("unique_sessions")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid unique_sessions"))?;

        let conversions = row
            .get("conversions")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid conversions"))?;

        let conversion_rate = row.get("conversion_rate").and_then(serde_json::Value::as_f64);

        Ok(Self {
            source_content_id,
            source_slug,
            source_title,
            target_url,
            target_slug,
            target_title,
            click_count,
            unique_sessions,
            conversions,
            conversion_rate,
        })
    }
}
