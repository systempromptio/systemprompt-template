//! Risk score computation for the Identity & Violations page.
//!
//! Reads weights from `services/governance/risk_score.yaml` (cached after
//! first read; the cache is invalidated by reloading the process — the YAML
//! is small and we cap re-reads at one per cold start). The formula is:
//!
//! ```text
//! raw        = deny_count          * deny_weight
//!            + secret_breach_count * secret_breach_weight
//!            + scope_violation_count * scope_violation_weight
//! normalised = raw / max(activity_volume, normalization_floor)
//! score      = clamp(normalised * scale, 0.0, 100.0)
//! ```


use serde::{Deserialize, Serialize};


/// Weights loaded from `services/governance/risk_score.yaml`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RiskScoreWeights {
    pub deny_weight: f64,
    pub secret_breach_weight: f64,
    pub scope_violation_weight: f64,
    pub normalization_floor: f64,
    pub scale: f64,
}

impl Default for RiskScoreWeights {
    fn default() -> Self {
        Self {
            deny_weight: 1.0,
            secret_breach_weight: 3.0,
            scope_violation_weight: 2.0,
            normalization_floor: 10.0,
            scale: 50.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ViolationCounts {
    pub deny_count: i64,
    pub secret_breach_count: i64,
    pub scope_violation_count: i64,
    pub activity_volume: i64,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct RiskScore {
    pub raw: f64,
    pub normalised: f64,
    pub score: f64,
}

#[must_use]
pub fn compute_risk_score(v: &ViolationCounts, w: RiskScoreWeights) -> RiskScore {
    let deny_term = (v.deny_count as f64).mul_add(
        w.deny_weight,
        (v.secret_breach_count as f64) * w.secret_breach_weight,
    );
    let raw = (v.scope_violation_count as f64).mul_add(w.scope_violation_weight, deny_term);
    let denom = (v.activity_volume as f64).max(w.normalization_floor);
    let normalised = if denom > 0.0 { raw / denom } else { 0.0 };
    let score = (normalised * w.scale).clamp(0.0, 100.0);
    RiskScore {
        raw,
        normalised,
        score,
    }
}

/// Group-by alignment with `identity::IdentityGroupBy`.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityGroupBy {
    User,
    Agent,
    AgentScope,
}

impl IdentityGroupBy {
    pub fn parse(value: Option<&str>) -> Self {
        match value.unwrap_or("user") {
            "agent" => Self::Agent,
            "agent_scope" => Self::AgentScope,
            _ => Self::User,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Agent => "agent",
            Self::AgentScope => "agent_scope",
        }
    }
}
