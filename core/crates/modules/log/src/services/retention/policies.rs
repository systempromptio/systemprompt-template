use crate::models::LogLevel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub name: String,
    pub level: Option<LogLevel>,
    pub module: Option<String>,
    pub retention_days: u32,
}

impl RetentionPolicy {
    pub fn new(name: impl Into<String>, retention_days: u32) -> Self {
        Self {
            name: name.into(),
            level: None,
            module: None,
            retention_days,
        }
    }

    #[must_use]
    pub const fn with_level(mut self, level: LogLevel) -> Self {
        self.level = Some(level);
        self
    }

    #[must_use]
    pub fn with_module(mut self, module: impl Into<String>) -> Self {
        self.module = Some(module.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionConfig {
    pub enabled: bool,
    pub schedule: String,
    pub policies: Vec<RetentionPolicy>,
    pub vacuum_after_cleanup: bool,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            schedule: "0 0 2 * * *".to_string(),
            policies: vec![
                RetentionPolicy::new("debug_logs", 1).with_level(LogLevel::Debug),
                RetentionPolicy::new("info_logs", 7).with_level(LogLevel::Info),
                RetentionPolicy::new("warnings", 30).with_level(LogLevel::Warn),
                RetentionPolicy::new("errors", 90).with_level(LogLevel::Error),
            ],
            vacuum_after_cleanup: true,
        }
    }
}

impl RetentionConfig {
    #[must_use]
    pub fn with_schedule(mut self, schedule: impl Into<String>) -> Self {
        self.schedule = schedule.into();
        self
    }

    #[must_use]
    pub fn add_policy(mut self, policy: RetentionPolicy) -> Self {
        self.policies.push(policy);
        self
    }

    #[must_use]
    pub const fn vacuum_enabled(mut self, enabled: bool) -> Self {
        self.vacuum_after_cleanup = enabled;
        self
    }

    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}
