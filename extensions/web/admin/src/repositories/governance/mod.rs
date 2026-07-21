//! Persistence for the governance record — what actually happened.
//!
//! Every tool-call decision, the policies that produced it, and the rollups the
//! audit pages read are served from here. Rows are append-only history: nothing
//! in this module changes what is allowed, only what was decided.
//!
//! The configured side of that pairing — gateway routes, agent definitions,
//! the access-control YAML — lives in [`super::config`].

pub mod audit;
pub mod chain;
pub mod counts;
pub mod decisions;
pub mod effective;
pub mod filter_options;
pub mod hook_events;
pub mod rankings;
pub mod resolve;

pub use audit::{GovernanceDecisionRecord, insert_governance_decision};

pub use counts::{
    get_governance_counts, get_governance_counts_windowed, list_per_policy_counts,
    list_per_policy_counts_windowed,
};
pub use decisions::list_decisions_for_policy;
pub use rankings::{list_top_actors, list_top_policies};

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
