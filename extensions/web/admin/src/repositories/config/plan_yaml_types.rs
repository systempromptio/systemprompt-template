//! Wire types for the bootstrap plan loader (`access-control/plans.yaml`).
//!
//! Plans and organizations are web-owned tables, so unlike role rules — which
//! core parses through `AccessControlConfig` — this schema is ours. The grants
//! a plan carries are deliberately a thin shape: they become
//! `access_control_rules` rows through core's repository, which owns what a
//! rule means.

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlansDoc {
    #[serde(default)]
    pub plans: Vec<YamlPlan>,
    #[serde(default)]
    pub organizations: Vec<YamlOrganization>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YamlPlan {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub seat_limit: Option<i32>,
    /// Authored in dollars because that is how a contract is written; stored
    /// as microdollars because that is how `ai_requests` accounts for cost.
    #[serde(default)]
    pub monthly_cost_cap_usd: Option<f64>,
    /// Soft warning threshold below the cap. Crossing it never denies a
    /// request — the gateway guard records it and the dashboard shows
    /// proximity. Requires a cap, and must be below it; the loader rejects
    /// anything else. Omit for no warning.
    #[serde(default)]
    pub monthly_cost_warn_usd: Option<f64>,
    /// The monthly licence fee. The cap above is what the customer may spend;
    /// this is what they pay, and the difference is the margin the enterprise
    /// dashboard reports. Absent means a non-billed plan.
    #[serde(default)]
    pub monthly_price_usd: Option<f64>,
    #[serde(default)]
    pub grants: Vec<YamlGrant>,
}

/// One entitlement. Mirrors an `access_control_rules` row minus the subject,
/// which is supplied per organization when the plan is projected.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct YamlGrant {
    pub entity_type: String,
    pub entity_id: String,
    #[serde(default = "default_access")]
    pub access: String,
}

fn default_access() -> String {
    "allow".to_owned()
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YamlOrganization {
    pub slug: String,
    pub name: String,
    pub plan: String,
    #[serde(default)]
    pub seat_limit_override: Option<i32>,
    /// Email domains whose SSO arrivals join this organization.
    #[serde(default)]
    pub email_domains: Vec<String>,
    #[serde(default = "default_status")]
    pub status: String,
    /// The operator's own tenant. Its members administer every organization,
    /// and no SSO arrival ever joins it, whatever domain they present.
    #[serde(default)]
    pub platform: bool,
}

fn default_status() -> String {
    "active".to_owned()
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PlanLoadReport {
    pub plans_upserted: usize,
    pub organizations_upserted: usize,
    pub grants_projected: usize,
}
