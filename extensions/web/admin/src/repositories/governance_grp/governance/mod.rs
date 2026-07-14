//! Governance-decision read queries backing the governance dashboards.
//!
//! Everything here reads `governance_decisions` (sometimes joined to `users`):
//! raw decision lists, allow/deny rollups, sliding-window aggregates for
//! anomaly baselines, and the top-actor / top-policy / grouped-incident
//! rankings.

mod counts;
mod decisions;
mod rankings;
mod windows;

pub use counts::{
    fetch_governance_counts, fetch_governance_counts_windowed, fetch_per_policy_counts,
    fetch_per_policy_counts_windowed,
};
pub use decisions::{
    fetch_governance_events, list_decisions_for_policy, list_governance_decisions,
};
pub use rankings::{fetch_grouped_incidents, fetch_top_actors, fetch_top_policies};
pub use windows::{fetch_baseline_window_samples, fetch_windowed_counts};

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
