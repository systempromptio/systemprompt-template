mod usage;

pub use usage::{UsageSnapshot, UsageSummary};

use crate::admin::numeric;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct TierLimits {
    #[serde(default = "IngestionLimits::free_default")]
    pub ingestion: IngestionLimits,
    #[serde(default = "EntityLimits::free_default")]
    pub entities: EntityLimits,
    #[serde(default = "FeatureFlags::free_default")]
    pub features: FeatureFlags,
    #[serde(default = "ApiLimits::free_default")]
    pub api: ApiLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct IngestionLimits {
    #[serde(alias = "events_per_day")]
    pub events: i64,
    #[serde(alias = "content_bytes_per_day")]
    pub content_bytes: i64,
    #[serde(alias = "sessions_per_day")]
    pub sessions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct EntityLimits {
    #[serde(alias = "max_skills")]
    pub skills: i64,
    #[serde(alias = "max_agents")]
    pub agents: i64,
    #[serde(alias = "max_plugins")]
    pub plugins: i64,
    #[serde(alias = "max_mcp_servers")]
    pub mcp_servers: i64,
    #[serde(alias = "max_hooks")]
    pub hooks: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct AiFeatures {
    pub session_analysis: bool,
    pub daily_summaries: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct FeatureFlags {
    pub ai: AiFeatures,
    pub apm_metrics: bool,
    pub gamification: bool,
    pub export_zip: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct ApiLimits {
    pub requests_per_minute: i64,
}

impl TierLimits {
    #[must_use]
    pub fn free_default() -> Self {
        Self {
            ingestion: IngestionLimits::free_default(),
            entities: EntityLimits::free_default(),
            features: FeatureFlags::free_default(),
            api: ApiLimits::free_default(),
        }
    }
}

impl IngestionLimits {
    #[must_use]
    pub fn free_default() -> Self {
        Self {
            events: 500,
            content_bytes: 10 * 1024 * 1024,
            sessions: 10,
        }
    }
}
impl EntityLimits {
    #[must_use]
    pub fn free_default() -> Self {
        Self {
            skills: 5,
            agents: 3,
            plugins: 2,
            mcp_servers: 3,
            hooks: 3,
        }
    }
}
impl FeatureFlags {
    #[must_use]
    pub fn free_default() -> Self {
        Self {
            ai: AiFeatures {
                session_analysis: false,
                daily_summaries: false,
            },
            apm_metrics: false,
            gamification: true,
            export_zip: false,
        }
    }
}
impl ApiLimits {
    #[must_use]
    pub fn free_default() -> Self {
        Self {
            requests_per_minute: 30,
        }
    }
}

#[derive(Clone, Copy)]
pub enum LimitCheck {
    IngestEvent,
    IngestContentBytes(i64),
    IngestSession,
    CreateSkill,
    CreateAgent,
    CreatePlugin,
    CreateMcpServer,
    CreateHook,
    FeatureAccess(Feature),
}

#[derive(Clone, Copy)]
pub enum Feature {
    AiSessionAnalysis,
    AiDailySummaries,
    ApmMetrics,
    ExportZip,
}

#[derive(Debug, Clone, Serialize)]
pub struct LimitCheckResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub usage_pct: f64,
    pub limit_value: i64,
    pub current_value: i64,
}

impl LimitCheckResult {
    #[must_use]
    pub fn allowed() -> Self {
        Self {
            allowed: true,
            reason: None,
            usage_pct: 0.0,
            limit_value: 0,
            current_value: 0,
        }
    }
    #[must_use]
    pub fn with_usage(limit: i64, current: i64) -> Self {
        let pct = if limit > 0 {
            numeric::to_f64(current) / numeric::to_f64(limit)
        } else {
            0.0
        };
        Self {
            allowed: current < limit,
            reason: if current >= limit {
                Some(format!("Limit reached: {current}/{limit}"))
            } else {
                None
            },
            usage_pct: pct,
            limit_value: limit,
            current_value: current,
        }
    }
    #[must_use]
    pub fn feature_denied(feature_name: &str) -> Self {
        Self {
            allowed: false,
            reason: Some(format!(
                "Feature not available on your plan: {feature_name}"
            )),
            usage_pct: 1.0,
            limit_value: 0,
            current_value: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageWarning {
    pub category: String,
    pub message: String,
    pub usage_pct: f64,
}
