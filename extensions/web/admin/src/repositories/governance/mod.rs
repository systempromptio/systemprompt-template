//! Persistence for the governance record — what actually happened.
//!
//! Every tool-call decision, the policies that produced it, and the rollups the
//! audit pages read are served from here. Rows are append-only history: nothing
//! in this module changes what is allowed, only what was decided.
//!
//! The configured side of that pairing — gateway routes, agent definitions,
//! the access-control YAML — lives in [`super::config`].

pub mod approvals;
pub mod chain;
pub mod counts;
pub mod decision_log;
pub mod decisions;
pub mod demo_trace;
pub mod effective;
pub mod filter_options;
pub mod findings;
pub mod hook_events;
pub mod rankings;
pub mod resolve;
pub mod secret_audit_log;
pub mod warnings;

pub use counts::{
    get_governance_counts, get_governance_counts_windowed, list_per_policy_counts,
    list_per_policy_counts_windowed,
};
pub use decisions::list_decisions_for_policy;
// Why: ungated beside `warnings`. The hooks tab of `/admin/governance` renders
// both rankings and the recent hook events, so these are the fork's own
// surface rather than the upstream one the feature compiles out.
pub use rankings::{list_top_actors, list_top_policies};
// Why: not behind `governance-ssr` like its neighbours. That feature exists to
// compile out queries this fork does not serve; the warnings page is a surface
// this fork does ship, and gating it would leave warn mode with no reader.
pub use warnings::{WarningGroup, WarningGroupBy, WarningRollupRow, group_warning_rollup};

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
