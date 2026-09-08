//! Typed template-context structs for the Governance Policies page
//! (`governance.hbs`).

use serde::Serialize;
use systemprompt::identifiers::UserId;

use crate::handlers::ssr::types::BreadcrumbView;

#[derive(Debug, Serialize)]
pub(super) struct GovernancePageContext {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) breadcrumbs: Vec<BreadcrumbView>,
    pub(super) kpis: Vec<GovernanceKpiView>,
    pub(super) policies: Vec<PolicyRow>,
    pub(super) policy_count: usize,
    pub(super) has_policies: bool,
    pub(super) has_enforcement_activity: bool,
    pub(super) top_tools: Vec<TopToolRow>,
    pub(super) has_top_tools: bool,
    pub(super) top_actors: Vec<TopActorRow>,
    pub(super) has_top_actors: bool,
    pub(super) orphans: Vec<OrphanRow>,
    pub(super) has_orphans: bool,
    pub(super) orphans_count: usize,
    pub(super) config_path: &'static str,
}

// Why: the five numbers an operator reads before opening any policy — the
// window's volume, its split, the secret breaches inside it, and lifetime for
// scale. One band, one line each, so the chain table stays above the fold.
#[derive(Debug, Serialize)]
pub(super) struct GovernanceKpiView {
    pub(super) label: &'static str,
    pub(super) value: String,
    pub(super) note: String,
    pub(super) tone: &'static str,
    pub(super) href: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct PolicyRow {
    pub(super) order: usize,
    pub(super) id: String,
    pub(super) name: String,
    pub(super) description: String,
    pub(super) enabled: bool,
    pub(super) state: &'static str,
    pub(super) state_tone: &'static str,
    pub(super) params_line: String,
    pub(super) has_params: bool,
    pub(super) lifetime_allowed: i64,
    pub(super) lifetime_denied: i64,
    pub(super) window_allowed: i64,
    pub(super) window_denied: i64,
    pub(super) window_evaluations: i64,
    pub(super) deny_rate: String,
    pub(super) has_recent_denies: bool,
    pub(super) last_at: String,
    pub(super) edit_url: String,
    pub(super) decisions_url: String,
    pub(super) deny_decisions_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct OrphanRow {
    pub(super) id: String,
    pub(super) allowed: i64,
    pub(super) denied: i64,
    pub(super) last_at: String,
}

#[derive(Debug, Serialize)]
pub(super) struct TopToolRow {
    pub(super) policy: String,
    pub(super) tool_name: String,
    pub(super) hits: i64,
    pub(super) distinct_actors: i64,
    pub(super) decisions_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct TopActorRow {
    pub(super) user_id: UserId,
    pub(super) display_name: String,
    pub(super) email: String,
    pub(super) deny_count: i64,
    pub(super) secret_count: i64,
    pub(super) total: i64,
    pub(super) decisions_url: String,
    pub(super) user_url: String,
}
