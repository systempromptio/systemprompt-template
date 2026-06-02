//! Data-collection orchestration for the Governance Policies page.
//!
//! Runs the six decision-count / leaderboard queries in parallel and collapses
//! each `Result` into a logged default, then indexes the per-policy rows by id
//! so the view layer can join them against the registered policy chain.

use std::collections::HashMap;

use sqlx::PgPool;

use crate::repositories;

use super::{TOP_ACTORS_LIMIT, TOP_POLICIES_LIMIT, WINDOW_24H_SECS};

pub(super) struct GovernanceData {
    pub lifetime: repositories::governance::GovernanceCounts,
    pub window: repositories::governance::GovernanceCounts,
    pub lifetime_by_id: HashMap<String, repositories::governance::PerPolicyCounts>,
    pub window_by_id: HashMap<String, repositories::governance::PerPolicyCounts>,
    pub top_tools: Vec<crate::types::TopPolicy>,
    pub top_actors: Vec<crate::types::TopActor>,
}

pub(super) async fn fetch_governance_data(pool: &PgPool) -> GovernanceData {
    let (lifetime, window, per_policy_lifetime, per_policy_window, top_tools, top_actors) = tokio::join!(
        repositories::governance::fetch_governance_counts(pool),
        repositories::governance::fetch_governance_counts_windowed(pool, WINDOW_24H_SECS),
        repositories::governance::fetch_per_policy_counts(pool),
        repositories::governance::fetch_per_policy_counts_windowed(pool, WINDOW_24H_SECS),
        repositories::governance::fetch_top_policies(pool, WINDOW_24H_SECS, TOP_POLICIES_LIMIT),
        repositories::governance::fetch_top_actors(pool, WINDOW_24H_SECS, TOP_ACTORS_LIMIT),
    );

    let lifetime = lifetime.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "governance lifetime counts query failed");
        repositories::governance::GovernanceCounts::default()
    });
    let window = window.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "governance windowed counts query failed");
        repositories::governance::GovernanceCounts::default()
    });
    let per_policy_lifetime_rows = per_policy_lifetime.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "per-policy lifetime counts query failed");
        Vec::new()
    });
    let per_policy_window_rows = per_policy_window.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "per-policy windowed counts query failed");
        Vec::new()
    });
    let top_tools = top_tools.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "top denying tools query failed");
        Vec::new()
    });
    let top_actors = top_actors.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "top denied actors query failed");
        Vec::new()
    });

    let lifetime_by_id: HashMap<String, repositories::governance::PerPolicyCounts> =
        per_policy_lifetime_rows
            .into_iter()
            .map(|r| (r.policy.clone(), r))
            .collect();
    let window_by_id: HashMap<String, repositories::governance::PerPolicyCounts> =
        per_policy_window_rows
            .into_iter()
            .map(|r| (r.policy.clone(), r))
            .collect();

    GovernanceData {
        lifetime,
        window,
        lifetime_by_id,
        window_by_id,
        top_tools,
        top_actors,
    }
}
