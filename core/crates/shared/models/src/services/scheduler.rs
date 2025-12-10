use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobConfig {
    pub name: String,
    pub enabled: bool,
    pub schedule: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub enabled: bool,
    pub jobs: Vec<JobConfig>,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            jobs: vec![
                JobConfig {
                    name: "cleanup_anonymous_users".into(),
                    enabled: true,
                    schedule: "0 0 3 * * *".into(),
                },
                JobConfig {
                    name: "cleanup_empty_contexts".into(),
                    enabled: true,
                    schedule: "0 0 * * * *".into(),
                },
                JobConfig {
                    name: "cleanup_inactive_sessions".into(),
                    enabled: true,
                    schedule: "0 0 * * * *".into(),
                },
                JobConfig {
                    name: "database_cleanup".into(),
                    enabled: true,
                    schedule: "0 0 4 * * *".into(),
                },
                JobConfig {
                    name: "publish_content".into(),
                    enabled: true,
                    schedule: "0 */30 * * * *".into(),
                },
                JobConfig {
                    name: "evaluate_conversations".into(),
                    enabled: true,
                    schedule: "0 */30 * * * *".into(),
                },
            ],
        }
    }
}
