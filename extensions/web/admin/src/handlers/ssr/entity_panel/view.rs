//! View model of the "Who gets this" panel, one shape for every entity kind.
//!
//! Decisions travel as one of four words — `allow`, `deny`, `none`, and
//! `open`/`closed` for a default — which `components/access-badge` renders
//! as *Allowed*, *Denied*, *No rule*, *Open* and *Closed*. Nothing here
//! carries a tone or a display word for a decision, so no page can drift
//! back to "granted" or "included".

use serde::Serialize;

use crate::handlers::ssr::types::PickableUserView;
use crate::repositories::sync::attention::ReviewedEntity;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ReachView {
    pub label: String,
    pub band_label: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PanelRuleView {
    pub id: String,
    pub band: String,
    pub subject: String,
    pub subject_label: String,
    pub decision: &'static str,
    pub why: String,
    pub source_label: String,
    pub source_tone: &'static str,
    pub until: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BandRulesView {
    pub band: String,
    pub label: &'static str,
    pub rules: Vec<PanelRuleView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OptionView {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct SubjectOptions {
    pub roles: Vec<OptionView>,
    pub groups: Vec<OptionView>,
    pub projects: Vec<OptionView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct WhyBandView {
    pub label: String,
    pub precedence: u16,
    pub held: String,
    pub decision: &'static str,
    pub verdict_label: &'static str,
    pub verdict_tone: &'static str,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct WhyView {
    pub query: String,
    pub found: bool,
    pub person: String,
    pub decision: String,
    pub decided_by: String,
    pub detail: String,
    pub bands: Vec<WhyBandView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EntityAccessView {
    pub entity_type: String,
    pub entity_id: String,
    pub key: String,
    pub default_decision: &'static str,
    pub headline: String,
    pub nobody: bool,
    pub reaches: Vec<ReachView>,
    pub bands: Vec<BandRulesView>,
    pub rule_count: usize,
    pub review: Option<ReviewedEntity>,
    pub not_live: bool,
    pub drift_unreadable: Option<String>,
    pub subjects: SubjectOptions,
    pub people: Vec<PickableUserView>,
    pub can_write: bool,
    pub note: Option<&'static str>,
    pub why: Option<WhyView>,
    pub page_url: String,
    pub sync_url: &'static str,
}
