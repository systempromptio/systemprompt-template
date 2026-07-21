pub mod acl_detect;
pub mod acl_yaml_loader;
pub mod acl_yaml_snapshot;
pub mod acl_yaml_types;
pub mod agents;
pub mod audit;
pub mod chain;
pub mod counts;
pub mod decisions;
pub mod effective;
pub mod filter_options;
pub mod gateway;
pub mod gateway_acl;
pub mod hook_events;
pub mod identity;
pub mod jobs;
pub mod rankings;
pub mod resolve;
pub mod risk_score;
pub mod time_range;

pub use audit::{GovernanceDecisionRecord, insert_governance_decision};

pub use counts::{
    fetch_governance_counts, fetch_governance_counts_windowed, fetch_per_policy_counts,
    fetch_per_policy_counts_windowed,
};
pub use decisions::list_decisions_for_policy;
pub use rankings::{fetch_top_actors, fetch_top_policies};

/// Allow/deny rollup over `governance_decisions`.
#[derive(Debug, Clone, Copy, Default)]
pub struct GovernanceCounts {
    pub total: i64,
    pub allowed: i64,
    pub denied: i64,
    pub secret_breaches: i64,
}

#[derive(Debug, Clone)]
pub struct PerPolicyCounts {
    pub policy: String,
    pub allowed: i64,
    pub denied: i64,
    pub last_at: Option<chrono::DateTime<chrono::Utc>>,
}
