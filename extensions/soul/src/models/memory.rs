use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::identifiers::MemoryId;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SoulMemory {
    pub id: MemoryId,
    pub memory_type: String,
    pub category: String,
    pub subject: String,
    pub content: String,
    pub context_text: Option<String>,
    pub priority: Option<i32>,
    pub confidence: Option<f32>,
    pub source_task_id: Option<String>,
    pub source_context_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub last_accessed_at: Option<DateTime<Utc>>,
    pub access_count: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    Core,
    LongTerm,
    ShortTerm,
    Working,
}

impl MemoryType {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::LongTerm => "long_term",
            Self::ShortTerm => "short_term",
            Self::Working => "working",
        }
    }

    #[must_use]
    pub fn default_ttl_hours(&self) -> Option<i64> {
        match self {
            Self::Core => None,
            Self::LongTerm => Some(30 * 24),
            Self::ShortTerm => Some(48),
            Self::Working => Some(4),
        }
    }
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for MemoryType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "core" => Ok(Self::Core),
            "long_term" | "longterm" => Ok(Self::LongTerm),
            "short_term" | "shortterm" => Ok(Self::ShortTerm),
            "working" => Ok(Self::Working),
            _ => Err(format!("Unknown memory type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryCategory {
    Fact,
    Preference,
    Goal,
    Relationship,
    Reminder,
}

impl MemoryCategory {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Fact => "fact",
            Self::Preference => "preference",
            Self::Goal => "goal",
            Self::Relationship => "relationship",
            Self::Reminder => "reminder",
        }
    }
}

impl std::fmt::Display for MemoryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for MemoryCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fact" => Ok(Self::Fact),
            "preference" => Ok(Self::Preference),
            "goal" => Ok(Self::Goal),
            "relationship" => Ok(Self::Relationship),
            "reminder" => Ok(Self::Reminder),
            _ => Err(format!("Unknown memory category: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMemoryParams {
    pub memory_type: MemoryType,
    pub category: MemoryCategory,
    pub subject: String,
    pub content: String,
    pub context_text: Option<String>,
    pub priority: Option<i32>,
    pub confidence: Option<f32>,
    pub source_task_id: Option<String>,
    pub source_context_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl CreateMemoryParams {
    #[must_use]
    pub fn new(
        memory_type: MemoryType,
        category: MemoryCategory,
        subject: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            memory_type,
            category,
            subject: subject.into(),
            content: content.into(),
            context_text: None,
            priority: None,
            confidence: None,
            source_task_id: None,
            source_context_id: None,
            tags: None,
            metadata: None,
            expires_at: None,
        }
    }

    #[must_use]
    pub fn with_context_text(mut self, text: impl Into<String>) -> Self {
        self.context_text = Some(text.into());
        self
    }

    #[must_use]
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = Some(priority);
        self
    }

    #[must_use]
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = Some(confidence);
        self
    }

    #[must_use]
    pub fn with_source(mut self, task_id: Option<String>, context_id: Option<String>) -> Self {
        self.source_task_id = task_id;
        self.source_context_id = context_id;
        self
    }

    #[must_use]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    #[must_use]
    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
}
