//! Per-identity violation rollup for the heatmap and top-N lists.
//!
//! Returns one row per (identity, policy) where decision = 'deny'. Each row
//! carries `total_count` (allow + deny for the same pair) so callers can
//! render deny/total ratios. The identity dimension is dispatched at the
//! Rust layer rather than via string interpolation into SQL — every branch
//! is compile-time-verified by `sqlx::query!` against the live schema.

use serde::Serialize;


#[derive(Debug, Clone, Copy)]
pub enum IdentityGroupBy {
    User,
    Agent,
    AgentScope,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdentityViolations {
    // Why: polymorphic identity dimension (user/agent/agent_scope), no single typed-ID equivalent
    pub identity_id: String,
    pub policy: String,
    pub deny_count: i64,
    pub total_count: i64,
}
