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

use std::path::PathBuf;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::time_range::TimeRange;

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

#[derive(Debug, Deserialize)]
struct RiskScoreFile {
    risk_score: RiskScoreWeights,
}

static WEIGHTS: OnceLock<RiskScoreWeights> = OnceLock::new();

/// Cached weights from `services/governance/risk_score.yaml`.
///
/// Falls back to [`RiskScoreWeights::default`] if the file is missing or
/// malformed (logged at WARN once).
pub fn weights() -> RiskScoreWeights {
    *WEIGHTS.get_or_init(load_weights)
}

fn load_weights() -> RiskScoreWeights {
    let Some(path) = config_path() else {
        tracing::warn!("ProfileBootstrap unavailable; using default risk_score weights");
        return RiskScoreWeights::default();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        tracing::info!(
            path = %path.display(),
            "risk_score.yaml not found; using default weights"
        );
        return RiskScoreWeights::default();
    };
    match serde_yaml::from_str::<RiskScoreFile>(&text) {
        Ok(parsed) => parsed.risk_score,
        Err(e) => {
            tracing::warn!(
                path = %path.display(),
                error = %e,
                "risk_score.yaml malformed; using default weights"
            );
            RiskScoreWeights::default()
        }
    }
}

fn config_path() -> Option<PathBuf> {
    let bootstrap = systemprompt::config::ProfileBootstrap::get().ok()?;
    Some(PathBuf::from(&bootstrap.paths.services).join("governance/risk_score.yaml"))
}

/// Per-identity violation counts read from the live DB.
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

/// Per-identity violation counts over a window, broken out by the policy
/// categories the risk-score formula cares about (deny / secret / scope).
pub async fn fetch_violation_counts(
    pool: &PgPool,
    range: TimeRange,
    group_by: IdentityGroupBy,
) -> Result<Vec<(String, ViolationCounts)>, sqlx::Error> {
    let identity_expr = match group_by {
        IdentityGroupBy::User => "g.user_id",
        IdentityGroupBy::Agent => "COALESCE(g.agent_id, '')",
        IdentityGroupBy::AgentScope => "COALESCE(g.agent_scope, '')",
    };

    let sql = format!(
        r"SELECT
            {identity_expr} AS identity_id,
            COUNT(*) FILTER (WHERE g.decision = 'deny')::bigint AS deny_count,
            COUNT(*) FILTER (
                WHERE g.decision = 'deny'
                  AND (g.policy = 'secret_scan' OR g.policy = 'secret_injection'
                       OR g.reason ILIKE '%secret%')
            )::bigint AS secret_breach_count,
            COUNT(*) FILTER (
                WHERE g.decision = 'deny'
                  AND (g.policy = 'scope_check' OR g.policy = 'scope')
            )::bigint AS scope_violation_count,
            COUNT(*)::bigint AS activity_volume
          FROM governance_decisions g
          WHERE g.created_at >= $1 AND g.created_at < $2
          GROUP BY identity_id
          HAVING COUNT(*) FILTER (WHERE g.decision = 'deny') > 0
          ORDER BY deny_count DESC, activity_volume DESC
          LIMIT 200",
    );

    let rows = sqlx::query_as::<_, ViolationCountsRow>(&sql)
        .bind(range.from)
        .bind(range.to)
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            (
                r.identity_id,
                ViolationCounts {
                    deny_count: r.deny_count,
                    secret_breach_count: r.secret_breach_count,
                    scope_violation_count: r.scope_violation_count,
                    activity_volume: r.activity_volume,
                },
            )
        })
        .collect())
}

#[derive(sqlx::FromRow)]
struct ViolationCountsRow {
    identity_id: String,
    deny_count: i64,
    secret_breach_count: i64,
    scope_violation_count: i64,
    activity_volume: i64,
}
