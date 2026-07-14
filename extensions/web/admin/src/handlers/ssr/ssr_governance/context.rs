//! Typed template-context structs for the Governance Policies page
//! (`governance.hbs`).

use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct GovernancePageContext {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) lifetime_total: i64,
    pub(super) lifetime_allowed: i64,
    pub(super) lifetime_denied: i64,
    pub(super) window_total: i64,
    pub(super) window_allowed: i64,
    pub(super) window_denied: i64,
    pub(super) window_breaches: i64,
    pub(super) policies: Vec<PolicyRow>,
    pub(super) has_policies: bool,
    pub(super) enforcement: Vec<PolicyRow>,
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

/// One policy card / enforcement-table row. The `enforcement` list reuses the
/// same shape as `policies`, just re-sorted by `window_denied` DESC.
#[derive(Debug, Clone, Serialize)]
pub(super) struct PolicyRow {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) description: String,
    pub(super) enabled: bool,
    pub(super) source_path: String,
    pub(super) params_preview: Vec<ParamPreview>,
    pub(super) has_params: bool,
    pub(super) lifetime_allowed: i64,
    pub(super) lifetime_denied: i64,
    pub(super) window_allowed: i64,
    pub(super) window_denied: i64,
    pub(super) window_evaluations: i64,
    pub(super) deny_rate: String,
    pub(super) has_recent_denies: bool,
    pub(super) last_at: String,
    pub(super) last_at_window: String,
    pub(super) edit_url: String,
    pub(super) decisions_url: String,
    pub(super) deny_decisions_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ParamPreview {
    pub(super) key: String,
    pub(super) value: String,
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
    pub(super) user_id: String,
    pub(super) display_name: String,
    pub(super) email: Option<String>,
    pub(super) deny_count: i64,
    pub(super) secret_count: i64,
    pub(super) total: i64,
    pub(super) decisions_url: String,
}
